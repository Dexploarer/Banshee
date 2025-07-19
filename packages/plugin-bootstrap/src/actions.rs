//! Core actions provided by the bootstrap plugin

use async_trait::async_trait;
use emotional_agents_core::action::*;
use emotional_agents_core::*;
use std::collections::HashMap;

/// Action for agent thinking/reasoning
pub struct ThinkAction {
    config: ActionConfig,
}

impl ThinkAction {
    pub fn new() -> Self {
        Self {
            config: ActionConfig {
                name: "think".to_string(),
                description: "Engage in internal reasoning and reflection".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "What to think about"
                        }
                    },
                    "required": ["topic"]
                }),
                output_schema: None,
                has_side_effects: true,
                emotional_impact: None,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Action for ThinkAction {
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
        let topic = request
            .parameters
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("general reflection");

        let thoughts = format!("Thinking about '{}'...", topic);

        Ok(ActionResult {
            success: true,
            data: serde_json::json!({
                "thoughts": thoughts,
                "topic": topic
            }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Think about a topic".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "topic".to_string(),
                    serde_json::Value::String("problem solving".to_string()),
                );
                params
            },
            expected_output: serde_json::json!({
                "thoughts": "Thinking about 'problem solving'...",
                "topic": "problem solving"
            }),
        }]
    }
}

/// Action for generating responses
pub struct RespondAction {
    config: ActionConfig,
}

impl RespondAction {
    pub fn new() -> Self {
        Self {
            config: ActionConfig {
                name: "respond".to_string(),
                description: "Generate a response to a message".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to respond to"
                        }
                    },
                    "required": ["message"]
                }),
                output_schema: None,
                has_side_effects: false,
                emotional_impact: None,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Action for RespondAction {
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
        let message = request
            .parameters
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let response = format!("I understand your message: {}", message);

        Ok(ActionResult {
            success: true,
            data: serde_json::json!({
                "response": response,
                "original_message": message
            }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Respond to a user message".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "message".to_string(),
                    serde_json::Value::String("Hello".to_string()),
                );
                params
            },
            expected_output: serde_json::json!({
                "response": "I understand your message: Hello",
                "original_message": "Hello"
            }),
        }]
    }
}

/// Action for self-reflection and learning
pub struct ReflectAction {
    config: ActionConfig,
}

impl ReflectAction {
    pub fn new() -> Self {
        Self {
            config: ActionConfig {
                name: "reflect".to_string(),
                description: "Reflect on past interactions and learn from them".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: None,
                has_side_effects: true,
                emotional_impact: None,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Action for ReflectAction {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.config
    }

    async fn execute(&self, _request: ActionRequest) -> Result<ActionResult> {
        let reflection = "Reflecting on recent interactions and learning...";

        Ok(ActionResult {
            success: true,
            data: serde_json::json!({
                "reflection": reflection,
                "insights": ["Better understanding", "Improved responses"]
            }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Reflect on recent interactions".to_string(),
            parameters: HashMap::new(),
            expected_output: serde_json::json!({
                "reflection": "Reflecting on recent interactions and learning...",
                "insights": ["Better understanding", "Improved responses"]
            }),
        }]
    }
}
