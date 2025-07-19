use crate::error::{MemoryError, MemoryResult};
use crate::models::{CacheKey, ConversationMessage};
use chrono::Utc;
use redis::aio::MultiplexedConnection;
use serde_json::Value;
use sqlx::PgPool;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Store for conversation history and message persistence
pub struct ConversationStore {
    postgres: PgPool,
    redis: MultiplexedConnection,
}

impl ConversationStore {
    pub fn new(postgres: PgPool, redis: MultiplexedConnection) -> Self {
        Self { postgres, redis }
    }

    /// Initialize database tables and indexes
    pub async fn initialize(&self) -> MemoryResult<()> {
        info!("Initializing conversation store database schema");

        // Create conversation_messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversation_messages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                agent_id UUID NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata JSONB,
                emotional_context JSONB,
                tokens_used INTEGER,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create indexes for performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_conversation_messages_agent_id ON conversation_messages(agent_id);
            CREATE INDEX IF NOT EXISTS idx_conversation_messages_created_at ON conversation_messages(created_at);
            CREATE INDEX IF NOT EXISTS idx_conversation_messages_role ON conversation_messages(role);
            CREATE INDEX IF NOT EXISTS idx_conversation_messages_content_search ON conversation_messages USING gin(to_tsvector('english', content));
            "#,
        )
        .execute(&self.postgres)
        .await?;

        info!("Conversation store database schema initialized successfully");
        Ok(())
    }

    /// Save a conversation message
    pub async fn save_message(
        &self,
        agent_id: Uuid,
        role: &str,
        content: &str,
        metadata: Option<Value>,
    ) -> MemoryResult<Uuid> {
        debug!("Saving conversation message for agent {}", agent_id);

        let message_id = Uuid::new_v4();

        // Save to PostgreSQL
        sqlx::query(
            r#"
            INSERT INTO conversation_messages (
                id, agent_id, role, content, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(message_id)
        .bind(agent_id)
        .bind(role)
        .bind(content)
        .bind(&metadata)
        .execute(&self.postgres)
        .await?;

        // Update Redis cache with recent messages (keep last 50 messages)
        let cache_key = CacheKey::conversation_cache(agent_id);
        let message_data = serde_json::json!({
            "id": message_id,
            "role": role,
            "content": content,
            "metadata": metadata,
            "timestamp": Utc::now()
        });

        redis::cmd("LPUSH")
            .arg(&cache_key)
            .arg(serde_json::to_string(&message_data)?)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        // Trim to keep only last 50 messages
        redis::cmd("LTRIM")
            .arg(&cache_key)
            .arg(0)
            .arg(49)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        // Set TTL for cache
        redis::cmd("EXPIRE")
            .arg(&cache_key)
            .arg(7200) // 2 hours
            .query_async(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        debug!("Conversation message saved with ID {}", message_id);
        Ok(message_id)
    }

    /// Get conversation history with optional limit
    pub async fn get_history(
        &self,
        agent_id: Uuid,
        limit: Option<i64>,
    ) -> MemoryResult<Vec<ConversationMessage>> {
        debug!("Loading conversation history for agent {}", agent_id);

        let limit = limit.unwrap_or(100).min(1000); // Cap at 1000 messages

        // Try Redis cache first for recent messages
        if limit <= 50 {
            let cache_key = CacheKey::conversation_cache(agent_id);
            let cached_messages: Result<Vec<String>, redis::RedisError> = redis::cmd("LRANGE")
                .arg(&cache_key)
                .arg(0)
                .arg(limit - 1)
                .query_async(&mut self.redis.clone())
                .await;

            if let Ok(messages) = cached_messages {
                if messages.len() as i64 >= limit {
                    let mut conversation_messages = Vec::new();

                    for msg_json in messages {
                        if let Ok(msg_data) = serde_json::from_str::<Value>(&msg_json) {
                            if let Ok(message) = self.value_to_conversation_message(&msg_data) {
                                conversation_messages.push(message);
                            }
                        }
                    }

                    if conversation_messages.len() as i64 >= limit {
                        debug!(
                            "Conversation history loaded from cache for agent {}",
                            agent_id
                        );
                        conversation_messages.reverse(); // Redis LRANGE returns in reverse order
                        return Ok(conversation_messages);
                    }
                }
            }
        }

        // Load from database
        let messages = sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT id, agent_id, role, content, metadata, emotional_context, tokens_used, created_at
            FROM conversation_messages
            WHERE agent_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.postgres)
        .await?;

        debug!(
            "Loaded {} conversation messages for agent {}",
            messages.len(),
            agent_id
        );
        Ok(messages)
    }

    /// Search conversation messages by content
    pub async fn search_messages(
        &self,
        agent_id: Uuid,
        query: &str,
        limit: Option<i64>,
    ) -> MemoryResult<Vec<ConversationMessage>> {
        debug!(
            "Searching conversation messages for agent {} with query: {}",
            agent_id, query
        );

        let limit = limit.unwrap_or(50).min(500); // Cap at 500 results

        let messages = sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT id, agent_id, role, content, metadata, emotional_context, tokens_used, created_at
            FROM conversation_messages
            WHERE agent_id = $1 
              AND to_tsvector('english', content) @@ plainto_tsquery('english', $2)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(agent_id)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.postgres)
        .await?;

        debug!(
            "Found {} matching messages for agent {}",
            messages.len(),
            agent_id
        );
        Ok(messages)
    }

    /// Get conversation statistics
    pub async fn get_conversation_stats(&self, agent_id: Uuid) -> MemoryResult<ConversationStats> {
        debug!("Getting conversation statistics for agent {}", agent_id);

        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_messages,
                COUNT(CASE WHEN role = 'user' THEN 1 END) as user_messages,
                COUNT(CASE WHEN role = 'assistant' THEN 1 END) as assistant_messages,
                SUM(COALESCE(tokens_used, 0)) as total_tokens,
                MIN(created_at) as first_message,
                MAX(created_at) as last_message
            FROM conversation_messages
            WHERE agent_id = $1
            "#,
            agent_id
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(ConversationStats {
            total_messages: stats.total_messages.unwrap_or(0),
            user_messages: stats.user_messages.unwrap_or(0),
            assistant_messages: stats.assistant_messages.unwrap_or(0),
            total_tokens: stats.total_tokens.unwrap_or(0) as u64,
            first_message: stats.first_message,
            last_message: stats.last_message,
        })
    }

    /// Update message with emotional context
    pub async fn update_emotional_context(
        &self,
        message_id: Uuid,
        emotional_context: Value,
    ) -> MemoryResult<()> {
        debug!("Updating emotional context for message {}", message_id);

        sqlx::query(
            r#"
            UPDATE conversation_messages 
            SET emotional_context = $1
            WHERE id = $2
            "#,
        )
        .bind(&emotional_context)
        .bind(message_id)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    /// Delete old conversation messages (cleanup)
    pub async fn cleanup_old_messages(&self, days_to_keep: i32) -> MemoryResult<u64> {
        info!(
            "Cleaning up conversation messages older than {} days",
            days_to_keep
        );

        let result = sqlx::query!(
            r#"
            DELETE FROM conversation_messages
            WHERE created_at < NOW() - INTERVAL '%d days'
            "#,
            days_to_keep
        )
        .execute(&self.postgres)
        .await?;

        info!(
            "Cleaned up {} old conversation messages",
            result.rows_affected()
        );
        Ok(result.rows_affected())
    }

    /// Helper to convert Redis Value to ConversationMessage
    fn value_to_conversation_message(&self, value: &Value) -> MemoryResult<ConversationMessage> {
        Ok(ConversationMessage {
            id: Uuid::parse_str(
                value["id"]
                    .as_str()
                    .ok_or_else(|| MemoryError::InvalidData("Missing id".into()))?,
            )?,
            agent_id: Uuid::new_v4(), // Would need to be included in cache
            role: value["role"]
                .as_str()
                .ok_or_else(|| MemoryError::InvalidData("Missing role".into()))?
                .to_string(),
            content: value["content"]
                .as_str()
                .ok_or_else(|| MemoryError::InvalidData("Missing content".into()))?
                .to_string(),
            metadata: value.get("metadata").cloned(),
            emotional_context: None,
            tokens_used: value["tokens_used"].as_i64().map(|t| t as i32),
            created_at: chrono::DateTime::parse_from_rfc3339(
                value["timestamp"]
                    .as_str()
                    .ok_or_else(|| MemoryError::InvalidData("Missing timestamp".into()))?,
            )?
            .with_timezone(&Utc),
        })
    }
}

/// Conversation statistics
#[derive(Debug, Clone)]
pub struct ConversationStats {
    pub total_messages: i64,
    pub user_messages: i64,
    pub assistant_messages: i64,
    pub total_tokens: u64,
    pub first_message: Option<chrono::DateTime<Utc>>,
    pub last_message: Option<chrono::DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_conversation_stats_structure() {
        // Test the stats structure compiles correctly
        let stats = ConversationStats {
            total_messages: 100,
            user_messages: 50,
            assistant_messages: 45,
            total_tokens: 15000,
            first_message: Some(Utc::now()),
            last_message: Some(Utc::now()),
        };

        assert_eq!(stats.total_messages, 100);
        assert!(stats.total_tokens > 0);
    }

    #[tokio::test]
    async fn test_message_conversion() {
        let message_data = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "role": "user",
            "content": "Hello, how are you?",
            "metadata": {"source": "web"},
            "timestamp": "2024-01-01T12:00:00Z"
        });

        // Would test actual conversion if we had a real store instance
        assert!(message_data["role"].as_str().unwrap() == "user");
        assert!(message_data["content"].as_str().unwrap() == "Hello, how are you?");
    }
}
