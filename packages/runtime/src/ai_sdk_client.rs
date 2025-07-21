//! AI SDK client module with AI SDK 5 transport-based configuration
//! Implements July 2025 AI SDK 5 beta architecture with MCP integration

use chrono;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::key_manager::{KeyManager, get_api_key};
use crate::http_pool::{ConnectionPool, PoolConfigBuilder};


/// AI SDK client for LLM integration
pub struct AiSdkClient {
    client: Client,
    config: Config,
}

/// Configuration for AI SDK client
#[derive(Debug, Clone)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub api_key_id: Option<String>, // ID from key manager
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub streaming: bool,
    pub base_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: String::new(),
            api_key_id: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            streaming: false,
            base_url: None,
        }
    }
}

/// Generate request
#[derive(Debug, Clone, Serialize, Default)]
pub struct GenerateRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub tools: Option<Vec<Tool>>,
    pub stream: Option<bool>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

/// Image URL content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: String,
    pub function: ToolFunction,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    pub r#type: String,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Generate response
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Response choice
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// Anthropic specific request format
#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: i32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic response format
#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicContent {
    r#type: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// === AI SDK 5 Transport-Based Configuration ===

/// AI SDK 5 transport configuration (July 2025)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportConfig {
    /// Server-Sent Events transport (recommended for production)
    SSE {
        url: String,
        headers: HashMap<String, String>,
        timeout_seconds: u64,
    },
    /// HTTP transport for request/response
    HTTP {
        endpoint: String,
        headers: HashMap<String, String>,
        timeout_seconds: u64,
    },
    /// Standard input/output for local processes
    Stdio {
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    },
}

/// AI SDK 5 enhanced client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSdk5Config {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub api_key_id: Option<String>, // ID from key manager
    pub transport: TransportConfig,
    pub streaming_enabled: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

/// MCP client configuration for AI SDK 5 integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    pub name: String,
    pub transport: TransportConfig,
    pub enabled: bool,
    pub capabilities: Vec<String>,
    pub priority: u32,
    pub health_check_interval_seconds: u64,
}

/// MCP tool representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub server: String,
}

/// AI SDK 5 enhanced client with transport support
pub struct AiSdk5Client {
    legacy_client: AiSdkClient,
    config: AiSdk5Config,
    client_id: Uuid,
    transport: Arc<RwLock<Option<Box<dyn transport::Transport>>>>,
    streaming_active: Arc<RwLock<bool>>,
}


/// MCP client wrapper for AI SDK 5 integration
pub struct McpClient {
    config: McpClientConfig,
    client_id: Uuid,
    transport: Arc<RwLock<Option<transport::McpTransport>>>,
    tools: Arc<RwLock<Vec<McpTool>>>,
    connected: Arc<RwLock<bool>>,
}


/// AI SDK 5 client manager
pub struct AiSdk5ClientManager {
    clients: HashMap<String, Arc<AiSdk5Client>>,
    mcp_clients: HashMap<String, Arc<McpClient>>,
    default_client: Option<String>,
}

impl AiSdk5Client {
    /// Create new AI SDK 5 client with transport configuration
    pub async fn new(config: AiSdk5Config) -> crate::Result<Self> {
        // Create legacy client for compatibility
        let legacy_config = Config {
            provider: config.provider.clone(),
            model: config.model.clone(),
            api_key: config.api_key.clone().unwrap_or_default(),
            api_key_id: config.api_key_id.clone(),
            temperature: config.temperature.map(|t| t as f32),
            max_tokens: config.max_tokens,
            streaming: config.streaming_enabled,
            base_url: match &config.transport {
                TransportConfig::HTTP { endpoint, .. } => Some(endpoint.clone()),
                _ => None,
            },
        };

        let legacy_client = AiSdkClient::new(legacy_config).await?;

        Ok(Self {
            legacy_client,
            client_id: Uuid::new_v4(),
            config,
            transport: Arc::new(RwLock::new(None)),
            streaming_active: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Create new AI SDK 5 client with secure key retrieval
    pub async fn new_with_key_manager(
        mut config: AiSdk5Config,
        key_manager: &KeyManager,
    ) -> crate::Result<Self> {
        // Create legacy config with key manager
        let legacy_config = Config {
            provider: config.provider.clone(),
            model: config.model.clone(),
            api_key: config.api_key.clone().unwrap_or_default(),
            api_key_id: config.api_key_id.clone(),
            temperature: config.temperature.map(|t| t as f32),
            max_tokens: config.max_tokens,
            streaming: config.streaming_enabled,
            base_url: match &config.transport {
                TransportConfig::HTTP { endpoint, .. } => Some(endpoint.clone()),
                _ => None,
            },
        };

        let legacy_client = AiSdkClient::new_with_key_manager(legacy_config, key_manager).await?;
        
        // Update config with retrieved key
        config.api_key = Some(legacy_client.config.api_key.clone());

        Ok(Self {
            legacy_client,
            client_id: Uuid::new_v4(),
            config,
            transport: Arc::new(RwLock::new(None)),
            streaming_active: Arc::new(RwLock::new(false)),
        })
    }

    /// Initialize the client and establish transport connection
    pub async fn initialize(&self) -> crate::Result<()> {
        info!(
            "Initializing AI SDK 5 client {} with transport",
            self.client_id
        );

        // Create transport based on configuration
        let transport = transport::create_transport(&self.config.transport).await?;
        
        *self.transport.write().await = Some(transport);
        info!(
            "AI SDK 5 client {} initialized successfully",
            self.client_id
        );
        Ok(())
    }

    /// Send message with AI SDK 5 transport (fallback to legacy for now)
    pub async fn generate_with_transport(
        &self,
        request: GenerateRequest,
        emotional_context: Option<&serde_json::Value>,
    ) -> crate::Result<GenerateResponse> {
        let transport_guard = self.transport.read().await;
        if let Some(transport) = transport_guard.as_ref() {
            debug!(
                "Using {} transport for AI SDK 5 client {}",
                transport.transport_type(), self.client_id
            );
        }

        // For now, delegate to legacy client
        // In real implementation, this would use AI SDK 5 transport
        let response = self.legacy_client.generate(request).await?;

        // Add AI SDK 5 metadata
        if let Some(context) = emotional_context {
            // In real implementation, this would be handled by AI SDK 5
            debug!("AI SDK 5 emotional context: {}", context);
        }

        Ok(response)
    }

    /// Start streaming response with AI SDK 5
    pub async fn stream_with_transport(
        &self,
        _request: GenerateRequest,
    ) -> crate::Result<tokio::sync::mpsc::Receiver<String>> {
        if !self.config.streaming_enabled {
            return Err(crate::error::RuntimeError::InvalidState("Streaming not enabled for this client".to_string()));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        *self.streaming_active.write().await = true;

        let streaming_active = self.streaming_active.clone();
        let client_id = self.client_id;

        tokio::spawn(async move {
            // Mock streaming response for AI SDK 5
            let chunks = vec!["AI SDK 5 ", "streaming ", "response"];

            for chunk in chunks {
                if !*streaming_active.read().await {
                    break;
                }

                if tx.send(chunk.to_string()).await.is_err() {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            *streaming_active.write().await = false;
            debug!("AI SDK 5 streaming completed for client {}", client_id);
        });

        Ok(rx)
    }

    /// Health check for AI SDK 5 client
    pub async fn health_check(&self) -> crate::Result<bool> {
        let transport_guard = self.transport.read().await;
        if let Some(transport) = transport_guard.as_ref() {
            let now = chrono::Utc::now();
            let last_activity = transport.last_activity().await;
            let time_since_activity = now - last_activity;
            Ok(transport.is_connected().await && time_since_activity.num_seconds() < 300)
        } else {
            Ok(false)
        }
    }

    /// Shutdown the AI SDK 5 client
    pub async fn shutdown(&self) -> crate::Result<()> {
        info!("Shutting down AI SDK 5 client {}", self.client_id);
        *self.streaming_active.write().await = false;
        *self.transport.write().await = None;
        Ok(())
    }
}

impl McpClient {
    /// Create new MCP client for AI SDK 5 integration
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            client_id: Uuid::new_v4(),
            config,
            transport: Arc::new(RwLock::new(None)),
            tools: Arc::new(RwLock::new(Vec::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize MCP client with AI SDK 5 experimental_createMCPClient
    pub async fn initialize(&self) -> crate::Result<()> {
        info!(
            "Initializing MCP client {} for server {}",
            self.client_id, self.config.name
        );

        // Create base transport
        let base_transport = transport::create_transport(&self.config.transport).await?;
        
        // Wrap in MCP transport
        let mcp_transport = transport::McpTransport::new(base_transport, self.config.name.clone());
        
        // Initialize MCP handshake
        let init_response = mcp_transport.initialize().await?;
        debug!("MCP handshake response: {:?}", init_response);
        
        *self.transport.write().await = Some(mcp_transport);
        *self.connected.write().await = true;

        // Load available tools from MCP server
        self.load_tools().await?;

        info!(
            "MCP client {} initialized with {} tools",
            self.client_id,
            self.tools.read().await.len()
        );
        Ok(())
    }

    /// Load tools from MCP server and convert to AI SDK format
    async fn load_tools(&self) -> crate::Result<()> {
        let transport_guard = self.transport.read().await;
        if let Some(transport) = transport_guard.as_ref() {
            let tools_json = transport.list_tools().await?;
            
            let tools: Vec<McpTool> = tools_json
                .into_iter()
                .map(|tool| McpTool {
                    name: tool.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: tool.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    parameters: tool.get("inputSchema").cloned().unwrap_or(serde_json::json!({})),
                    server: self.config.name.clone(),
                })
                .collect();
            
            *self.tools.write().await = tools;
        }
        Ok(())
    }

    /// Get available tools
    pub async fn get_tools(&self) -> Vec<McpTool> {
        self.tools.read().await.clone()
    }

    /// Execute tool call on MCP server
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> crate::Result<serde_json::Value> {
        if !*self.connected.read().await {
            return Err(crate::error::RuntimeError::InvalidState("MCP client not connected".to_string()));
        }

        debug!(
            "Executing tool {} on MCP server {}",
            tool_name, self.config.name
        );

        let transport_guard = self.transport.read().await;
        if let Some(transport) = transport_guard.as_ref() {
            let result = transport.execute_tool(tool_name, arguments.clone()).await?;
            Ok(result)
        } else {
            Err(crate::error::RuntimeError::Transport("MCP transport not available".to_string()))
        }
    }

    /// Shutdown MCP client
    pub async fn shutdown(&self) -> crate::Result<()> {
        info!(
            "Shutting down MCP client {} ({})",
            self.client_id, self.config.name
        );
        *self.connected.write().await = false;
        *self.transport.write().await = None;
        Ok(())
    }
}

impl AiSdk5ClientManager {
    /// Create new AI SDK 5 client manager
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            mcp_clients: HashMap::new(),
            default_client: None,
        }
    }

    /// Add AI SDK 5 client
    pub async fn add_client(&mut self, name: String, config: AiSdk5Config) -> crate::Result<()> {
        let client = Arc::new(AiSdk5Client::new(config).await?);
        client.initialize().await?;

        if self.default_client.is_none() {
            self.default_client = Some(name.clone());
        }

        self.clients.insert(name.clone(), client);
        info!("Added AI SDK 5 client: {}", name);
        Ok(())
    }

    /// Add MCP client
    pub async fn add_mcp_client(
        &mut self,
        name: String,
        config: McpClientConfig,
    ) -> crate::Result<()> {
        let client = Arc::new(McpClient::new(config));
        client.initialize().await?;

        self.mcp_clients.insert(name.clone(), client);
        info!("Added MCP client: {}", name);
        Ok(())
    }

    /// Get AI SDK 5 client
    pub fn get_client(&self, name: &str) -> Option<Arc<AiSdk5Client>> {
        self.clients.get(name).cloned()
    }

    /// Get MCP client
    pub fn get_mcp_client(&self, name: &str) -> Option<Arc<McpClient>> {
        self.mcp_clients.get(name).cloned()
    }

    /// Get all available MCP tools
    pub async fn get_all_mcp_tools(&self) -> Vec<McpTool> {
        let mut all_tools = Vec::new();
        for client in self.mcp_clients.values() {
            let tools = client.get_tools().await;
            all_tools.extend(tools);
        }
        all_tools
    }

    /// Execute MCP tool by name
    pub async fn execute_mcp_tool(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> crate::Result<serde_json::Value> {
        for client in self.mcp_clients.values() {
            let tools = client.get_tools().await;
            if tools.iter().any(|t| t.name == tool_name) {
                return client.execute_tool(tool_name, arguments).await;
            }
        }
        Err(crate::error::RuntimeError::NotFound(format!("MCP tool '{}'", tool_name)))
    }

    /// Shutdown all clients
    pub async fn shutdown_all(&self) -> crate::Result<()> {
        info!("Shutting down all AI SDK 5 and MCP clients");

        for (name, client) in &self.clients {
            if let Err(e) = client.shutdown().await {
                error!("Failed to shutdown AI client {}: {}", name, e);
            }
        }

        for (name, client) in &self.mcp_clients {
            if let Err(e) = client.shutdown().await {
                error!("Failed to shutdown MCP client {}: {}", name, e);
            }
        }

        Ok(())
    }
}

impl AiSdkClient {
    /// Create new AI SDK client
    pub async fn new(config: Config) -> crate::Result<Self> {
        // Use connection pool for HTTP client
        let pool_config = PoolConfigBuilder::new()
            .timeout_secs(60)
            .enforce_tls(true)
            .build();
        let client = ConnectionPool::get_client(&pool_config);

        Ok(Self { client: (*client).clone(), config })
    }
    
    /// Create new AI SDK client with secure key retrieval
    pub async fn new_with_key_manager(
        mut config: Config,
        key_manager: &KeyManager,
    ) -> crate::Result<Self> {
        // If API key is not provided, try to retrieve it securely
        if config.api_key.is_empty() {
            if let Some(key_id) = &config.api_key_id {
                // Retrieve by specific key ID
                let secure_key = key_manager.retrieve(key_id).await
                    .map_err(|e| format!("Failed to retrieve API key: {}", e))?;
                config.api_key = String::from_utf8(secure_key.material().to_vec())
                    .map_err(|e| format!("Invalid API key format: {}", e))?;
            } else {
                // Try to get from environment or key manager by provider name
                let env_var = match config.provider.as_str() {
                    "openai" => "OPENAI_API_KEY",
                    "anthropic" => "ANTHROPIC_API_KEY",
                    "azure" => "AZURE_API_KEY",
                    _ => return Err(crate::error::RuntimeError::Config(format!("Unknown provider: {}", config.provider))),
                };
                
                config.api_key = get_api_key(key_manager, env_var, &config.provider).await
                    .map_err(|e| format!("Failed to get API key: {}", e))?;
            }
        }
        
        Self::new(config).await
    }

    /// Generate completion
    pub async fn generate(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        match self.config.provider.as_str() {
            "openai" => self.generate_openai(request).await,
            "anthropic" => self.generate_anthropic(request).await,
            "azure" => self.generate_azure(request).await,
            provider => Err(crate::error::RuntimeError::Config(format!("Unsupported provider: {}", provider))),
        }
    }

    /// Generate completion using OpenAI API
    async fn generate_openai(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        let url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com")
            .to_string()
            + "/v1/chat/completions";

        let mut req_body = serde_json::json!({
            "model": request.model.unwrap_or(self.config.model.clone()),
            "messages": request.messages,
        });

        if let Some(temp) = request.temperature.or(self.config.temperature) {
            req_body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request
            .max_tokens
            .or(self.config.max_tokens.map(|t| t as i32))
        {
            req_body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = request.tools {
            req_body["tools"] = serde_json::json!(tools);
        }

        if let Some(stream) = request.stream {
            req_body["stream"] = serde_json::json!(stream);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::RuntimeError::AiSdk(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        Ok(result)
    }

    /// Generate completion using Anthropic API
    async fn generate_anthropic(
        &self,
        request: GenerateRequest,
    ) -> crate::Result<GenerateResponse> {
        let url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
            .to_string()
            + "/v1/messages";

        // Convert messages to Anthropic format
        let anthropic_messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .map(|msg| AnthropicMessage {
                role: msg.role,
                content: match msg.content {
                    MessageContent::Text(text) => text,
                    MessageContent::Parts(parts) => parts
                        .into_iter()
                        .filter_map(|part| part.text)
                        .collect::<Vec<_>>()
                        .join(" "),
                },
            })
            .collect();

        let anthropic_request = AnthropicRequest {
            model: request.model.unwrap_or(self.config.model.clone()),
            max_tokens: request
                .max_tokens
                .or(self.config.max_tokens.map(|t| t as i32))
                .unwrap_or(2000),
            messages: anthropic_messages,
            temperature: request.temperature.or(self.config.temperature),
            tools: request.tools,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| format!("Anthropic API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::RuntimeError::AiSdk(format!("Anthropic API error {}: {}", status, error_text)));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        // Convert to OpenAI-compatible format
        let content_text = anthropic_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let result = GenerateResponse {
            id: anthropic_response.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: anthropic_response.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: anthropic_response.role,
                    content: MessageContent::Text(content_text),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: anthropic_response.stop_reason,
            }],
            usage: Some(Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            }),
        };

        Ok(result)
    }

    /// Generate completion using Azure OpenAI
    async fn generate_azure(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .ok_or("Azure base URL is required for Azure provider")?;

        // Azure URL format: https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version=2023-12-01-preview
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version=2023-12-01-preview",
            base_url, self.config.model
        );

        let mut req_body = serde_json::json!({
            "messages": request.messages,
        });

        if let Some(temp) = request.temperature.or(self.config.temperature) {
            req_body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request
            .max_tokens
            .or(self.config.max_tokens.map(|t| t as i32))
        {
            req_body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = request.tools {
            req_body["tools"] = serde_json::json!(tools);
        }

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Azure OpenAI API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::RuntimeError::AiSdk(format!("Azure OpenAI API error {}: {}", status, error_text)));
        }

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Azure OpenAI response: {}", e))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_request_serialization() {
        let message = Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let request = GenerateRequest {
            messages: vec![message],
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            tools: None,
            stream: None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("Hello"));
        assert!(serialized.contains("gpt-4"));
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: "test-key".to_string(),
            api_key_id: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            streaming: false,
            base_url: None,
        };

        let client = AiSdkClient::new(config).await;
        assert!(client.is_ok());
    }
}

/// Transport layer module for AI SDK 5 integration
pub mod transport {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use reqwest::Client;
    use std::collections::HashMap;
    use std::process::Stdio;
    use std::sync::Arc;
    use crate::http_pool::{ConnectionPool, PoolConfigBuilder};
    use tokio::process::{Child, Command};
    use tokio::sync::RwLock;
    use tracing::{debug, info, warn};
    use uuid::Uuid;

    use super::TransportConfig;

    /// Wrapper for reqwest errors to implement RetryableError
    struct HttpRetryError(reqwest::Error);
    
    impl std::fmt::Display for HttpRetryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    
    impl std::fmt::Debug for HttpRetryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    
    impl crate::retry::RetryableError for HttpRetryError {
        fn is_retryable(&self) -> bool {
            self.0.is_retryable()
        }
    }

    /// Transport trait for AI SDK 5 communication
    #[async_trait]
    pub trait Transport: Send + Sync {
        /// Get transport type identifier
        fn transport_type(&self) -> &str;

        /// Check if transport is connected
        async fn is_connected(&self) -> bool;

        /// Connect the transport
        async fn connect(&mut self) -> crate::Result<()>;

        /// Disconnect the transport
        async fn disconnect(&mut self) -> crate::Result<()>;

        /// Send a request and receive response
        async fn request(&self, payload: serde_json::Value) -> crate::Result<serde_json::Value>;

        /// Get last activity timestamp
        async fn last_activity(&self) -> DateTime<Utc>;
    }

    /// SSE (Server-Sent Events) transport implementation
    pub struct SseTransport {
        url: String,
        headers: HashMap<String, String>,
        timeout_seconds: u64,
        client: Client,
        connected: Arc<RwLock<bool>>,
        last_activity: Arc<RwLock<DateTime<Utc>>>,
    }

    impl SseTransport {
        pub fn new(url: String, headers: HashMap<String, String>, timeout_seconds: u64) -> Self {
            // Use connection pool for HTTP client
            let pool_config = PoolConfigBuilder::new()
                .timeout_secs(timeout_seconds)
                .enforce_tls(true)
                .build();
            let client = ConnectionPool::get_client(&pool_config);

            Self {
                url,
                headers,
                timeout_seconds,
                client: (*client).clone(),
                connected: Arc::new(RwLock::new(false)),
                last_activity: Arc::new(RwLock::new(Utc::now())),
            }
        }
    }

    #[async_trait]
    impl Transport for SseTransport {
        fn transport_type(&self) -> &str {
            "SSE"
        }

        async fn is_connected(&self) -> bool {
            *self.connected.read().await
        }

        async fn connect(&mut self) -> crate::Result<()> {
            info!("Connecting SSE transport to {}", self.url);
            
            // Test connection with a simple request
            let mut request = self.client.get(&self.url);
            for (key, value) in &self.headers {
                request = request.header(key, value);
            }
            
            // Use retry logic for SSE connection
            let request_clone = request.try_clone().ok_or_else(|| {
                crate::error::RuntimeError::Transport("Failed to clone request".to_string())
            })?;
            
            let response = crate::retry::retry_with_config(
                &crate::retry::RetryConfig::network(),
                "SSE transport connect",
                move || {
                    let req = request_clone.try_clone().unwrap();
                    Box::pin(async move {
                        req.send()
                            .await
                            .map_err(|e| HttpRetryError(e))
                    })
                },
            )
            .await
            .map_err(|e| crate::error::RuntimeError::Transport(format!("SSE connection failed: {}", e.0)))?;

            if response.status().is_success() {
                *self.connected.write().await = true;
                *self.last_activity.write().await = Utc::now();
                info!("SSE transport connected successfully");
                Ok(())
            } else {
                Err(crate::error::RuntimeError::Transport(format!("SSE connection failed with status: {}", response.status())))
            }
        }

        async fn disconnect(&mut self) -> crate::Result<()> {
            *self.connected.write().await = false;
            info!("SSE transport disconnected");
            Ok(())
        }

        async fn request(&self, payload: serde_json::Value) -> crate::Result<serde_json::Value> {
            if !self.is_connected().await {
                return Err("SSE transport not connected".into());
            }

            let mut request = self.client.post(&self.url);
            for (key, value) in &self.headers {
                request = request.header(key, value);
            }

            let response = request
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("SSE request failed: {}", e))?;

            *self.last_activity.write().await = Utc::now();

            if response.status().is_success() {
                let data = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse SSE response: {}", e))?;
                Ok(data)
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(crate::error::RuntimeError::Transport(format!("SSE request failed with status {}: {}", status, error_text)))
            }
        }

        async fn last_activity(&self) -> DateTime<Utc> {
            *self.last_activity.read().await
        }
    }

    /// HTTP transport implementation
    pub struct HttpTransport {
        endpoint: String,
        headers: HashMap<String, String>,
        timeout_seconds: u64,
        client: Client,
        connected: Arc<RwLock<bool>>,
        last_activity: Arc<RwLock<DateTime<Utc>>>,
    }

    impl HttpTransport {
        pub fn new(endpoint: String, headers: HashMap<String, String>, timeout_seconds: u64) -> Self {
            // Use connection pool for HTTP client
            let pool_config = PoolConfigBuilder::new()
                .timeout_secs(timeout_seconds)
                .enforce_tls(true)
                .build();
            let client = ConnectionPool::get_client(&pool_config);

            Self {
                endpoint,
                headers,
                timeout_seconds,
                client: (*client).clone(),
                connected: Arc::new(RwLock::new(false)),
                last_activity: Arc::new(RwLock::new(Utc::now())),
            }
        }
    }

    #[async_trait]
    impl Transport for HttpTransport {
        fn transport_type(&self) -> &str {
            "HTTP"
        }

        async fn is_connected(&self) -> bool {
            *self.connected.read().await
        }

        async fn connect(&mut self) -> crate::Result<()> {
            info!("Connecting HTTP transport to {}", self.endpoint);
            *self.connected.write().await = true;
            *self.last_activity.write().await = Utc::now();
            Ok(())
        }

        async fn disconnect(&mut self) -> crate::Result<()> {
            *self.connected.write().await = false;
            info!("HTTP transport disconnected");
            Ok(())
        }

        async fn request(&self, payload: serde_json::Value) -> crate::Result<serde_json::Value> {
            if !self.is_connected().await {
                return Err("HTTP transport not connected".into());
            }

            let mut request = self.client.post(&self.endpoint);
            for (key, value) in &self.headers {
                request = request.header(key, value);
            }

            // Use retry logic for network operations
            let request_clone = request.try_clone().ok_or_else(|| {
                crate::error::RuntimeError::Transport("Failed to clone request".to_string())
            })?;
            
            let payload_clone = payload.clone();
            let response = crate::retry::retry_with_config(
                &crate::retry::RetryConfig::network(),
                "HTTP transport request",
                move || {
                    let req = request_clone.try_clone().unwrap();
                    let payload = payload_clone.clone();
                    Box::pin(async move {
                        req.json(&payload)
                            .send()
                            .await
                            .map_err(|e| HttpRetryError(e))
                    })
                },
            )
            .await
            .map_err(|e| crate::error::RuntimeError::Transport(format!("HTTP request failed: {}", e.0)))?;

            *self.last_activity.write().await = Utc::now();

            if response.status().is_success() {
                let data = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse HTTP response: {}", e))?;
                Ok(data)
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(crate::error::RuntimeError::Transport(format!("HTTP request failed with status {}: {}", status, error_text)))
            }
        }

        async fn last_activity(&self) -> DateTime<Utc> {
            *self.last_activity.read().await
        }
    }

    /// Stdio transport implementation for local processes
    pub struct StdioTransport {
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
        process: Option<Child>,
        connected: Arc<RwLock<bool>>,
        last_activity: Arc<RwLock<DateTime<Utc>>>,
    }

    impl StdioTransport {
        pub fn new(command: String, args: Vec<String>, working_dir: Option<String>) -> Self {
            Self {
                command,
                args,
                working_dir,
                process: None,
                connected: Arc::new(RwLock::new(false)),
                last_activity: Arc::new(RwLock::new(Utc::now())),
            }
        }
    }

    #[async_trait]
    impl Transport for StdioTransport {
        fn transport_type(&self) -> &str {
            "Stdio"
        }

        async fn is_connected(&self) -> bool {
            *self.connected.read().await && self.process.is_some()
        }

        async fn connect(&mut self) -> crate::Result<()> {
            info!("Starting stdio transport process: {} {:?}", self.command, self.args);

            let mut cmd = Command::new(&self.command);
            cmd.args(&self.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            if let Some(ref dir) = self.working_dir {
                cmd.current_dir(dir);
            }

            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn process: {}", e))?;

            self.process = Some(child);
            *self.connected.write().await = true;
            *self.last_activity.write().await = Utc::now();

            info!("Stdio transport process started successfully");
            Ok(())
        }

        async fn disconnect(&mut self) -> crate::Result<()> {
            if let Some(mut process) = self.process.take() {
                debug!("Terminating stdio transport process");
                process.kill().await.ok();
            }
            *self.connected.write().await = false;
            info!("Stdio transport disconnected");
            Ok(())
        }

        async fn request(&self, _payload: serde_json::Value) -> crate::Result<serde_json::Value> {
            if !self.is_connected().await {
                return Err("Stdio transport not connected".into());
            }

            // Note: This is a simplified implementation. Real stdio transport would need
            // proper message framing and async I/O handling
            warn!("Stdio transport request handling not fully implemented");
            
            *self.last_activity.write().await = Utc::now();
            
            // Return mock response for now
            Ok(serde_json::json!({
                "status": "ok",
                "message": "Stdio transport placeholder response"
            }))
        }

        async fn last_activity(&self) -> DateTime<Utc> {
            *self.last_activity.read().await
        }
    }

    /// MCP (Model Context Protocol) transport wrapper
    pub struct McpTransport {
        inner: Box<dyn Transport>,
        server_name: String,
        protocol_version: String,
    }

    impl McpTransport {
        pub fn new(transport: Box<dyn Transport>, server_name: String) -> Self {
            Self {
                inner: transport,
                server_name,
                protocol_version: "1.0".to_string(),
            }
        }

        /// Initialize MCP handshake
        pub async fn initialize(&self) -> crate::Result<serde_json::Value> {
            let init_request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": self.protocol_version,
                    "clientInfo": {
                        "name": "banshee-runtime",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                },
                "id": Uuid::new_v4().to_string()
            });

            self.inner.request(init_request).await
        }

        /// List available tools
        pub async fn list_tools(&self) -> crate::Result<Vec<serde_json::Value>> {
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "tools/list",
                "params": {},
                "id": Uuid::new_v4().to_string()
            });

            let response = self.inner.request(request).await?;
            
            if let Some(tools) = response.get("result").and_then(|r| r.get("tools")) {
                Ok(tools.as_array().cloned().unwrap_or_default())
            } else {
                Ok(vec![])
            }
        }

        /// Execute a tool
        pub async fn execute_tool(
            &self,
            tool_name: &str,
            arguments: serde_json::Value,
        ) -> crate::Result<serde_json::Value> {
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": arguments
                },
                "id": Uuid::new_v4().to_string()
            });

            self.inner.request(request).await
        }
    }

    /// Create transport from configuration
    pub async fn create_transport(config: &TransportConfig) -> crate::Result<Box<dyn Transport>> {
        match config {
            TransportConfig::SSE { url, headers, timeout_seconds } => {
                let mut transport = SseTransport::new(url.clone(), headers.clone(), *timeout_seconds);
                transport.connect().await?;
                Ok(Box::new(transport))
            }
            TransportConfig::HTTP { endpoint, headers, timeout_seconds } => {
                let mut transport = HttpTransport::new(endpoint.clone(), headers.clone(), *timeout_seconds);
                transport.connect().await?;
                Ok(Box::new(transport))
            }
            TransportConfig::Stdio { command, args, working_dir } => {
                let mut transport = StdioTransport::new(
                    command.clone(),
                    args.clone(),
                    working_dir.clone(),
                );
                transport.connect().await?;
                Ok(Box::new(transport))
            }
        }
    }
}
