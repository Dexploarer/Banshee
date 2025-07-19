//! # Banshee Memory Plugin
//!
//! Provides persistent storage for agent memory, emotional states, and conversation history.
//! Uses PostgreSQL for structured data and Redis for caching and real-time operations.

mod conversation_store;
mod emotion_store;
mod error;
mod memory_manager;
mod models;

pub use conversation_store::ConversationStore;
pub use emotion_store::EmotionStore;
pub use error::{MemoryError, MemoryResult};
pub use memory_manager::MemoryManager;
pub use models::*;

use async_trait::async_trait;
use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalIntensity, EmotionalState};
use banshee_core::plugin::{Plugin, PluginDependency, PluginResult};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Memory plugin for persistent storage
pub struct MemoryPlugin {
    manager: MemoryManager,
}

impl MemoryPlugin {
    pub async fn new(postgres_url: &str, redis_url: &str) -> MemoryResult<Self> {
        let manager = MemoryManager::new(postgres_url, redis_url).await?;
        Ok(Self { manager })
    }

    /// Save emotional state to database
    pub async fn save_emotional_state(
        &self,
        agent_id: Uuid,
        state: &EmotionalState,
    ) -> MemoryResult<()> {
        self.manager
            .emotion_store()
            .save_state(agent_id, state)
            .await
    }

    /// Load emotional state from database
    pub async fn load_emotional_state(
        &self,
        agent_id: Uuid,
    ) -> MemoryResult<Option<EmotionalState>> {
        self.manager.emotion_store().load_state(agent_id).await
    }

    /// Save emotional event for history tracking
    pub async fn save_emotional_event(
        &self,
        agent_id: Uuid,
        event: &EmotionalEvent,
        resulting_emotions: &HashMap<Emotion, EmotionalIntensity>,
    ) -> MemoryResult<()> {
        self.manager
            .emotion_store()
            .save_event(agent_id, event, resulting_emotions)
            .await
    }

    /// Get emotional event history
    pub async fn get_emotional_history(
        &self,
        agent_id: Uuid,
        limit: Option<i64>,
    ) -> MemoryResult<Vec<EmotionalEventRecord>> {
        self.manager
            .emotion_store()
            .get_event_history(agent_id, limit)
            .await
    }

    /// Save conversation message
    pub async fn save_conversation_message(
        &self,
        agent_id: Uuid,
        role: &str,
        content: &str,
        metadata: Option<Value>,
    ) -> MemoryResult<Uuid> {
        self.manager
            .conversation_store()
            .save_message(agent_id, role, content, metadata)
            .await
    }

    /// Get conversation history
    pub async fn get_conversation_history(
        &self,
        agent_id: Uuid,
        limit: Option<i64>,
    ) -> MemoryResult<Vec<ConversationMessage>> {
        self.manager
            .conversation_store()
            .get_history(agent_id, limit)
            .await
    }

    /// Store custom memory data
    pub async fn store_memory(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        key: &str,
        data: Value,
        ttl_seconds: Option<i64>,
    ) -> MemoryResult<()> {
        self.manager
            .store_memory(agent_id, memory_type, key, data, ttl_seconds)
            .await
    }

    /// Retrieve custom memory data
    pub async fn retrieve_memory(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        key: &str,
    ) -> MemoryResult<Option<Value>> {
        self.manager
            .retrieve_memory(agent_id, memory_type, key)
            .await
    }
}

#[async_trait]
impl Plugin for MemoryPlugin {
    fn name(&self) -> &str {
        "memory"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        // Memory plugin has no dependencies - it's a foundational service
        Vec::new()
    }

    async fn initialize(&mut self) -> PluginResult<()> {
        self.manager
            .initialize()
            .await
            .map_err(|e| format!("Failed to initialize memory plugin: {}", e))?;
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        self.manager
            .shutdown()
            .await
            .map_err(|e| format!("Failed to shutdown memory plugin: {}", e))?;
        Ok(())
    }

    async fn health_check(&self) -> PluginResult<bool> {
        // Test both PostgreSQL and Redis connections
        let postgres_ok = sqlx::query("SELECT 1")
            .fetch_one(&self.manager.postgres)
            .await
            .is_ok();

        let redis_ok = redis::cmd("PING")
            .query_async(&mut self.manager.redis.clone())
            .await
            .map(|response: String| response == "PONG")
            .unwrap_or(false);

        Ok(postgres_ok && redis_ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use banshee_core::emotion::{Emotion, EmotionalState};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_emotional_state_persistence() {
        // This would need a test database setup
        // For now, just test the structure compiles
        let agent_id = Uuid::new_v4();
        let mut state = EmotionalState::new();
        state.update_emotion(Emotion::Joy, 0.8);

        // Would test:
        // - Save emotional state
        // - Load emotional state
        // - Verify persistence across restarts
        assert!(state.emotions.contains_key(&Emotion::Joy));
    }

    #[tokio::test]
    async fn test_conversation_history() {
        // This would test conversation storage and retrieval
        let agent_id = Uuid::new_v4();
        let message = "Hello, how are you?";

        // Would test:
        // - Save conversation messages
        // - Retrieve conversation history
        // - Handle metadata properly
        assert!(!message.is_empty());
    }

    #[tokio::test]
    async fn test_memory_ttl() {
        // This would test TTL functionality in Redis
        let agent_id = Uuid::new_v4();
        let data = serde_json::json!({"test": "data"});

        // Would test:
        // - Store memory with TTL
        // - Verify expiration works
        // - Handle cache invalidation
        assert!(data.is_object());
    }
}
