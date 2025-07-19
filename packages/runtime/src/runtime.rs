//! Core runtime implementation with AI SDK 5 and MCP integration

// async_trait is used implicitly by emotional_agents_core
use emotional_agents_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::decision::EmotionalDecisionEngine;
use crate::integration::{AiSdkIntegration, McpIntegration};
use crate::memory::KnowledgeGraphMemory;
use crate::relationships::RelationshipManager;
use crate::utils::EmotionalStateExt;

/// Advanced runtime for emotional agents
pub struct EmotionalAgentsRuntime {
    /// Plugin manager for modular functionality
    plugin_manager: PluginManager,

    /// Active agents
    agents: Arc<RwLock<HashMap<AgentId, AgentInstance>>>,

    /// AI SDK 5 integration
    ai_sdk: Arc<AiSdkIntegration>,

    /// MCP server integration
    mcp: Arc<McpIntegration>,

    /// Knowledge graph memory system
    memory: Arc<KnowledgeGraphMemory>,

    /// Relationship manager
    relationships: Arc<RelationshipManager>,

    /// Emotional decision engine
    decision_engine: Arc<EmotionalDecisionEngine>,

    /// Event bus for system-wide events
    event_bus: Arc<RwLock<event::EventBus>>,

    /// Runtime configuration
    config: RuntimeConfig,
}

/// Instance of a running agent
pub struct AgentInstance {
    /// The agent itself
    agent: Box<dyn Agent>,

    /// Agent-specific memory context
    memory_context: Uuid,

    /// Active relationships
    relationships: HashMap<String, Uuid>,

    /// Current emotional context
    emotional_context: EmotionalState,

    /// Decision history
    decision_history: Vec<DecisionRecord>,
}

/// Configuration for the runtime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    /// AI SDK configuration
    pub ai_sdk: AiSdkConfig,

    /// MCP configuration
    pub mcp: McpConfig,

    /// Memory system configuration
    pub memory: MemoryConfig,

    /// Relationship configuration
    pub relationships: RelationshipConfig,

    /// Decision engine configuration
    pub decision: DecisionConfig,

    /// Runtime behavior settings
    pub runtime: RuntimeSettings,
}

/// AI SDK configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSdkConfig {
    /// Model provider (openai, anthropic, etc.)
    pub provider: String,

    /// Model name
    pub model: String,

    /// API key (can be from env)
    pub api_key: Option<String>,

    /// Temperature for generation
    pub temperature: f32,

    /// Max tokens
    pub max_tokens: u32,

    /// Streaming enabled
    pub streaming: bool,
}

/// MCP configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpConfig {
    /// MCP server endpoints
    pub servers: Vec<McpServerConfig>,

    /// Tool discovery settings
    pub tool_discovery: ToolDiscoveryConfig,

    /// Execution limits
    pub execution_limits: ExecutionLimits,
}

/// Individual MCP server configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    /// Server name
    pub name: String,

    /// Server URL
    pub url: String,

    /// Authentication
    pub auth: Option<McpAuth>,

    /// Enabled tools
    pub enabled_tools: Option<Vec<String>>,
}

/// MCP authentication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpAuth {
    /// Auth type (bearer, basic, etc.)
    pub auth_type: String,

    /// Credentials
    pub credentials: String,
}

/// Tool discovery configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDiscoveryConfig {
    /// Auto-discover tools
    pub auto_discover: bool,

    /// Discovery interval (seconds)
    pub discovery_interval: u32,

    /// Tool categories to discover
    pub categories: Vec<String>,
}

/// Execution limits for MCP tools
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionLimits {
    /// Max concurrent executions
    pub max_concurrent: u32,

    /// Timeout per execution (ms)
    pub timeout_ms: u32,

    /// Max retries
    pub max_retries: u32,
}

/// Memory system configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryConfig {
    /// Storage backend
    pub storage: StorageConfig,

    /// Knowledge graph settings
    pub knowledge_graph: KnowledgeGraphConfig,

    /// Memory consolidation
    pub consolidation: ConsolidationConfig,
}

/// Storage configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageConfig {
    /// Storage type (postgres, redis, memory)
    pub storage_type: String,

    /// Connection string
    pub connection_string: Option<String>,

    /// Cache settings
    pub cache: CacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache TTL (seconds)
    pub ttl_seconds: u32,

    /// Max cache size (MB)
    pub max_size_mb: u32,
}

/// Knowledge graph configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeGraphConfig {
    /// Max nodes
    pub max_nodes: u32,

    /// Max edges per node
    pub max_edges_per_node: u32,

    /// Similarity threshold for connections
    pub similarity_threshold: f32,

    /// Enable automatic relationship inference
    pub auto_infer_relationships: bool,
}

/// Memory consolidation configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsolidationConfig {
    /// Enable consolidation
    pub enabled: bool,

    /// Consolidation interval (hours)
    pub interval_hours: u32,

    /// Importance threshold
    pub importance_threshold: f32,
}

/// Relationship configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelationshipConfig {
    /// Standing calculation method
    pub standing_method: StandingMethod,

    /// Relationship decay settings
    pub decay: RelationshipDecayConfig,

    /// Trust building settings
    pub trust: TrustConfig,
}

/// How to calculate relationship standing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StandingMethod {
    /// Simple numeric scale
    Numeric { min: f32, max: f32 },

    /// Category-based (Good, Neutral, Bad)
    Categorical,

    /// Multi-dimensional
    MultiDimensional { dimensions: Vec<String> },
}

/// Relationship decay configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelationshipDecayConfig {
    /// Enable decay
    pub enabled: bool,

    /// Decay rate per day
    pub daily_decay_rate: f32,

    /// Minimum standing (won't decay below)
    pub minimum_standing: f32,
}

/// Trust configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrustConfig {
    /// Initial trust level
    pub initial_trust: f32,

    /// Trust gain rate
    pub gain_rate: f32,

    /// Trust loss multiplier
    pub loss_multiplier: f32,
}

/// Decision engine configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionConfig {
    /// Emotion weight in decisions
    pub emotion_weight: f32,

    /// Logic weight in decisions
    pub logic_weight: f32,

    /// Memory weight in decisions
    pub memory_weight: f32,

    /// Relationship weight in decisions
    pub relationship_weight: f32,

    /// Decision strategies
    pub strategies: Vec<DecisionStrategy>,
}

/// Decision strategy configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionStrategy {
    /// Strategy name
    pub name: String,

    /// When to apply this strategy
    pub conditions: HashMap<String, serde_json::Value>,

    /// Strategy parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Runtime behavior settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeSettings {
    /// Enable debug mode
    pub debug: bool,

    /// Performance monitoring
    pub monitoring: MonitoringConfig,

    /// Error handling
    pub error_handling: ErrorHandlingConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics export interval (seconds)
    pub export_interval: u32,

    /// Metrics backend
    pub backend: String,
}

/// Error handling configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorHandlingConfig {
    /// Retry failed operations
    pub retry_enabled: bool,

    /// Max retry attempts
    pub max_retries: u32,

    /// Error recovery strategies
    pub recovery_strategies: Vec<String>,
}

/// Decision record for history
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionRecord {
    /// Decision ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Decision type
    pub decision_type: String,

    /// Options considered
    pub options: Vec<DecisionOption>,

    /// Selected option
    pub selected: usize,

    /// Emotional state at decision time
    pub emotional_state: EmotionalState,

    /// Factors that influenced the decision
    pub factors: DecisionFactors,
}

/// Option in a decision
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionOption {
    /// Option description
    pub description: String,

    /// Calculated score
    pub score: f32,

    /// Breakdown of scoring
    pub score_breakdown: HashMap<String, f32>,
}

/// Factors influencing a decision
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionFactors {
    /// Emotional factors
    pub emotional: HashMap<String, f32>,

    /// Logical factors
    pub logical: HashMap<String, f32>,

    /// Memory-based factors
    pub memory: HashMap<String, f32>,

    /// Relationship factors
    pub relationship: HashMap<String, f32>,
}

impl EmotionalAgentsRuntime {
    /// Create a new runtime with configuration
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        // Initialize components
        let plugin_manager = PluginManager::new();
        let agents = Arc::new(RwLock::new(HashMap::new()));

        // Initialize AI SDK integration
        let ai_sdk = Arc::new(AiSdkIntegration::new(config.ai_sdk.clone()).await?);

        // Initialize MCP integration
        let mcp = Arc::new(McpIntegration::new(config.mcp.clone()).await?);

        // Initialize memory system
        let memory = Arc::new(KnowledgeGraphMemory::new(config.memory.clone()).await?);

        // Initialize relationship manager
        let relationships = Arc::new(RelationshipManager::new(config.relationships.clone()).await?);

        // Initialize decision engine
        let decision_engine = Arc::new(
            EmotionalDecisionEngine::new(
                config.decision.clone(),
                memory.clone(),
                relationships.clone(),
            )
            .await?,
        );

        // Initialize event bus
        let event_bus = Arc::new(RwLock::new(event::EventBus::new()));

        Ok(Self {
            plugin_manager,
            agents,
            ai_sdk,
            mcp,
            memory,
            relationships,
            decision_engine,
            event_bus,
            config,
        })
    }

    /// Register a plugin
    pub async fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        self.plugin_manager.register(plugin).await
    }

    /// Create a new agent
    pub async fn create_agent(&self, config: AgentConfig) -> Result<AgentId> {
        let agent = self.plugin_manager.create_agent(config.clone()).await?;
        let agent_id = agent.id();

        // Create memory context for agent
        let memory_context = self.memory.create_context(agent_id).await?;

        // Initialize agent instance
        let instance = AgentInstance {
            agent,
            memory_context,
            relationships: HashMap::new(),
            emotional_context: config.initial_emotions.unwrap_or_default(),
            decision_history: Vec::new(),
        };

        // Store agent
        self.agents.write().await.insert(agent_id, instance);

        // Emit agent created event
        let event = event::EventBuilder::new(event::EventType::AgentCreated, agent_id).build();
        self.event_bus
            .write()
            .await
            .publish(event, &Context::new(agent_id, "system".to_string()))
            .await?;

        Ok(agent_id)
    }

    /// Process a message for an agent with full integration
    pub async fn process_message(
        &self,
        agent_id: &AgentId,
        message: Message,
    ) -> Result<Vec<Message>> {
        let mut agents = self.agents.write().await;
        let instance = agents.get_mut(agent_id).ok_or("Agent not found")?;

        // Update emotional state based on message
        let emotional_impact = self.analyze_emotional_impact(&message).await?;
        instance.emotional_context.apply_event(emotional_impact);

        // Check relationships
        if let Some(user_id) = self.extract_user_id(&message) {
            let relationship = self
                .relationships
                .get_or_create(agent_id.to_string(), user_id.clone())
                .await?;

            // Update relationship based on interaction
            self.relationships
                .update_from_interaction(&relationship.id, &message, &instance.emotional_context)
                .await?;
        }

        // Store message in memory
        self.memory
            .store_message(instance.memory_context, message.clone())
            .await?;

        // Get relevant memories
        let relevant_memories = self
            .memory
            .retrieve_relevant(instance.memory_context, &message, 10)
            .await?;

        // Make decision using emotional decision engine
        let decision = self
            .decision_engine
            .make_decision(
                &instance.emotional_context,
                &message,
                &relevant_memories,
                &instance.relationships,
            )
            .await?;

        // Record decision
        instance.decision_history.push(decision.clone());

        // Use AI SDK to generate response
        let response_context = self
            .build_response_context(
                &message,
                &relevant_memories,
                &decision,
                &instance.emotional_context,
            )
            .await?;

        let ai_response = self
            .ai_sdk
            .generate_response(response_context, &self.config.ai_sdk)
            .await?;

        // Execute any MCP tools if needed
        let tool_results = self.execute_tools_if_needed(&ai_response).await?;

        // Process through agent
        let mut final_responses = instance.agent.process_message(message.clone()).await?;

        // Enhance responses with AI-generated content
        if let Some(primary_response) = final_responses.get_mut(0) {
            self.enhance_response_with_ai(primary_response, ai_response, tool_results)?;
        }

        // Update knowledge graph
        self.memory
            .update_knowledge_graph(instance.memory_context, &message, &final_responses)
            .await?;

        Ok(final_responses)
    }

    /// Analyze emotional impact of a message
    async fn analyze_emotional_impact(&self, message: &Message) -> Result<EmotionalEvent> {
        // Use emotion engine to analyze message
        // This is a simplified version - real implementation would be more sophisticated
        let sentiment = if message.text_content().contains("good")
            || message.text_content().contains("happy")
        {
            0.7
        } else if message.text_content().contains("bad") || message.text_content().contains("sad") {
            -0.7
        } else {
            0.0
        };

        Ok(EmotionalEvent::UserFeedback {
            sentiment,
            specificity: 0.5,
            is_constructive: true,
        })
    }

    /// Extract user ID from message
    fn extract_user_id(&self, message: &Message) -> Option<String> {
        message
            .metadata
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| message.name.clone())
    }

    /// Build context for AI response generation
    async fn build_response_context(
        &self,
        message: &Message,
        memories: &[memory::MemoryResult],
        decision: &DecisionRecord,
        emotional_state: &EmotionalState,
    ) -> Result<ResponseContext> {
        Ok(ResponseContext {
            message: message.clone(),
            relevant_memories: memories.to_vec(),
            decision: decision.clone(),
            emotional_state: emotional_state.clone(),
            available_tools: self.mcp.list_available_tools().await?,
        })
    }

    /// Execute MCP tools if needed
    async fn execute_tools_if_needed(&self, ai_response: &AiResponse) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        for tool_call in &ai_response.tool_calls {
            let result = self
                .mcp
                .execute_tool(&tool_call.tool_name, &tool_call.arguments)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Enhance response with AI-generated content
    fn enhance_response_with_ai(
        &self,
        response: &mut Message,
        ai_response: AiResponse,
        tool_results: Vec<ToolResult>,
    ) -> Result<()> {
        // Add AI-generated content to response
        if let Some(text) = ai_response.text {
            response.content.push(MessageContent::Text { text });
        }

        // Add tool results
        for result in tool_results {
            response.content.push(MessageContent::ToolResult {
                call_id: result.call_id,
                result: result.result,
                is_error: result.is_error,
            });
        }

        // Add emotional context
        if !ai_response.emotional_context.is_empty() {
            response.content.push(MessageContent::Emotion {
                emotions: ai_response.emotional_context,
                context: "AI analysis".to_string(),
            });
        }

        Ok(())
    }

    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<AgentInfo>> {
        let agents = self.agents.read().await;

        if let Some(instance) = agents.get(agent_id) {
            Ok(Some(AgentInfo {
                id: *agent_id,
                character: instance.agent.character().clone(),
                emotional_state: instance.emotional_context.clone(),
                relationship_count: instance.relationships.len(),
                memory_entries: self.memory.count_entries(instance.memory_context).await?,
                decision_count: instance.decision_history.len(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Shutdown runtime
    pub async fn shutdown(&mut self) -> Result<()> {
        // Shutdown all agents
        let mut agents = self.agents.write().await;
        for (_, instance) in agents.drain() {
            if let Ok(state) = instance.agent.save_state().await {
                // Save agent state
                self.memory
                    .save_agent_state(instance.memory_context, state)
                    .await?;
            }
        }

        // Shutdown components
        self.mcp.shutdown().await?;
        self.memory.shutdown().await?;
        self.relationships.shutdown().await?;

        Ok(())
    }
}

/// Response context for AI generation
#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub message: Message,
    pub relevant_memories: Vec<memory::MemoryResult>,
    pub decision: DecisionRecord,
    pub emotional_state: EmotionalState,
    pub available_tools: Vec<String>,
}

/// AI response structure
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub emotional_context: HashMap<String, f32>,
}

/// Tool call structure
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub call_id: String,
    pub result: serde_json::Value,
    pub is_error: bool,
}

/// Agent information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub character: CharacterSheet,
    pub emotional_state: EmotionalState,
    pub relationship_count: usize,
    pub memory_entries: usize,
    pub decision_count: usize,
}

/// Default runtime configuration
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            ai_sdk: AiSdkConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: None,
                temperature: 0.7,
                max_tokens: 2000,
                streaming: false,
            },
            mcp: McpConfig {
                servers: vec![],
                tool_discovery: ToolDiscoveryConfig {
                    auto_discover: true,
                    discovery_interval: 300,
                    categories: vec!["general".to_string()],
                },
                execution_limits: ExecutionLimits {
                    max_concurrent: 5,
                    timeout_ms: 30000,
                    max_retries: 3,
                },
            },
            memory: MemoryConfig {
                storage: StorageConfig {
                    storage_type: "memory".to_string(),
                    connection_string: None,
                    cache: CacheConfig {
                        enabled: true,
                        ttl_seconds: 3600,
                        max_size_mb: 100,
                    },
                },
                knowledge_graph: KnowledgeGraphConfig {
                    max_nodes: 10000,
                    max_edges_per_node: 50,
                    similarity_threshold: 0.7,
                    auto_infer_relationships: true,
                },
                consolidation: ConsolidationConfig {
                    enabled: true,
                    interval_hours: 24,
                    importance_threshold: 0.3,
                },
            },
            relationships: RelationshipConfig {
                standing_method: StandingMethod::Categorical,
                decay: RelationshipDecayConfig {
                    enabled: true,
                    daily_decay_rate: 0.01,
                    minimum_standing: -0.5,
                },
                trust: TrustConfig {
                    initial_trust: 0.5,
                    gain_rate: 0.1,
                    loss_multiplier: 2.0,
                },
            },
            decision: DecisionConfig {
                emotion_weight: 0.3,
                logic_weight: 0.3,
                memory_weight: 0.2,
                relationship_weight: 0.2,
                strategies: vec![],
            },
            runtime: RuntimeSettings {
                debug: false,
                monitoring: MonitoringConfig {
                    enabled: true,
                    export_interval: 60,
                    backend: "prometheus".to_string(),
                },
                error_handling: ErrorHandlingConfig {
                    retry_enabled: true,
                    max_retries: 3,
                    recovery_strategies: vec!["exponential_backoff".to_string()],
                },
            },
        }
    }
}
