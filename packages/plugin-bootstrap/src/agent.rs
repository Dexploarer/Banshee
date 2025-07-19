//! Core agent implementation for the bootstrap plugin

use async_trait::async_trait;
use emotional_agents_core::*;
use uuid::Uuid;

/// Basic emotional agent implementation
pub struct EmotionalAgent {
    id: AgentId,
    character: CharacterSheet,
    emotional_state: EmotionalState,
    context: Context,
}

impl EmotionalAgent {
    pub fn new(config: AgentConfig) -> Self {
        let id = config.id.unwrap_or_else(|| Uuid::new_v4());
        let emotional_state = config.initial_emotions.unwrap_or_default();
        let context = Context::new(id, "default_session".to_string());

        Self {
            id,
            character: config.character,
            emotional_state,
            context,
        }
    }
}

#[async_trait]
impl Agent for EmotionalAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn character(&self) -> &CharacterSheet {
        &self.character
    }

    fn emotional_state(&self) -> &EmotionalState {
        &self.emotional_state
    }

    async fn process_message(&mut self, message: Message) -> Result<Vec<Message>> {
        // Update context with new message
        self.context.add_message(message.clone());

        // Simple response generation using the helper method
        let text_content = message.text_content();
        let response_text = if text_content.is_empty() {
            "I received your message.".to_string()
        } else {
            format!("I received your message: {}", text_content)
        };

        let response = Message::assistant(response_text);

        Ok(vec![response])
    }

    async fn update_context(&mut self, context: Context) -> Result<()> {
        self.context = context;
        Ok(())
    }

    async fn get_context(&self) -> Result<Context> {
        Ok(self.context.clone())
    }

    async fn process_emotion(&mut self, event: EmotionalEvent) -> Result<()> {
        // Simple emotional processing - would be more sophisticated in real implementation
        tracing::debug!("Processing emotional event: {:?}", event);
        Ok(())
    }

    async fn save_state(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id,
            "emotional_state": self.emotional_state,
            "context": self.context
        }))
    }

    async fn load_state(&mut self, state: serde_json::Value) -> Result<()> {
        if let Some(emotional_state) = state.get("emotional_state") {
            self.emotional_state = serde_json::from_value(emotional_state.clone())?;
        }
        if let Some(context) = state.get("context") {
            self.context = serde_json::from_value(context.clone())?;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
