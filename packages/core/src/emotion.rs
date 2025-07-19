use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core emotion types following the OCC model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Emotion {
    // Event-based emotions (consequences of events)
    Joy,
    Distress,
    Hope,
    Fear,
    Satisfaction,
    Disappointment,
    Relief,
    FearConfirmed,

    // Attribution emotions (actions of agents)
    Pride,
    Shame,
    Admiration,
    Reproach,
    Gratification,
    Remorse,
    Gratitude,
    Anger,

    // Attraction emotions (aspects of objects)
    Love,
    Hate,

    // Well-being emotions (fortunes of others)
    HappyFor,
    Resentment,
    Gloating,
    Pity,
}

/// Emotional intensity (0.0 to 1.0)
pub type EmotionalIntensity = f32;

/// Current emotional state of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Current emotions and their intensities
    pub emotions: HashMap<Emotion, EmotionalIntensity>,

    /// When this state was last updated
    pub last_updated: DateTime<Utc>,

    /// Emotional decay rates for each emotion
    pub decay_rates: HashMap<Emotion, f32>,
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionalState {
    pub fn new() -> Self {
        Self {
            emotions: HashMap::new(),
            last_updated: Utc::now(),
            decay_rates: Self::default_decay_rates(),
        }
    }

    /// Get default decay rates for emotions
    fn default_decay_rates() -> HashMap<Emotion, f32> {
        use Emotion::*;
        [
            // Fast-decaying emotions (high arousal)
            (Fear, 0.12),
            (Anger, 0.15),
            (Joy, 0.05),
            (Relief, 0.08),
            (FearConfirmed, 0.10),
            // Medium-decaying emotions
            (Distress, 0.08),
            (Hope, 0.06),
            (Satisfaction, 0.04),
            (Disappointment, 0.07),
            (Admiration, 0.04),
            (Reproach, 0.08),
            (Gratitude, 0.03),
            (HappyFor, 0.05),
            (Resentment, 0.06),
            (Gloating, 0.07),
            (Pity, 0.04),
            // Slow-decaying emotions (deep feelings)
            (Pride, 0.03),
            (Shame, 0.10),
            (Gratification, 0.03),
            (Remorse, 0.09),
            (Love, 0.01),
            (Hate, 0.02),
        ]
        .iter()
        .cloned()
        .collect()
    }

    /// Update emotion intensity
    pub fn update_emotion(&mut self, emotion: Emotion, intensity: EmotionalIntensity) {
        self.emotions.insert(emotion, intensity.clamp(0.0, 1.0));
        self.last_updated = Utc::now();
    }

    /// Apply temporal decay to all emotions
    pub fn apply_decay(&mut self, delta_seconds: f32) {
        let mut to_remove = Vec::new();

        for (emotion, intensity) in self.emotions.iter_mut() {
            let decay_rate = self.decay_rates.get(emotion).unwrap_or(&0.05);
            *intensity *= (1.0 - decay_rate).powf(delta_seconds);

            if *intensity < 0.01 {
                to_remove.push(*emotion);
            }
        }

        for emotion in to_remove {
            self.emotions.remove(&emotion);
        }

        self.last_updated = Utc::now();
    }

    /// Get overall emotional valence (-1.0 to 1.0)
    pub fn overall_valence(&self) -> f32 {
        if self.emotions.is_empty() {
            return 0.0;
        }

        let weighted_sum: f32 = self
            .emotions
            .iter()
            .map(|(emotion, intensity)| emotion.valence() * intensity)
            .sum();

        let total_intensity: f32 = self.emotions.values().sum();

        if total_intensity > 0.0 {
            weighted_sum / total_intensity
        } else {
            0.0
        }
    }

    /// Get overall arousal level (0.0 to 1.0)
    pub fn overall_arousal(&self) -> f32 {
        if self.emotions.is_empty() {
            return 0.0;
        }

        let weighted_sum: f32 = self
            .emotions
            .iter()
            .map(|(emotion, intensity)| emotion.arousal() * intensity)
            .sum();

        let total_intensity: f32 = self.emotions.values().sum();

        if total_intensity > 0.0 {
            weighted_sum / total_intensity
        } else {
            0.0
        }
    }
}

impl Emotion {
    /// Get the valence (positive/negative) of the emotion
    pub fn valence(self) -> f32 {
        match self {
            Emotion::Joy
            | Emotion::Hope
            | Emotion::Satisfaction
            | Emotion::Relief
            | Emotion::Pride
            | Emotion::Admiration
            | Emotion::Gratification
            | Emotion::Gratitude
            | Emotion::Love
            | Emotion::HappyFor
            | Emotion::Gloating => 1.0,

            Emotion::Distress
            | Emotion::Fear
            | Emotion::Disappointment
            | Emotion::FearConfirmed
            | Emotion::Shame
            | Emotion::Reproach
            | Emotion::Remorse
            | Emotion::Anger
            | Emotion::Hate
            | Emotion::Resentment
            | Emotion::Pity => -1.0,
        }
    }

    /// Get the arousal level of the emotion
    pub fn arousal(self) -> f32 {
        match self {
            Emotion::Joy => 0.8,
            Emotion::Distress => 0.7,
            Emotion::Hope => 0.6,
            Emotion::Fear => 0.9,
            Emotion::Satisfaction => 0.7,
            Emotion::Disappointment => 0.6,
            Emotion::Relief => 0.5,
            Emotion::FearConfirmed => 0.8,
            Emotion::Pride => 0.6,
            Emotion::Shame => 0.7,
            Emotion::Admiration => 0.5,
            Emotion::Reproach => 0.6,
            Emotion::Gratification => 0.7,
            Emotion::Remorse => 0.6,
            Emotion::Gratitude => 0.5,
            Emotion::Anger => 0.9,
            Emotion::Love => 0.6,
            Emotion::Hate => 0.8,
            Emotion::HappyFor => 0.6,
            Emotion::Resentment => 0.7,
            Emotion::Gloating => 0.6,
            Emotion::Pity => 0.4,
        }
    }
}

/// Events that can trigger emotional responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmotionalEvent {
    TaskCompleted {
        success: bool,
        difficulty: f32,
        time_taken: f32,
        expected_time: f32,
    },

    UserFeedback {
        sentiment: f32, // -1.0 to 1.0
        specificity: f32,
        is_constructive: bool,
    },

    ToolCallFailed {
        attempts: u32,
        error_severity: f32,
        is_critical: bool,
    },

    GoalProgress {
        progress_delta: f32,
        goal_importance: f32,
        is_milestone: bool,
    },

    SocialInteraction {
        interaction_type: SocialInteractionType,
        outcome: f32,
        peer_status: f32,
    },

    UnexpectedEvent {
        surprise_level: f32,
        positive_outcome: bool,
        context: String,
    },

    MemoryRecall {
        memory_relevance: f32,
        emotional_content: f32,
        memory_age: f32,
    },

    Custom {
        event_type: String,
        data: HashMap<String, f32>,
    },
}

/// Types of social interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialInteractionType {
    Collaboration,
    Competition,
    Recognition,
    Criticism,
    Support,
    Conflict,
}

/// Emotional appraisal result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalAppraisal {
    /// Event that was appraised
    pub event: EmotionalEvent,

    /// Resulting emotional changes
    pub emotional_changes: HashMap<Emotion, EmotionalIntensity>,

    /// Appraisal timestamp
    pub timestamp: DateTime<Utc>,

    /// Context factors that influenced appraisal
    pub context_factors: Vec<String>,
}

/// Trait for emotional appraisal engines
pub trait EmotionalAppraisalEngine: Send + Sync {
    /// Appraise an event and return emotional changes
    fn appraise(
        &self,
        event: &EmotionalEvent,
        current_state: &EmotionalState,
    ) -> EmotionalAppraisal;

    /// Get personality factors that influence appraisal
    fn personality_factors(&self) -> &HashMap<String, f32>;

    /// Update personality factors
    fn update_personality(&mut self, factors: HashMap<String, f32>);
}
