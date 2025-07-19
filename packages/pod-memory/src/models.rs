use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalIntensity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Database model for emotional state snapshots
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmotionalStateRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub emotions: Value, // JSON representation of HashMap<Emotion, EmotionalIntensity>
    pub decay_rates: Value, // JSON representation of HashMap<Emotion, f32>
    pub overall_valence: f32,
    pub overall_arousal: f32,
    pub dominant_emotion: Option<String>,
    pub dominant_intensity: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for emotional event history
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmotionalEventRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub event_type: String,
    pub event_data: Value,              // JSON representation of EmotionalEvent
    pub resulting_emotions: Value, // JSON representation of HashMap<Emotion, EmotionalIntensity>
    pub context_factors: Option<Value>, // Additional context that influenced the event
    pub intensity_delta: f32,      // Overall change in emotional intensity
    pub valence_delta: f32,        // Change in emotional valence
    pub arousal_delta: f32,        // Change in emotional arousal
    pub created_at: DateTime<Utc>,
}

/// Database model for conversation messages
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub role: String, // "user", "assistant", "system", etc.
    pub content: String,
    pub metadata: Option<Value>,          // Additional message metadata
    pub emotional_context: Option<Value>, // Emotional state when message was sent/received
    pub tokens_used: Option<i32>,         // Token count if applicable
    pub created_at: DateTime<Utc>,
}

/// Database model for general memory storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MemoryRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub memory_type: String, // "goal", "preference", "fact", "skill", etc.
    pub key: String,
    pub data: Value,
    pub importance: f32,   // 0.0 to 1.0 - for memory prioritization
    pub access_count: i32, // How often this memory has been accessed
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>, // TTL for temporary memories
}

/// Database model for agent session tracking
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentSession {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub session_start: DateTime<Utc>,
    pub session_end: Option<DateTime<Utc>>,
    pub initial_emotional_state: Option<Value>,
    pub final_emotional_state: Option<Value>,
    pub total_messages: i32,
    pub total_events: i32,
    pub session_metadata: Option<Value>,
}

/// Redis cache key builder
pub struct CacheKey;

impl CacheKey {
    pub fn emotional_state(agent_id: Uuid) -> String {
        format!("emotion:state:{}", agent_id)
    }

    pub fn conversation_cache(agent_id: Uuid) -> String {
        format!("conversation:cache:{}", agent_id)
    }

    pub fn memory_cache(agent_id: Uuid, memory_type: &str, key: &str) -> String {
        format!("memory:{}:{}:{}", agent_id, memory_type, key)
    }

    pub fn session_lock(agent_id: Uuid) -> String {
        format!("session:lock:{}", agent_id)
    }

    pub fn emotional_event_stream(agent_id: Uuid) -> String {
        format!("emotion:events:{}", agent_id)
    }
}

/// Helper for converting between domain types and database models
pub struct ModelConverter;

impl ModelConverter {
    /// Convert EmotionalState to JSON for database storage
    pub fn emotional_state_to_json(
        emotions: &HashMap<Emotion, EmotionalIntensity>,
        decay_rates: &HashMap<Emotion, f32>,
    ) -> serde_json::Result<(Value, Value)> {
        let emotions_json = serde_json::to_value(emotions)?;
        let decay_rates_json = serde_json::to_value(decay_rates)?;
        Ok((emotions_json, decay_rates_json))
    }

    /// Convert JSON back to EmotionalState components
    pub fn json_to_emotional_state(
        emotions_json: &Value,
        decay_rates_json: &Value,
    ) -> serde_json::Result<(HashMap<Emotion, EmotionalIntensity>, HashMap<Emotion, f32>)> {
        let emotions: HashMap<Emotion, EmotionalIntensity> =
            serde_json::from_value(emotions_json.clone())?;
        let decay_rates: HashMap<Emotion, f32> = serde_json::from_value(decay_rates_json.clone())?;
        Ok((emotions, decay_rates))
    }

    /// Convert EmotionalEvent to JSON for database storage
    pub fn emotional_event_to_json(event: &EmotionalEvent) -> serde_json::Result<(String, Value)> {
        let event_type = match event {
            EmotionalEvent::TaskCompleted { .. } => "task_completed",
            EmotionalEvent::UserFeedback { .. } => "user_feedback",
            EmotionalEvent::ToolCallFailed { .. } => "tool_call_failed",
            EmotionalEvent::GoalProgress { .. } => "goal_progress",
            EmotionalEvent::SocialInteraction { .. } => "social_interaction",
            EmotionalEvent::UnexpectedEvent { .. } => "unexpected_event",
            EmotionalEvent::MemoryRecall { .. } => "memory_recall",
            EmotionalEvent::Custom { event_type, .. } => event_type,
        };

        let event_data = serde_json::to_value(event)?;
        Ok((event_type.to_string(), event_data))
    }

    /// Convert JSON back to EmotionalEvent
    pub fn json_to_emotional_event(event_data: &Value) -> serde_json::Result<EmotionalEvent> {
        serde_json::from_value(event_data.clone())
    }

    /// Calculate emotional state metrics for database storage
    pub fn calculate_state_metrics(
        emotions: &HashMap<Emotion, EmotionalIntensity>,
    ) -> (f32, f32, Option<(String, f32)>) {
        if emotions.is_empty() {
            return (0.0, 0.0, None);
        }

        // Calculate overall valence
        let valence = {
            let weighted_sum: f32 = emotions
                .iter()
                .map(|(emotion, intensity)| emotion.valence() * intensity)
                .sum();
            let total_intensity: f32 = emotions.values().sum();
            if total_intensity > 0.0 {
                weighted_sum / total_intensity
            } else {
                0.0
            }
        };

        // Calculate overall arousal
        let arousal = {
            let weighted_sum: f32 = emotions
                .iter()
                .map(|(emotion, intensity)| emotion.arousal() * intensity)
                .sum();
            let total_intensity: f32 = emotions.values().sum();
            if total_intensity > 0.0 {
                weighted_sum / total_intensity
            } else {
                0.0
            }
        };

        // Find dominant emotion
        let dominant = emotions
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(emotion, intensity)| (format!("{:?}", emotion), *intensity));

        (valence, arousal, dominant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use banshee_core::emotion::Emotion;

    #[test]
    fn test_cache_key_generation() {
        let agent_id = Uuid::new_v4();

        let state_key = CacheKey::emotional_state(agent_id);
        assert!(state_key.contains(&agent_id.to_string()));
        assert!(state_key.starts_with("emotion:state:"));

        let memory_key = CacheKey::memory_cache(agent_id, "goals", "primary_objective");
        assert!(memory_key.contains(&agent_id.to_string()));
        assert!(memory_key.contains("goals"));
        assert!(memory_key.contains("primary_objective"));
    }

    #[test]
    fn test_emotional_state_conversion() {
        let mut emotions = HashMap::new();
        emotions.insert(Emotion::Joy, 0.8);
        emotions.insert(Emotion::Pride, 0.6);

        let mut decay_rates = HashMap::new();
        decay_rates.insert(Emotion::Joy, 0.05);
        decay_rates.insert(Emotion::Pride, 0.03);

        let (emotions_json, decay_json) =
            ModelConverter::emotional_state_to_json(&emotions, &decay_rates).unwrap();

        let (emotions_back, decay_back) =
            ModelConverter::json_to_emotional_state(&emotions_json, &decay_json).unwrap();

        assert_eq!(emotions, emotions_back);
        assert_eq!(decay_rates, decay_back);
    }

    #[test]
    fn test_state_metrics_calculation() {
        let mut emotions = HashMap::new();
        emotions.insert(Emotion::Joy, 0.8); // valence: 1.0, arousal: 0.8
        emotions.insert(Emotion::Anger, 0.4); // valence: -1.0, arousal: 0.9

        let (valence, arousal, dominant) = ModelConverter::calculate_state_metrics(&emotions);

        // Weighted valence: (1.0 * 0.8 + (-1.0) * 0.4) / (0.8 + 0.4) = 0.4 / 1.2 = 0.33...
        assert!((valence - 0.333).abs() < 0.01);

        // Weighted arousal: (0.8 * 0.8 + 0.9 * 0.4) / (0.8 + 0.4) = 1.0 / 1.2 = 0.83...
        assert!((arousal - 0.833).abs() < 0.01);

        // Dominant emotion should be Joy (higher intensity)
        assert!(dominant.is_some());
        let (emotion_name, intensity) = dominant.unwrap();
        assert_eq!(emotion_name, "Joy");
        assert_eq!(intensity, 0.8);
    }
}
