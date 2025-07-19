use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{AgentId, EmotionalState, Message};

/// Runtime context for agents containing all relevant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Current agent ID
    pub agent_id: AgentId,

    /// Session identifier
    pub session_id: String,

    /// User identifier (if available)
    pub user_id: Option<String>,

    /// Current conversation
    pub conversation: Vec<Message>,

    /// Agent's current emotional state
    pub emotional_state: EmotionalState,

    /// Available capabilities
    pub capabilities: Vec<String>,

    /// Context metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// When this context was created
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Context {
    /// Create a new context
    pub fn new(agent_id: AgentId, session_id: String) -> Self {
        let now = Utc::now();
        Self {
            agent_id,
            session_id,
            user_id: None,
            conversation: Vec::new(),
            emotional_state: EmotionalState::new(),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.conversation.push(message);
        self.updated_at = Utc::now();
    }

    /// Get the latest message
    pub fn latest_message(&self) -> Option<&Message> {
        self.conversation.last()
    }

    /// Get conversation history (last N messages)
    pub fn recent_messages(&self, count: usize) -> &[Message] {
        let start = self.conversation.len().saturating_sub(count);
        &self.conversation[start..]
    }

    /// Update emotional state
    pub fn update_emotional_state(&mut self, state: EmotionalState) {
        self.emotional_state = state;
        self.updated_at = Utc::now();
    }

    /// Add capability
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
            self.updated_at = Utc::now();
        }
    }

    /// Remove capability
    pub fn remove_capability(&mut self, capability: &str) {
        self.capabilities.retain(|c| c != capability);
        self.updated_at = Utc::now();
    }

    /// Check if capability is available
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set user ID
    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
        self.updated_at = Utc::now();
    }
}

/// Message-specific context for processing individual messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    /// Reference to the main context
    pub context: Context,

    /// The specific message being processed
    pub current_message: Message,

    /// Previous message in conversation (if any)
    pub previous_message: Option<Message>,

    /// Processing metadata
    pub processing_metadata: HashMap<String, serde_json::Value>,

    /// Processing start time
    pub processing_started: DateTime<Utc>,
}

impl MessageContext {
    /// Create message context from main context and message
    pub fn new(context: Context, message: Message) -> Self {
        let previous_message = context.latest_message().cloned();

        Self {
            context,
            current_message: message,
            previous_message,
            processing_metadata: HashMap::new(),
            processing_started: Utc::now(),
        }
    }

    /// Add processing metadata
    pub fn add_processing_metadata(&mut self, key: String, value: serde_json::Value) {
        self.processing_metadata.insert(key, value);
    }

    /// Get processing duration
    pub fn processing_duration(&self) -> chrono::Duration {
        Utc::now() - self.processing_started
    }

    /// Get current message text
    pub fn current_text(&self) -> String {
        self.current_message.text_content()
    }

    /// Get previous message text
    pub fn previous_text(&self) -> Option<String> {
        self.previous_message.as_ref().map(|m| m.text_content())
    }
}

/// User context containing information about the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User identifier
    pub user_id: String,

    /// User preferences
    pub preferences: HashMap<String, serde_json::Value>,

    /// Interaction history summary
    pub history_summary: String,

    /// User's emotional patterns
    pub emotional_patterns: HashMap<String, f32>,

    /// Trust level with this user (0.0 to 1.0)
    pub trust_level: f32,

    /// Communication style preferences
    pub communication_style: CommunicationStyle,

    /// Last interaction timestamp
    pub last_interaction: DateTime<Utc>,
}

/// Communication style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Preferred formality level (0.0 = casual, 1.0 = formal)
    pub formality: f32,

    /// Preferred response length (0.0 = concise, 1.0 = detailed)
    pub verbosity: f32,

    /// Preferred emotional expression (0.0 = neutral, 1.0 = expressive)
    pub emotional_expression: f32,

    /// Preferred interaction pace (0.0 = slow, 1.0 = fast)
    pub pace: f32,
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        Self {
            formality: 0.5,
            verbosity: 0.5,
            emotional_expression: 0.5,
            pace: 0.5,
        }
    }
}

/// Session context for tracking conversation sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session identifier
    pub session_id: String,

    /// Session start time
    pub started_at: DateTime<Utc>,

    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Goals for this session
    pub goals: Vec<String>,

    /// Session state
    pub state: SessionState,

    /// Participants in this session
    pub participants: Vec<AgentId>,
}

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active
    Active,

    /// Session is paused
    Paused,

    /// Session completed successfully
    Completed,

    /// Session ended with error
    Failed(String),

    /// Session timed out
    TimedOut,
}

/// Environment context containing external factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// Current timestamp
    pub timestamp: DateTime<Utc>,

    /// Platform being used (Discord, Web, etc.)
    pub platform: String,

    /// Available external resources
    pub available_resources: Vec<String>,

    /// System load indicators
    pub system_load: HashMap<String, f32>,

    /// Rate limiting information
    pub rate_limits: HashMap<String, RateLimitInfo>,
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Requests remaining
    pub remaining: u32,

    /// Reset time
    pub reset_at: DateTime<Utc>,

    /// Request limit
    pub limit: u32,
}
