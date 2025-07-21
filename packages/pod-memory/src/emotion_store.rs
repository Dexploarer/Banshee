use crate::error::{MemoryError, MemoryResult};
use crate::models::{CacheKey, EmotionalEventRecord, EmotionalStateRecord, ModelConverter};
use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalIntensity, EmotionalState};
use chrono::Utc;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Store for emotional state persistence and event history
pub struct EmotionStore {
    postgres: PgPool,
    redis: MultiplexedConnection,
}

impl EmotionStore {
    pub fn new(postgres: PgPool, redis: MultiplexedConnection) -> Self {
        Self { postgres, redis }
    }

    /// Initialize database tables and indexes
    pub async fn initialize(&self) -> MemoryResult<()> {
        info!("Initializing emotion store database schema");

        // Create emotional_states table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS emotional_states (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                agent_id UUID NOT NULL,
                emotions JSONB NOT NULL,
                decay_rates JSONB NOT NULL,
                overall_valence REAL NOT NULL,
                overall_arousal REAL NOT NULL,
                dominant_emotion TEXT,
                dominant_intensity REAL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create emotional_events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS emotional_events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                agent_id UUID NOT NULL,
                event_type TEXT NOT NULL,
                event_data JSONB NOT NULL,
                resulting_emotions JSONB NOT NULL,
                context_factors JSONB,
                intensity_delta REAL NOT NULL,
                valence_delta REAL NOT NULL,
                arousal_delta REAL NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create indexes for performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_emotional_states_agent_id ON emotional_states(agent_id);
            CREATE INDEX IF NOT EXISTS idx_emotional_states_updated_at ON emotional_states(updated_at);
            CREATE INDEX IF NOT EXISTS idx_emotional_events_agent_id ON emotional_events(agent_id);
            CREATE INDEX IF NOT EXISTS idx_emotional_events_created_at ON emotional_events(created_at);
            CREATE INDEX IF NOT EXISTS idx_emotional_events_type ON emotional_events(event_type);
            "#,
        )
        .execute(&self.postgres)
        .await?;

        // Create triggers for auto-updating timestamps
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql';

            DROP TRIGGER IF EXISTS update_emotional_states_updated_at ON emotional_states;
            CREATE TRIGGER update_emotional_states_updated_at
                BEFORE UPDATE ON emotional_states
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
            "#,
        )
        .execute(&self.postgres)
        .await?;

        info!("Emotion store database schema initialized successfully");
        Ok(())
    }

    /// Save emotional state to both PostgreSQL and Redis cache
    pub async fn save_state(&self, agent_id: Uuid, state: &EmotionalState) -> MemoryResult<()> {
        debug!("Saving emotional state for agent {}", agent_id);

        // Convert to database format
        let (emotions_json, decay_rates_json) =
            ModelConverter::emotional_state_to_json(&state.emotions, &state.decay_rates)?;

        // Calculate metrics
        let (valence, arousal, dominant) = ModelConverter::calculate_state_metrics(&state.emotions);

        let (dominant_emotion, dominant_intensity) = match dominant {
            Some((emotion, intensity)) => (Some(emotion), Some(intensity)),
            None => (None, None),
        };

        // Save to PostgreSQL (upsert)
        sqlx::query(
            r#"
            INSERT INTO emotional_states (
                agent_id, emotions, decay_rates, overall_valence, overall_arousal,
                dominant_emotion, dominant_intensity, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
            ON CONFLICT (agent_id) DO UPDATE SET
                emotions = EXCLUDED.emotions,
                decay_rates = EXCLUDED.decay_rates,
                overall_valence = EXCLUDED.overall_valence,
                overall_arousal = EXCLUDED.overall_arousal,
                dominant_emotion = EXCLUDED.dominant_emotion,
                dominant_intensity = EXCLUDED.dominant_intensity,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(agent_id)
        .bind(&emotions_json)
        .bind(&decay_rates_json)
        .bind(valence)
        .bind(arousal)
        .bind(dominant_emotion)
        .bind(dominant_intensity)
        .bind(state.last_updated)
        .execute(&self.postgres)
        .await?;

        // Cache in Redis for quick access
        let cache_key = CacheKey::emotional_state(agent_id);
        let state_json = serde_json::to_string(state)?;

        redis::cmd("SET")
            .arg(&cache_key)
            .arg(&state_json)
            .arg("EX")
            .arg(3600) // 1 hour TTL
            .query_async::<()>(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        info!("Emotional state saved for agent {}", agent_id);
        Ok(())
    }

    /// Load emotional state from cache first, then database
    pub async fn load_state(&self, agent_id: Uuid) -> MemoryResult<Option<EmotionalState>> {
        debug!("Loading emotional state for agent {}", agent_id);

        // Try Redis cache first
        let cache_key = CacheKey::emotional_state(agent_id);
        let cached_result: Result<String, redis::RedisError> = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut self.redis.clone())
            .await;

        if let Ok(cached_json) = cached_result {
            if !cached_json.is_empty() {
                match serde_json::from_str::<EmotionalState>(&cached_json) {
                    Ok(state) => {
                        debug!("Emotional state loaded from cache for agent {}", agent_id);
                        return Ok(Some(state));
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize cached state for agent {}: {}",
                            agent_id, e
                        );
                    }
                }
            }
        }

        // Load from database
        let record = sqlx::query_as::<_, EmotionalStateRecord>(
            r#"
            SELECT id, agent_id, emotions, decay_rates, overall_valence, overall_arousal,
                   dominant_emotion, dominant_intensity, created_at, updated_at
            FROM emotional_states
            WHERE agent_id = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(agent_id)
        .fetch_optional(&self.postgres)
        .await?;

        if let Some(record) = record {
            // Convert back to domain model
            let (emotions, decay_rates) =
                ModelConverter::json_to_emotional_state(&record.emotions, &record.decay_rates)?;

            let state = EmotionalState {
                emotions,
                last_updated: record.updated_at,
                decay_rates,
            };

            // Update cache
            let state_json = serde_json::to_string(&state)?;
            redis::cmd("SET")
                .arg(&cache_key)
                .arg(&state_json)
                .arg("EX")
                .arg(3600)
                .query_async::<()>(&mut self.redis.clone())
                .await
                .map_err(MemoryError::Redis)?;

            debug!(
                "Emotional state loaded from database for agent {}",
                agent_id
            );
            Ok(Some(state))
        } else {
            debug!("No emotional state found for agent {}", agent_id);
            Ok(None)
        }
    }

    /// Save emotional event and resulting emotions
    pub async fn save_event(
        &self,
        agent_id: Uuid,
        event: &EmotionalEvent,
        resulting_emotions: &HashMap<Emotion, EmotionalIntensity>,
    ) -> MemoryResult<()> {
        debug!("Saving emotional event for agent {}", agent_id);

        // Convert event to database format
        let (event_type, event_data) = ModelConverter::emotional_event_to_json(event)?;
        let resulting_emotions_json = serde_json::to_value(resulting_emotions)?;

        // Calculate deltas (would need previous state for accurate calculation)
        let (_, _, _) = ModelConverter::calculate_state_metrics(resulting_emotions);
        let intensity_delta: f32 = resulting_emotions.values().sum();
        let valence_delta = 0.0; // Would calculate actual delta with previous state
        let arousal_delta = 0.0; // Would calculate actual delta with previous state

        // Save to database
        sqlx::query(
            r#"
            INSERT INTO emotional_events (
                agent_id, event_type, event_data, resulting_emotions,
                intensity_delta, valence_delta, arousal_delta
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(agent_id)
        .bind(&event_type)
        .bind(&event_data)
        .bind(&resulting_emotions_json)
        .bind(intensity_delta)
        .bind(valence_delta)
        .bind(arousal_delta)
        .execute(&self.postgres)
        .await?;

        // Add to Redis stream for real-time processing
        let stream_key = CacheKey::emotional_event_stream(agent_id);
        let event_json = serde_json::to_string(event)?;

        redis::cmd("XADD")
            .arg(&stream_key)
            .arg("MAXLEN")
            .arg("~")
            .arg(1000) // Keep last 1000 events
            .arg("*")
            .arg("event")
            .arg(&event_json)
            .arg("emotions")
            .arg(serde_json::to_string(resulting_emotions)?)
            .query_async::<()>(&mut self.redis.clone())
            .await
            .map_err(MemoryError::Redis)?;

        info!("Emotional event saved for agent {}", agent_id);
        Ok(())
    }

    /// Get emotional event history with optional limit
    pub async fn get_event_history(
        &self,
        agent_id: Uuid,
        limit: Option<i64>,
    ) -> MemoryResult<Vec<EmotionalEventRecord>> {
        debug!("Loading emotional event history for agent {}", agent_id);

        let limit = limit.unwrap_or(100).min(1000); // Cap at 1000 events

        let records = sqlx::query_as::<_, EmotionalEventRecord>(
            r#"
            SELECT id, agent_id, event_type, event_data, resulting_emotions,
                   context_factors, intensity_delta, valence_delta, arousal_delta, created_at
            FROM emotional_events
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
            "Loaded {} emotional events for agent {}",
            records.len(),
            agent_id
        );
        Ok(records)
    }

    /// Get emotional trends over time
    pub async fn get_emotional_trends(
        &self,
        agent_id: Uuid,
        hours: i32,
    ) -> MemoryResult<Vec<(chrono::DateTime<Utc>, f32, f32)>> {
        debug!(
            "Loading emotional trends for agent {} over {} hours",
            agent_id, hours
        );

        let records = sqlx::query_as::<_, (chrono::DateTime<Utc>, f32, f32)>(&format!(
            r#"
            SELECT created_at, valence_delta, arousal_delta
            FROM emotional_events
            WHERE agent_id = '{}' 
              AND created_at > NOW() - INTERVAL '{} hours'
            ORDER BY created_at ASC
            "#,
            agent_id, hours
        ))
        .fetch_all(&self.postgres)
        .await?;

        let trends = records;

        debug!(
            "Loaded {} emotional trend points for agent {}",
            trends.len(),
            agent_id
        );
        Ok(trends)
    }

    /// Clear old emotional events (cleanup)
    pub async fn cleanup_old_events(&self, days_to_keep: i32) -> MemoryResult<u64> {
        info!(
            "Cleaning up emotional events older than {} days",
            days_to_keep
        );

        let result = sqlx::query(&format!(
            "DELETE FROM emotional_events WHERE created_at < NOW() - INTERVAL '{} days'",
            days_to_keep
        ))
        .execute(&self.postgres)
        .await?;

        info!("Cleaned up {} old emotional events", result.rows_affected());
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalState};

    #[tokio::test]
    async fn test_emotional_state_round_trip() {
        // This test would need actual database connections
        // For now, just test the data conversion logic

        let mut state = EmotionalState::new();
        state.update_emotion(Emotion::Joy, 0.8);
        state.update_emotion(Emotion::Pride, 0.6);

        let (emotions_json, decay_json) =
            ModelConverter::emotional_state_to_json(&state.emotions, &state.decay_rates).unwrap();

        let (emotions_back, decay_back) =
            ModelConverter::json_to_emotional_state(&emotions_json, &decay_json).unwrap();

        assert_eq!(state.emotions, emotions_back);
        assert_eq!(state.decay_rates, decay_back);
    }

    #[tokio::test]
    async fn test_event_serialization() {
        let event = EmotionalEvent::TaskCompleted {
            success: true,
            difficulty: 0.7,
            time_taken: 120.0,
            expected_time: 100.0,
        };

        let (event_type, event_data) = ModelConverter::emotional_event_to_json(&event).unwrap();
        assert_eq!(event_type, "task_completed");

        let event_back = ModelConverter::json_to_emotional_event(&event_data).unwrap();

        // Verify the round-trip worked
        match (event, event_back) {
            (
                EmotionalEvent::TaskCompleted {
                    success: s1,
                    difficulty: d1,
                    ..
                },
                EmotionalEvent::TaskCompleted {
                    success: s2,
                    difficulty: d2,
                    ..
                },
            ) => {
                assert_eq!(s1, s2);
                assert!((d1 - d2).abs() < 0.001);
            }
            _ => panic!("Event types don't match after round-trip"),
        }
    }
}
