use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 22 discrete emotions from the OCC model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OCCEmotion {
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

impl OCCEmotion {
    /// Get the valence (positive/negative) of the emotion
    pub fn valence(&self) -> f32 {
        match self {
            OCCEmotion::Joy
            | OCCEmotion::Hope
            | OCCEmotion::Satisfaction
            | OCCEmotion::Relief
            | OCCEmotion::Pride
            | OCCEmotion::Admiration
            | OCCEmotion::Gratification
            | OCCEmotion::Gratitude
            | OCCEmotion::Love
            | OCCEmotion::HappyFor
            | OCCEmotion::Gloating => 1.0,

            OCCEmotion::Distress
            | OCCEmotion::Fear
            | OCCEmotion::Disappointment
            | OCCEmotion::FearConfirmed
            | OCCEmotion::Shame
            | OCCEmotion::Reproach
            | OCCEmotion::Remorse
            | OCCEmotion::Anger
            | OCCEmotion::Hate
            | OCCEmotion::Resentment
            | OCCEmotion::Pity => -1.0,
        }
    }

    /// Get the arousal level of the emotion
    pub fn arousal(&self) -> f32 {
        match self {
            OCCEmotion::Joy => 0.8,
            OCCEmotion::Distress => 0.7,
            OCCEmotion::Hope => 0.6,
            OCCEmotion::Fear => 0.9,
            OCCEmotion::Satisfaction => 0.7,
            OCCEmotion::Disappointment => 0.6,
            OCCEmotion::Relief => 0.5,
            OCCEmotion::FearConfirmed => 0.8,
            OCCEmotion::Pride => 0.6,
            OCCEmotion::Shame => 0.7,
            OCCEmotion::Admiration => 0.5,
            OCCEmotion::Reproach => 0.6,
            OCCEmotion::Gratification => 0.7,
            OCCEmotion::Remorse => 0.6,
            OCCEmotion::Gratitude => 0.5,
            OCCEmotion::Anger => 0.9,
            OCCEmotion::Love => 0.6,
            OCCEmotion::Hate => 0.8,
            OCCEmotion::HappyFor => 0.6,
            OCCEmotion::Resentment => 0.7,
            OCCEmotion::Gloating => 0.6,
            OCCEmotion::Pity => 0.4,
        }
    }
}

/// Emotional intensity on a 0.0 to 1.0 scale
pub type EmotionalIntensity = f32;

/// Current emotional state with intensities and timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCCEmotionalState {
    pub emotions: HashMap<OCCEmotion, EmotionalIntensity>,
    pub last_updated: DateTime<Utc>,
    pub decay_rates: HashMap<OCCEmotion, f32>,
}

impl Default for OCCEmotionalState {
    fn default() -> Self {
        Self::new()
    }
}

impl OCCEmotionalState {
    pub fn new() -> Self {
        Self {
            emotions: HashMap::new(),
            last_updated: Utc::now(),
            decay_rates: Self::default_decay_rates(),
        }
    }

    /// Default emotional decay rates per second
    /// Based on psychological research about emotion duration
    fn default_decay_rates() -> HashMap<OCCEmotion, f32> {
        use OCCEmotion::*;
        [
            // Fast-decaying emotions (high arousal, event-based)
            (Joy, 0.05),
            (Fear, 0.12),
            (Anger, 0.15),
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
            // Slow-decaying emotions (self-attribution, deep feelings)
            (Pride, 0.03),
            (Shame, 0.10), // Shame decays faster due to its negative impact
            (Gratification, 0.03),
            (Remorse, 0.09),
            (Love, 0.01), // Very persistent
            (Hate, 0.02), // Also persistent but slightly faster than love
        ]
        .iter()
        .cloned()
        .collect()
    }

    /// Update emotional intensity for a specific emotion
    pub fn update_emotion(&mut self, emotion: OCCEmotion, intensity: EmotionalIntensity) {
        self.emotions.insert(emotion, intensity.clamp(0.0, 1.0));
        self.last_updated = Utc::now();
    }

    /// Add to existing emotional intensity (useful for accumulating emotions)
    pub fn add_emotion(&mut self, emotion: OCCEmotion, intensity_delta: EmotionalIntensity) {
        let current = self.emotions.get(&emotion).unwrap_or(&0.0);
        let new_intensity = (current + intensity_delta).clamp(0.0, 1.0);
        self.update_emotion(emotion, new_intensity);
    }

    /// Apply temporal decay to all emotions
    pub fn apply_decay(&mut self, delta_seconds: f32) {
        let mut to_remove = Vec::new();

        for (emotion, intensity) in self.emotions.iter_mut() {
            let decay_rate = self.decay_rates.get(emotion).unwrap_or(&0.05);
            *intensity *= (1.0 - decay_rate).powf(delta_seconds);

            // Remove emotions below threshold
            if *intensity < 0.01 {
                to_remove.push(*emotion);
            }
        }

        // Remove low-intensity emotions
        for emotion in to_remove {
            self.emotions.remove(&emotion);
        }

        self.last_updated = Utc::now();
    }

    /// Get the dominant emotion (highest intensity)
    pub fn dominant_emotion(&self) -> Option<(OCCEmotion, EmotionalIntensity)> {
        self.emotions
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&emotion, &intensity)| (emotion, intensity))
    }

    /// Calculate overall emotional valence (-1.0 to 1.0)
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

    /// Calculate overall arousal level (0.0 to 1.0)
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

    /// Get emotional temperature (how volatile the agent is feeling)
    pub fn emotional_temperature(&self) -> f32 {
        let high_arousal_emotions = [
            OCCEmotion::Fear,
            OCCEmotion::Anger,
            OCCEmotion::Joy,
            OCCEmotion::FearConfirmed,
            OCCEmotion::Hate,
        ];

        let temperature: f32 = high_arousal_emotions
            .iter()
            .map(|emotion| self.emotions.get(emotion).unwrap_or(&0.0))
            .sum();

        temperature.min(1.0)
    }

    /// Check if agent is in a frustrated state (high anger + distress)
    pub fn is_frustrated(&self) -> bool {
        let anger = self.emotions.get(&OCCEmotion::Anger).unwrap_or(&0.0);
        let distress = self.emotions.get(&OCCEmotion::Distress).unwrap_or(&0.0);

        anger + distress > 0.6
    }

    /// Check if agent is in a confident state (high pride, low shame)
    pub fn is_confident(&self) -> bool {
        let pride = self.emotions.get(&OCCEmotion::Pride).unwrap_or(&0.0);
        let shame = self.emotions.get(&OCCEmotion::Shame).unwrap_or(&0.0);

        *pride > 0.5 && *shame < 0.3
    }

    /// Get emotional state summary for logging/debugging
    pub fn summary(&self) -> String {
        if self.emotions.is_empty() {
            return "Neutral".to_string();
        }

        let dominant = self
            .dominant_emotion()
            .map(|(emotion, intensity)| format!("{emotion:?}({intensity:.2})"))
            .unwrap_or_else(|| "None".to_string());

        format!(
            "Dominant: {}, Valence: {:.2}, Arousal: {:.2}, Temp: {:.2}",
            dominant,
            self.overall_valence(),
            self.overall_arousal(),
            self.emotional_temperature()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(OCCEmotion::Joy, 0.8, 0.8)]
    #[case(OCCEmotion::Anger, 1.0, 1.0)]
    #[case(OCCEmotion::Fear, -0.1, 0.0)] // Test clamping
    #[case(OCCEmotion::Pride, 1.5, 1.0)] // Test upper clamping
    fn test_emotional_state_update(
        #[case] emotion: OCCEmotion,
        #[case] input_intensity: f32,
        #[case] expected_intensity: f32,
    ) {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(emotion, input_intensity);

        assert_eq!(
            state.emotions.get(&emotion).unwrap_or(&0.0),
            &expected_intensity
        );
    }

    #[test]
    fn test_emotional_decay() {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(OCCEmotion::Joy, 1.0);

        // Apply 1 second of decay
        state.apply_decay(1.0);

        let joy_intensity = state.emotions.get(&OCCEmotion::Joy).unwrap_or(&0.0);
        assert!(*joy_intensity < 1.0);
        assert!(*joy_intensity > 0.9); // Should decay slightly
    }

    #[test]
    fn test_overall_valence() {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(OCCEmotion::Joy, 0.8);
        state.update_emotion(OCCEmotion::Anger, 0.4);

        let valence = state.overall_valence();
        assert!(valence > 0.0); // Should be positive due to higher joy
    }

    #[test]
    fn test_frustration_detection() {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(OCCEmotion::Anger, 0.4);
        state.update_emotion(OCCEmotion::Distress, 0.3);

        assert!(state.is_frustrated());
    }

    #[test]
    fn test_confidence_detection() {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(OCCEmotion::Pride, 0.6);
        state.update_emotion(OCCEmotion::Shame, 0.1);

        assert!(state.is_confident());
    }

    #[test]
    fn test_emotion_valence() {
        assert_eq!(OCCEmotion::Joy.valence(), 1.0);
        assert_eq!(OCCEmotion::Anger.valence(), -1.0);
        assert_eq!(OCCEmotion::Pride.valence(), 1.0);
        assert_eq!(OCCEmotion::Shame.valence(), -1.0);
    }
}
