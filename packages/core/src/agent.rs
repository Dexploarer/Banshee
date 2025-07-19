use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CharacterSheet, Context, EmotionalState, Message, Result};

/// Unique identifier for agents
pub type AgentId = Uuid;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Optional agent ID (generates new if None)
    pub id: Option<AgentId>,

    /// Agent's character sheet defining personality and capabilities
    pub character: CharacterSheet,

    /// Initial emotional state
    pub initial_emotions: Option<EmotionalState>,

    /// Agent-specific settings
    pub settings: std::collections::HashMap<String, serde_json::Value>,

    /// Enabled plugin IDs
    pub enabled_plugins: Vec<String>,
}

/// Core agent trait that all agents must implement
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's unique identifier
    fn id(&self) -> AgentId;

    /// Get the agent's character sheet
    fn character(&self) -> &CharacterSheet;

    /// Get the agent's current emotional state
    fn emotional_state(&self) -> &EmotionalState;

    /// Process an incoming message and generate response(s)
    async fn process_message(&mut self, message: Message) -> Result<Vec<Message>>;

    /// Update the agent's context with new information
    async fn update_context(&mut self, context: Context) -> Result<()>;

    /// Get the agent's current context
    async fn get_context(&self) -> Result<Context>;

    /// Handle an emotional event and update state
    async fn process_emotion(&mut self, event: crate::EmotionalEvent) -> Result<()>;

    /// Save agent state (for persistence)
    async fn save_state(&self) -> Result<serde_json::Value>;

    /// Load agent state (from persistence)
    async fn load_state(&mut self, state: serde_json::Value) -> Result<()>;

    /// Check if agent is healthy and responsive
    async fn health_check(&self) -> Result<bool>;
}

/// Agent capabilities that can be dynamically enabled/disabled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentCapability {
    /// Can process and generate text
    TextProcessing,

    /// Can understand and express emotions
    EmotionalIntelligence,

    /// Can remember past conversations
    Memory,

    /// Can access external tools and APIs
    ToolUse,

    /// Can browse the web
    WebAccess,

    /// Can interact with blockchain networks
    Web3,

    /// Can process images
    ImageProcessing,

    /// Can generate images
    ImageGeneration,

    /// Can process audio
    AudioProcessing,

    /// Can generate speech
    SpeechGeneration,

    /// Can execute code
    CodeExecution,

    /// Can learn and adapt
    Learning,

    /// Custom capability
    Custom(String),
}

/// Agent status indicating current operational state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is initializing
    Initializing,

    /// Agent is ready to process messages
    Ready,

    /// Agent is currently processing a message
    Processing,

    /// Agent is in an error state
    Error(String),

    /// Agent is shutting down
    Shutting,

    /// Agent is offline
    Offline,
}

/// Runtime statistics for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    /// Total messages processed
    pub messages_processed: u64,

    /// Total tokens consumed
    pub tokens_consumed: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Number of emotional events processed
    pub emotional_events: u64,

    /// Current emotional intensity (0.0 to 1.0)
    pub emotional_intensity: f32,

    /// Memory utilization percentage
    pub memory_utilization: f32,

    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,

    /// Uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for AgentStats {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            tokens_consumed: 0,
            avg_response_time_ms: 0.0,
            emotional_events: 0,
            emotional_intensity: 0.0,
            memory_utilization: 0.0,
            last_activity: chrono::Utc::now(),
            uptime_seconds: 0,
        }
    }
}

/// Extended agent trait for runtime management
#[async_trait]
pub trait ManagedAgent: Agent {
    /// Get current agent status
    fn status(&self) -> AgentStatus;

    /// Get agent runtime statistics
    fn stats(&self) -> &AgentStats;

    /// Get agent capabilities
    fn capabilities(&self) -> &[AgentCapability];

    /// Enable a capability
    async fn enable_capability(&mut self, capability: AgentCapability) -> Result<()>;

    /// Disable a capability
    async fn disable_capability(&mut self, capability: AgentCapability) -> Result<()>;

    /// Gracefully shutdown the agent
    async fn shutdown(&mut self) -> Result<()>;
}

/// Agent factory for creating new agents
pub trait AgentFactory: Send + Sync {
    /// Create a new agent with the given configuration
    fn create_agent(&self, config: AgentConfig) -> Result<Box<dyn Agent>>;

    /// Get the supported agent types
    fn supported_types(&self) -> Vec<String>;

    /// Validate an agent configuration
    fn validate_config(&self, config: &AgentConfig) -> Result<()>;
}
