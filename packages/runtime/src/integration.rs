//! Integration layer for AI SDK 5 and MCP servers

// async_trait is used implicitly by emotional_agents_core
use emotional_agents_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai_sdk_client;
use crate::config::{AiSdkConfig, McpConfig};
use crate::mcp_manager;
use crate::runtime::{AiResponse, ResponseContext, ToolCall, ToolResult};
use crate::utils::{DecisionRecordExt, EmotionalStateExt};

/// AI SDK 5 integration
pub struct AiSdkIntegration {
    /// Configuration
    config: AiSdkConfig,

    /// AI SDK client
    client: Arc<ai_sdk_client::AiSdkClient>,

    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, AiSession>>>,
}

/// AI session state
struct AiSession {
    /// Session ID
    id: String,

    /// Conversation history
    history: Vec<AiMessage>,

    /// Session metadata
    metadata: HashMap<String, serde_json::Value>,
}

/// AI message format
#[derive(Debug, Clone)]
struct AiMessage {
    role: String,
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
}

impl AiSdkIntegration {
    /// Create new AI SDK integration
    pub async fn new(config: AiSdkConfig) -> Result<Self> {
        let client = Arc::new(
            ai_sdk_client::AiSdkClient::new(ai_sdk_client::Config {
                provider: config.provider.clone(),
                model: config.model.clone(),
                api_key: config.api_key.clone().unwrap_or_else(|| {
                    std::env::var(format!("{}_API_KEY", config.provider.to_uppercase()))
                        .unwrap_or_default()
                }),
                temperature: Some(config.temperature),
                max_tokens: Some(config.max_tokens),
                streaming: config.streaming,
                ..Default::default()
            })
            .await?,
        );

        Ok(Self {
            config,
            client,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate response using AI SDK
    pub async fn generate_response(
        &self,
        context: ResponseContext,
        config: &AiSdkConfig,
    ) -> Result<AiResponse> {
        // Build messages from context
        let mut messages = self.build_messages(&context).await?;

        // Add system prompt
        messages.insert(
            0,
            AiMessage {
                role: "system".to_string(),
                content: self.build_system_prompt(&context)?,
                tool_calls: None,
            },
        );

        // Create AI SDK request
        let request = ai_sdk_client::GenerateRequest {
            messages: messages
                .into_iter()
                .map(|m| ai_sdk_client::Message {
                    role: m.role,
                    content: ai_sdk_client::MessageContent::Text(m.content),
                    name: None,
                    tool_calls: m.tool_calls.map(|calls| {
                        calls
                            .into_iter()
                            .map(|call| ai_sdk_client::ToolCall {
                                id: uuid::Uuid::new_v4().to_string(),
                                function: ai_sdk_client::FunctionCall {
                                    name: call.tool_name,
                                    arguments: call.arguments,
                                },
                                r#type: "function".to_string(),
                            })
                            .collect()
                    }),
                    tool_call_id: None,
                })
                .collect(),
            model: Some(config.model.clone()),
            temperature: Some(config.temperature),
            max_tokens: Some(config.max_tokens as i32),
            tools: if !context.available_tools.is_empty() {
                Some(self.build_tool_definitions(&context.available_tools)?)
            } else {
                None
            },
            ..Default::default()
        };

        // Generate response
        let response = self.client.generate(request).await?;

        // Parse response
        self.parse_ai_response(response).await
    }

    /// Build messages from context
    async fn build_messages(&self, context: &ResponseContext) -> Result<Vec<AiMessage>> {
        let mut messages = Vec::new();

        // Add relevant memories as context
        for memory in &context.relevant_memories {
            messages.push(AiMessage {
                role: "assistant".to_string(),
                content: format!("[Memory: {}]", memory.memory.content),
                tool_calls: None,
            });
        }

        // Add current message
        messages.push(AiMessage {
            role: "user".to_string(),
            content: context.message.text_content(),
            tool_calls: None,
        });

        Ok(messages)
    }

    /// Build system prompt
    fn build_system_prompt(&self, context: &ResponseContext) -> Result<String> {
        let emotional_state = context
            .emotional_state
            .current_emotions()
            .into_iter()
            .map(|(emotion, intensity)| format!("{}: {:.2}", emotion, intensity))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!(
            "You are an emotional AI agent with the following state:\n\
             Emotions: {}\n\
             Decision: {}\n\
             Available tools: {}\n\n\
             Respond appropriately considering the emotional context and decision made.",
            emotional_state,
            context.decision.selected_option().description,
            context.available_tools.join(", ")
        ))
    }

    /// Build tool definitions
    fn build_tool_definitions(&self, tools: &[String]) -> Result<Vec<ai_sdk_client::Tool>> {
        tools
            .iter()
            .map(|tool_name| {
                Ok(ai_sdk_client::Tool {
                    r#type: "function".to_string(),
                    function: ai_sdk_client::ToolFunction {
                        name: tool_name.clone(),
                        description: Some(format!("Execute {} tool", tool_name)),
                        parameters: Some(serde_json::json!({
                            "type": "object",
                            "properties": {}
                        })),
                    },
                })
            })
            .collect()
    }

    /// Parse AI response
    async fn parse_ai_response(
        &self,
        response: ai_sdk_client::GenerateResponse,
    ) -> Result<AiResponse> {
        let text = response.choices.first().and_then(|choice| {
            if let ai_sdk_client::MessageContent::Text(text) = &choice.message.content {
                Some(text.clone())
            } else {
                None
            }
        });

        let tool_calls = response
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.as_ref())
            .map(|calls| {
                calls
                    .iter()
                    .map(|call| ToolCall {
                        tool_name: call.function.name.clone(),
                        arguments: call.function.arguments.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract emotional context from response
        let emotional_context =
            self.extract_emotional_context(&text.clone().unwrap_or_default())?;

        Ok(AiResponse {
            text,
            tool_calls,
            emotional_context,
        })
    }

    /// Extract emotional context from text
    fn extract_emotional_context(&self, text: &str) -> Result<HashMap<String, f32>> {
        let mut emotions = HashMap::new();

        // Simple keyword-based emotion detection
        let emotion_keywords = [
            ("happy", vec!["happy", "joy", "excited", "glad"]),
            ("sad", vec!["sad", "sorry", "unfortunate", "regret"]),
            ("angry", vec!["angry", "frustrated", "annoyed"]),
            ("fear", vec!["afraid", "worried", "concerned", "anxious"]),
        ];

        let text_lower = text.to_lowercase();

        for (emotion, keywords) in &emotion_keywords {
            let count = keywords
                .iter()
                .filter(|keyword| text_lower.contains(*keyword))
                .count();

            if count > 0 {
                emotions.insert(emotion.to_string(), (count as f32 * 0.2).min(1.0));
            }
        }

        Ok(emotions)
    }
}

/// MCP server integration
pub struct McpIntegration {
    /// Configuration
    config: McpConfig,

    /// MCP manager
    mcp_manager: Arc<mcp_manager::McpManager>,

    /// Available tools cache
    tools_cache: Arc<RwLock<HashMap<String, ToolInfo>>>,

    /// Execution stats
    stats: Arc<RwLock<ExecutionStats>>,
}

/// Tool information
#[derive(Debug, Clone)]
struct ToolInfo {
    /// Tool name
    name: String,

    /// Tool description
    description: String,

    /// Server providing this tool
    server: String,

    /// Input schema
    input_schema: serde_json::Value,

    /// Last execution time
    last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Execution statistics
#[derive(Debug, Default)]
struct ExecutionStats {
    /// Total executions
    total_executions: u64,

    /// Successful executions
    successful_executions: u64,

    /// Failed executions
    failed_executions: u64,

    /// Average execution time (ms)
    avg_execution_time: f64,
}

impl McpIntegration {
    /// Create new MCP integration
    pub async fn new(config: McpConfig) -> Result<Self> {
        let mut mcp_config = mcp_manager::Config::default();

        // Configure MCP servers
        for server in &config.servers {
            mcp_config.servers.push(mcp_manager::ServerConfig {
                name: server.name.clone(),
                transport: mcp_manager::TransportConfig::Http {
                    url: server.url.clone(),
                    headers: if let Some(auth) = &server.auth {
                        let mut headers = HashMap::new();
                        match auth.auth_type.as_str() {
                            "bearer" => {
                                headers.insert(
                                    "Authorization".to_string(),
                                    format!("Bearer {}", auth.credentials),
                                );
                            }
                            "basic" => {
                                headers.insert(
                                    "Authorization".to_string(),
                                    format!("Basic {}", auth.credentials),
                                );
                            }
                            _ => {}
                        }
                        headers
                    } else {
                        HashMap::new()
                    },
                },
                enabled_tools: server.enabled_tools.clone(),
            });
        }

        let mcp_manager = Arc::new(mcp_manager::McpManager::new(mcp_config).await?);

        // Initialize MCP servers
        mcp_manager.initialize_all().await?;

        let integration = Self {
            config,
            mcp_manager,
            tools_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
        };

        // Discover tools
        if integration.config.tool_discovery.auto_discover {
            integration.discover_tools().await?;
        }

        Ok(integration)
    }

    /// List available tools
    pub async fn list_available_tools(&self) -> Result<Vec<String>> {
        let cache = self.tools_cache.read().await;
        Ok(cache.keys().cloned().collect())
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();

        // Check execution limits
        self.check_execution_limits().await?;

        // Get tool info
        let tool_info = {
            let cache = self.tools_cache.read().await;
            cache
                .get(tool_name)
                .cloned()
                .ok_or_else(|| format!("Tool not found: {}", tool_name))?
        };

        // Execute through MCP manager
        let result = self
            .mcp_manager
            .execute_tool(&tool_info.server, tool_name, arguments.clone())
            .await;

        // Update stats
        let elapsed = start_time.elapsed().as_millis() as f64;
        self.update_stats(result.is_ok(), elapsed).await?;

        // Handle result
        match result {
            Ok(value) => Ok(ToolResult {
                call_id: uuid::Uuid::new_v4().to_string(),
                result: value,
                is_error: false,
            }),
            Err(e) => Ok(ToolResult {
                call_id: uuid::Uuid::new_v4().to_string(),
                result: serde_json::json!({
                    "error": e.to_string()
                }),
                is_error: true,
            }),
        }
    }

    /// Discover available tools
    async fn discover_tools(&self) -> Result<()> {
        let tools = self.mcp_manager.list_all_tools().await?;
        let mut cache = self.tools_cache.write().await;

        for (server_name, server_tools) in tools {
            for tool in server_tools {
                let tool_info = ToolInfo {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    server: server_name.clone(),
                    input_schema: tool.input_schema.clone(),
                    last_execution: None,
                };

                cache.insert(tool.name.clone(), tool_info);
            }
        }

        Ok(())
    }

    /// Check execution limits
    async fn check_execution_limits(&self) -> Result<()> {
        // Check concurrent executions
        let current_executions = self.mcp_manager.active_executions().await?;

        if current_executions >= self.config.execution_limits.max_concurrent {
            return Err("Maximum concurrent executions reached".into());
        }

        Ok(())
    }

    /// Update execution statistics
    async fn update_stats(&self, success: bool, execution_time: f64) -> Result<()> {
        let mut stats = self.stats.write().await;

        stats.total_executions += 1;
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        // Update average execution time
        let n = stats.total_executions as f64;
        stats.avg_execution_time = (stats.avg_execution_time * (n - 1.0) + execution_time) / n;

        Ok(())
    }

    /// Shutdown MCP integration
    pub async fn shutdown(&self) -> Result<()> {
        self.mcp_manager.shutdown_all().await?;
        Ok(())
    }
}

// Extension trait is now in utils module
