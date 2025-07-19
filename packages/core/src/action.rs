use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Context, Message, Result};

/// Action request containing parameters and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Name of the action to execute
    pub action_name: String,

    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,

    /// Message that triggered this action
    pub trigger_message: Message,

    /// Current conversation context
    pub context: Context,

    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub success: bool,

    /// Result data from the action
    pub data: serde_json::Value,

    /// Optional error message
    pub error: Option<String>,

    /// Additional context or side effects
    pub side_effects: Vec<SideEffect>,

    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Side effects that actions can produce
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SideEffect {
    /// Update agent's emotional state
    EmotionalUpdate {
        emotions: HashMap<String, f32>,
        reason: String,
    },

    /// Store information in memory
    MemoryStore {
        content: String,
        importance: f32,
        tags: Vec<String>,
    },

    /// Send a message to another agent
    MessageSend {
        recipient: crate::AgentId,
        message: Message,
    },

    /// Update agent capabilities
    CapabilityUpdate { capability: String, enabled: bool },

    /// Log an event
    LogEvent {
        level: String,
        message: String,
        data: HashMap<String, serde_json::Value>,
    },

    /// Custom side effect
    Custom {
        name: String,
        data: serde_json::Value,
    },
}

/// Configuration for an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    /// Action name
    pub name: String,

    /// Action description
    pub description: String,

    /// Input parameter schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Expected output schema
    pub output_schema: Option<serde_json::Value>,

    /// Whether this action can have side effects
    pub has_side_effects: bool,

    /// Emotional impact of this action
    pub emotional_impact: Option<EmotionalImpact>,

    /// Configuration settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Emotional impact configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalImpact {
    /// Base emotional changes when action succeeds
    pub on_success: HashMap<String, f32>,

    /// Base emotional changes when action fails
    pub on_failure: HashMap<String, f32>,

    /// Emotional intensity multiplier (0.0 to 2.0)
    pub intensity_multiplier: f32,
}

/// Core action trait that all actions must implement
#[async_trait]
pub trait Action: Send + Sync {
    /// Get the action name
    fn name(&self) -> &str;

    /// Get the action description
    fn description(&self) -> &str;

    /// Get the action configuration
    fn config(&self) -> &ActionConfig;

    /// Execute the action with given request
    async fn execute(&self, request: ActionRequest) -> Result<ActionResult>;

    /// Validate action parameters before execution
    async fn validate(&self, parameters: &HashMap<String, serde_json::Value>) -> Result<()>;

    /// Check if action is available in current context
    async fn is_available(&self, context: &Context) -> Result<bool>;

    /// Get examples of how to use this action
    fn examples(&self) -> Vec<ActionExample>;
}

/// Example of how to use an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExample {
    /// Example description
    pub description: String,

    /// Example input parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected output
    pub expected_output: serde_json::Value,
}

/// Registry for managing available actions
pub struct ActionRegistry {
    actions: HashMap<String, Box<dyn Action>>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register a new action
    pub fn register(&mut self, action: Box<dyn Action>) {
        self.actions.insert(action.name().to_string(), action);
    }

    /// Get an action by name
    pub fn get(&self, name: &str) -> Option<&dyn Action> {
        self.actions.get(name).map(|a| a.as_ref())
    }

    /// Get all registered actions
    pub fn all(&self) -> impl Iterator<Item = &dyn Action> {
        self.actions.values().map(|a| a.as_ref())
    }

    /// Get actions available in a given context
    pub async fn available_in_context(&self, context: &Context) -> Result<Vec<&dyn Action>> {
        let mut available = Vec::new();

        for action in self.actions.values() {
            if action.is_available(context).await? {
                available.push(action.as_ref());
            }
        }

        Ok(available)
    }
}

/// Macro to create simple actions
#[macro_export]
macro_rules! simple_action {
    ($name:expr, $description:expr, $execute:expr) => {
        struct SimpleAction {
            config: ActionConfig,
            execute_fn: fn(ActionRequest) -> Result<ActionResult>,
        }

        #[async_trait]
        impl Action for SimpleAction {
            fn name(&self) -> &str {
                &self.config.name
            }

            fn description(&self) -> &str {
                &self.config.description
            }

            fn config(&self) -> &ActionConfig {
                &self.config
            }

            async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
                (self.execute_fn)(request)
            }

            async fn validate(&self, _: &HashMap<String, serde_json::Value>) -> Result<()> {
                Ok(())
            }

            async fn is_available(&self, _: &Context) -> Result<bool> {
                Ok(true)
            }

            fn examples(&self) -> Vec<ActionExample> {
                vec![]
            }
        }

        SimpleAction {
            config: ActionConfig {
                name: $name.to_string(),
                description: $description.to_string(),
                input_schema: serde_json::json!({}),
                output_schema: None,
                has_side_effects: false,
                emotional_impact: None,
                settings: HashMap::new(),
            },
            execute_fn: $execute,
        }
    };
}
