use crate::error::{MemoryError, MemoryResult};
use crate::models::{CacheKey, MemoryRecord};
use crate::{ConversationStore, EmotionStore};
use chrono::{Duration, Utc};
use redis::aio::MultiplexedConnection;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Central memory manager coordinating all storage operations
pub struct MemoryManager {
    postgres: PgPool,
    redis: MultiplexedConnection,
    emotion_store: Arc<EmotionStore>,
    conversation_store: Arc<ConversationStore>,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MemoryManager {
    /// Create a new memory manager with database connections
    pub async fn new(postgres_url: &str, redis_url: &str) -> MemoryResult<Self> {
        info!("Initializing memory manager");

        // Connect to PostgreSQL
        let postgres = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(postgres_url)
            .await
            .map_err(|e| MemoryError::Connection(format!("PostgreSQL connection failed: {}", e)))?;

        // Connect to Redis
        let redis_client = redis::Client::open(redis_url)
            .map_err(|e| MemoryError::Connection(format!("Redis client creation failed: {}", e)))?;

        let redis = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| MemoryError::Connection(format!("Redis connection failed: {}", e)))?;

        // Create specialized stores
        let emotion_store = Arc::new(EmotionStore::new(postgres.clone(), redis.clone()));
        let conversation_store = Arc::new(ConversationStore::new(postgres.clone(), redis.clone()));

        Ok(Self {
            postgres,
            redis,
            emotion_store,
            conversation_store,
            cleanup_handle: None,
        })
    }

    /// Initialize all database schemas and start background tasks
    pub async fn initialize(&mut self) -> MemoryResult<()> {
        info!("Initializing memory manager components");

        // Initialize database schemas
        self.create_core_tables().await?;
        self.emotion_store.initialize().await?;
        self.conversation_store.initialize().await?;

        // Start background cleanup task
        self.start_cleanup_task().await?;

        info!("Memory manager initialized successfully");
        Ok(())
    }

    /// Shutdown the memory manager and cleanup resources
    pub async fn shutdown(&mut self) -> MemoryResult<()> {
        info!("Shutting down memory manager");

        // Stop cleanup task
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }

        // Close database connections
        self.postgres.close().await;

        info!("Memory manager shutdown complete");
        Ok(())
    }

    /// Get reference to emotion store
    pub fn emotion_store(&self) -> &EmotionStore {
        &self.emotion_store
    }

    /// Get reference to conversation store  
    pub fn conversation_store(&self) -> &ConversationStore {
        &self.conversation_store
    }

    /// Store custom memory data with optional TTL
    pub async fn store_memory(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        key: &str,
        data: Value,
        ttl_seconds: Option<i64>,
    ) -> MemoryResult<()> {
        debug!(
            "Storing memory for agent {}: {}:{}",
            agent_id, memory_type, key
        );

        let expires_at = ttl_seconds.map(|ttl| Utc::now() + Duration::seconds(ttl));
        let importance = self.calculate_memory_importance(&data, memory_type);

        // Save to PostgreSQL
        sqlx::query(
            r#"
            INSERT INTO memory_records (
                agent_id, memory_type, key, data, importance, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (agent_id, memory_type, key) DO UPDATE SET
                data = EXCLUDED.data,
                importance = EXCLUDED.importance,
                expires_at = EXCLUDED.expires_at,
                access_count = memory_records.access_count + 1,
                last_accessed = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(agent_id)
        .bind(memory_type)
        .bind(key)
        .bind(&data)
        .bind(importance)
        .bind(expires_at)
        .execute(&self.postgres)
        .await?;

        // Cache in Redis with TTL
        let cache_key = CacheKey::memory_cache(agent_id, memory_type, key);
        let data_json = serde_json::to_string(&data)?;

        if let Some(ttl) = ttl_seconds {
            redis::cmd("SET")
                .arg(&cache_key)
                .arg(&data_json)
                .arg("EX")
                .arg(ttl)
                .query_async(&mut self.redis.clone())
                .await
                .map_err(MemoryError::Redis)?;
        } else {
            redis::cmd("SET")
                .arg(&cache_key)
                .arg(&data_json)
                .query_async(&mut self.redis.clone())
                .await
                .map_err(MemoryError::Redis)?;
        }

        debug!(
            "Memory stored for agent {}: {}:{}",
            agent_id, memory_type, key
        );
        Ok(())
    }

    /// Retrieve custom memory data
    pub async fn retrieve_memory(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        key: &str,
    ) -> MemoryResult<Option<Value>> {
        debug!(
            "Retrieving memory for agent {}: {}:{}",
            agent_id, memory_type, key
        );

        // Try Redis cache first
        let cache_key = CacheKey::memory_cache(agent_id, memory_type, key);
        let cached_result: Result<String, redis::RedisError> = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut self.redis.clone())
            .await;

        if let Ok(cached_json) = cached_result {
            if !cached_json.is_empty() {
                match serde_json::from_str::<Value>(&cached_json) {
                    Ok(data) => {
                        // Update access count in background
                        let postgres = self.postgres.clone();
                        tokio::spawn(async move {
                            let _ = sqlx::query(
                                r#"
                                UPDATE memory_records 
                                SET access_count = access_count + 1, last_accessed = NOW()
                                WHERE agent_id = $1 AND memory_type = $2 AND key = $3
                                "#,
                            )
                            .bind(agent_id)
                            .bind(memory_type)
                            .bind(key)
                            .execute(&postgres)
                            .await;
                        });

                        debug!(
                            "Memory retrieved from cache for agent {}: {}:{}",
                            agent_id, memory_type, key
                        );
                        return Ok(Some(data));
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize cached memory for agent {}: {}",
                            agent_id, e
                        );
                    }
                }
            }
        }

        // Load from database
        let record = sqlx::query_as::<_, MemoryRecord>(
            r#"
            SELECT id, agent_id, memory_type, key, data, importance, 
                   access_count, last_accessed, created_at, updated_at, expires_at
            FROM memory_records
            WHERE agent_id = $1 AND memory_type = $2 AND key = $3
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(agent_id)
        .bind(memory_type)
        .bind(key)
        .fetch_optional(&self.postgres)
        .await?;

        if let Some(record) = record {
            // Update access count
            sqlx::query(
                r#"
                UPDATE memory_records 
                SET access_count = access_count + 1, last_accessed = NOW()
                WHERE id = $1
                "#,
            )
            .bind(record.id)
            .execute(&self.postgres)
            .await?;

            // Update cache
            let data_json = serde_json::to_string(&record.data)?;
            redis::cmd("SET")
                .arg(&cache_key)
                .arg(&data_json)
                .arg("EX")
                .arg(3600) // 1 hour cache
                .query_async(&mut self.redis.clone())
                .await
                .map_err(MemoryError::Redis)?;

            debug!(
                "Memory retrieved from database for agent {}: {}:{}",
                agent_id, memory_type, key
            );
            Ok(Some(record.data))
        } else {
            debug!(
                "Memory not found for agent {}: {}:{}",
                agent_id, memory_type, key
            );
            Ok(None)
        }
    }

    /// Get memory records by type with pagination
    pub async fn get_memories_by_type(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> MemoryResult<Vec<MemoryRecord>> {
        debug!(
            "Getting memories by type for agent {}: {}",
            agent_id, memory_type
        );

        let limit = limit.unwrap_or(50).min(500);
        let offset = offset.unwrap_or(0);

        let records = sqlx::query_as::<_, MemoryRecord>(
            r#"
            SELECT id, agent_id, memory_type, key, data, importance,
                   access_count, last_accessed, created_at, updated_at, expires_at
            FROM memory_records
            WHERE agent_id = $1 AND memory_type = $2
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY importance DESC, last_accessed DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(agent_id)
        .bind(memory_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.postgres)
        .await?;

        debug!(
            "Retrieved {} memories of type {} for agent {}",
            records.len(),
            memory_type,
            agent_id
        );
        Ok(records)
    }

    /// Delete memory
    pub async fn delete_memory(
        &self,
        agent_id: Uuid,
        memory_type: &str,
        key: &str,
    ) -> MemoryResult<bool> {
        debug!(
            "Deleting memory for agent {}: {}:{}",
            agent_id, memory_type, key
        );

        // Delete from database
        let result = sqlx::query(
            r#"
            DELETE FROM memory_records
            WHERE agent_id = $1 AND memory_type = $2 AND key = $3
            "#,
        )
        .bind(agent_id)
        .bind(memory_type)
        .bind(key)
        .execute(&self.postgres)
        .await?;

        // Delete from cache
        let cache_key = CacheKey::memory_cache(agent_id, memory_type, key);
        redis::cmd("DEL")
            .arg(&cache_key)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        let deleted = result.rows_affected() > 0;
        debug!(
            "Memory deletion result for agent {}: {}:{} = {}",
            agent_id, memory_type, key, deleted
        );
        Ok(deleted)
    }

    /// Create core database tables
    async fn create_core_tables(&self) -> MemoryResult<()> {
        info!("Creating core memory tables");

        // Memory records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memory_records (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                agent_id UUID NOT NULL,
                memory_type TEXT NOT NULL,
                key TEXT NOT NULL,
                data JSONB NOT NULL,
                importance REAL NOT NULL DEFAULT 0.5,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ,
                UNIQUE(agent_id, memory_type, key)
            );
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_memory_records_agent_id ON memory_records(agent_id);
            CREATE INDEX IF NOT EXISTS idx_memory_records_type ON memory_records(memory_type);
            CREATE INDEX IF NOT EXISTS idx_memory_records_importance ON memory_records(importance DESC);
            CREATE INDEX IF NOT EXISTS idx_memory_records_expires_at ON memory_records(expires_at);
            CREATE INDEX IF NOT EXISTS idx_memory_records_last_accessed ON memory_records(last_accessed);
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create trigger for updating timestamps
        sqlx::query(
            r#"
            DROP TRIGGER IF EXISTS update_memory_records_updated_at ON memory_records;
            CREATE TRIGGER update_memory_records_updated_at
                BEFORE UPDATE ON memory_records
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
            "#,
        )
        .execute(&self.postgres)
        .await?;

        info!("Core memory tables created successfully");
        Ok(())
    }

    /// Start background cleanup task
    async fn start_cleanup_task(&mut self) -> MemoryResult<()> {
        info!("Starting background cleanup task");

        let postgres = self.postgres.clone();
        let emotion_store = self.emotion_store.clone();
        let conversation_store = self.conversation_store.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Run every hour

            loop {
                interval.tick().await;

                // Cleanup expired memories
                if let Err(e) = sqlx::query(
                    "DELETE FROM memory_records WHERE expires_at IS NOT NULL AND expires_at < NOW()"
                ).execute(&postgres).await {
                    error!("Failed to cleanup expired memories: {}", e);
                }

                // Cleanup old emotional events (keep 30 days)
                if let Err(e) = emotion_store.cleanup_old_events(30).await {
                    error!("Failed to cleanup old emotional events: {}", e);
                }

                // Cleanup old conversation messages (keep 90 days)
                if let Err(e) = conversation_store.cleanup_old_messages(90).await {
                    error!("Failed to cleanup old conversation messages: {}", e);
                }

                info!("Background cleanup completed");
            }
        });

        self.cleanup_handle = Some(handle);
        Ok(())
    }

    /// Calculate memory importance based on content and type
    fn calculate_memory_importance(&self, data: &Value, memory_type: &str) -> f32 {
        let mut importance = 0.5; // Base importance

        // Adjust based on memory type
        match memory_type {
            "goal" | "objective" => importance += 0.3,
            "preference" | "personality" => importance += 0.2,
            "skill" | "capability" => importance += 0.1,
            "fact" | "knowledge" => importance += 0.05,
            "temp" | "cache" => importance -= 0.2,
            _ => {}
        }

        // Adjust based on data size/complexity
        if let Some(obj) = data.as_object() {
            let field_count = obj.len() as f32;
            importance += (field_count / 20.0).min(0.2);
        }

        // Adjust based on data content
        if let Some(text) = data.as_str() {
            let word_count = text.split_whitespace().count() as f32;
            importance += (word_count / 100.0).min(0.1);
        }

        importance.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_importance_calculation() {
        let manager = MemoryManager {
            postgres: PgPool::connect("postgresql://fake").await.unwrap(),
            redis: redis::Client::open("redis://fake")
                .unwrap()
                .get_multiplexed_async_connection()
                .await
                .unwrap(),
            emotion_store: Arc::new(EmotionStore::new(
                PgPool::connect("postgresql://fake").await.unwrap(),
                redis::Client::open("redis://fake")
                    .unwrap()
                    .get_multiplexed_async_connection()
                    .await
                    .unwrap(),
            )),
            conversation_store: Arc::new(ConversationStore::new(
                PgPool::connect("postgresql://fake").await.unwrap(),
                redis::Client::open("redis://fake")
                    .unwrap()
                    .get_multiplexed_async_connection()
                    .await
                    .unwrap(),
            )),
            cleanup_handle: None,
        };

        // Test goal memory (high importance)
        let goal_data = json!({"objective": "Complete the task successfully"});
        let goal_importance = manager.calculate_memory_importance(&goal_data, "goal");
        assert!(goal_importance > 0.7);

        // Test cache memory (low importance)
        let cache_data = json!({"temp": "data"});
        let cache_importance = manager.calculate_memory_importance(&cache_data, "cache");
        assert!(cache_importance < 0.4);

        // Test complex data (higher importance)
        let complex_data = json!({
            "field1": "value1",
            "field2": "value2",
            "field3": "value3",
            "field4": "value4",
            "field5": "value5"
        });
        let complex_importance = manager.calculate_memory_importance(&complex_data, "fact");
        let simple_importance =
            manager.calculate_memory_importance(&json!({"simple": "data"}), "fact");
        assert!(complex_importance > simple_importance);
    }

    #[test]
    fn test_cache_keys() {
        let agent_id = Uuid::new_v4();

        let emotion_key = CacheKey::emotional_state(agent_id);
        let memory_key = CacheKey::memory_cache(agent_id, "goals", "primary");
        let conversation_key = CacheKey::conversation_cache(agent_id);

        assert!(emotion_key.contains("emotion:state"));
        assert!(memory_key.contains("memory:"));
        assert!(conversation_key.contains("conversation:cache"));
    }
}
