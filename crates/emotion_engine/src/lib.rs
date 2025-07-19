//! # Emotion Engine
//!
//! A sophisticated emotional intelligence system for AI agents based on the OCC
//! (Ortony, Clore, and Collins) model of emotions. This crate provides:
//!
//! - 22 discrete emotions with proper intensity tracking
//! - Cognitive appraisal functions that convert events to emotional responses
//! - Temporal decay systems for realistic emotion evolution
//! - Personality-based modulation of emotional responses
//!
//! ## Quick Start
//!
//! ```rust
//! use emotion_engine::{OCCEmotionalState, AppraisalEngine, EmotionalEvent, PersonalityModifiers};
//!
//! // Create an emotional state
//! let mut emotional_state = OCCEmotionalState::new();
//!
//! // Create an appraisal engine
//! let mut appraisal_engine = AppraisalEngine::new(
//!     vec!["coding".to_string(), "helping_users".to_string()],
//!     PersonalityModifiers::default(),
//! );
//!
//! // Process an emotional event
//! let event = EmotionalEvent::TaskCompleted {
//!     difficulty: 0.7,
//!     success: true,
//!     time_taken: 120.0,
//!     expected_time: 100.0,
//!     was_retry: false,
//! };
//!
//! let emotional_responses = appraisal_engine.appraise_event(&event);
//!
//! // Apply emotions to state
//! for (emotion, intensity) in emotional_responses {
//!     emotional_state.update_emotion(emotion, intensity);
//! }
//!
//! // Check emotional state
//! println!("Current state: {}", emotional_state.summary());
//! ```

pub mod appraisal;
pub mod occ;

// Re-export main types for convenience
pub use appraisal::{
    AppraisalContext, AppraisalEngine, EmotionalEvent, PeerInteractionType, PersonalityModifiers,
};
pub use occ::{EmotionalIntensity, OCCEmotion, OCCEmotionalState};

/// PAD (Pleasure-Arousal-Dominance) emotional space representation
/// Useful for mapping OCC emotions to continuous emotional dimensions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PADState {
    /// Pleasure: how positive/negative the agent feels (-1.0 to 1.0)
    pub pleasure: f32,

    /// Arousal: how energized/calm the agent is (0.0 to 1.0)  
    pub arousal: f32,

    /// Dominance: how in-control/submissive the agent feels (0.0 to 1.0)
    pub dominance: f32,
}

impl PADState {
    /// Create a neutral PAD state
    pub fn neutral() -> Self {
        Self {
            pleasure: 0.0,
            arousal: 0.5,
            dominance: 0.5,
        }
    }

    /// Convert from OCC emotional state to PAD space
    pub fn from_occ_state(occ_state: &OCCEmotionalState) -> Self {
        let pleasure = occ_state.overall_valence();
        let arousal = occ_state.overall_arousal();

        // Calculate dominance based on confidence and control emotions
        let dominance = {
            let pride = occ_state.emotions.get(&OCCEmotion::Pride).unwrap_or(&0.0);
            let shame = occ_state.emotions.get(&OCCEmotion::Shame).unwrap_or(&0.0);
            let fear = occ_state.emotions.get(&OCCEmotion::Fear).unwrap_or(&0.0);
            let anger = occ_state.emotions.get(&OCCEmotion::Anger).unwrap_or(&0.0);

            // Pride and anger increase dominance, shame and fear decrease it
            let dominance_raw = pride + (anger * 0.3) - shame - (fear * 0.5);
            (dominance_raw + 0.5).clamp(0.0, 1.0)
        };

        Self {
            pleasure,
            arousal,
            dominance,
        }
    }

    /// Get emotional quadrant based on pleasure and arousal
    pub fn emotional_quadrant(&self) -> EmotionalQuadrant {
        match (self.pleasure > 0.0, self.arousal > 0.5) {
            (true, true) => EmotionalQuadrant::Excited, // High pleasure, high arousal
            (true, false) => EmotionalQuadrant::Content, // High pleasure, low arousal
            (false, true) => EmotionalQuadrant::Agitated, // Low pleasure, high arousal
            (false, false) => EmotionalQuadrant::Depressed, // Low pleasure, low arousal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionalQuadrant {
    Excited,   // Positive valence, high arousal (joy, excitement)
    Content,   // Positive valence, low arousal (satisfaction, calm)
    Agitated,  // Negative valence, high arousal (anger, fear)
    Depressed, // Negative valence, low arousal (sadness, disappointment)
}

/// Utility functions for emotional analysis
pub mod utils {
    use super::*;

    /// Calculate emotional distance between two states (0.0 to 1.0)
    pub fn emotional_distance(state1: &OCCEmotionalState, state2: &OCCEmotionalState) -> f32 {
        let all_emotions = [
            OCCEmotion::Joy,
            OCCEmotion::Distress,
            OCCEmotion::Hope,
            OCCEmotion::Fear,
            OCCEmotion::Satisfaction,
            OCCEmotion::Disappointment,
            OCCEmotion::Relief,
            OCCEmotion::FearConfirmed,
            OCCEmotion::Pride,
            OCCEmotion::Shame,
            OCCEmotion::Admiration,
            OCCEmotion::Reproach,
            OCCEmotion::Gratification,
            OCCEmotion::Remorse,
            OCCEmotion::Gratitude,
            OCCEmotion::Anger,
            OCCEmotion::Love,
            OCCEmotion::Hate,
            OCCEmotion::HappyFor,
            OCCEmotion::Resentment,
            OCCEmotion::Gloating,
            OCCEmotion::Pity,
        ];

        let sum_squared_diffs: f32 = all_emotions
            .iter()
            .map(|emotion| {
                let intensity1 = state1.emotions.get(emotion).unwrap_or(&0.0);
                let intensity2 = state2.emotions.get(emotion).unwrap_or(&0.0);
                (intensity1 - intensity2).powi(2)
            })
            .sum();

        (sum_squared_diffs / all_emotions.len() as f32).sqrt()
    }

    /// Classify emotional state into a simple category
    pub fn classify_emotional_state(state: &OCCEmotionalState) -> String {
        if state.emotions.is_empty() {
            return "Neutral".to_string();
        }

        let valence = state.overall_valence();
        let arousal = state.overall_arousal();
        let temperature = state.emotional_temperature();

        match (
            valence > 0.3,
            valence < -0.3,
            arousal > 0.6,
            temperature > 0.5,
        ) {
            (true, _, true, _) => "Excited/Enthusiastic".to_string(),
            (true, _, false, _) => "Content/Satisfied".to_string(),
            (false, true, true, true) => "Angry/Frustrated".to_string(),
            (false, true, true, false) => "Anxious/Worried".to_string(),
            (false, true, false, _) => "Sad/Disappointed".to_string(),
            _ => "Mixed/Complex".to_string(),
        }
    }

    /// Check if two emotional states are similar (within threshold)
    pub fn are_emotionally_similar(
        state1: &OCCEmotionalState,
        state2: &OCCEmotionalState,
        threshold: f32,
    ) -> bool {
        emotional_distance(state1, state2) < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_conversion() {
        let mut occ_state = OCCEmotionalState::new();
        occ_state.update_emotion(OCCEmotion::Joy, 0.8);
        occ_state.update_emotion(OCCEmotion::Pride, 0.6);

        let pad_state = PADState::from_occ_state(&occ_state);

        assert!(pad_state.pleasure > 0.0);
        assert!(pad_state.arousal > 0.0);
        assert!(pad_state.dominance > 0.5);
    }

    #[test]
    fn test_emotional_quadrant() {
        let excited_state = PADState {
            pleasure: 0.7,
            arousal: 0.8,
            dominance: 0.6,
        };

        assert_eq!(
            excited_state.emotional_quadrant(),
            EmotionalQuadrant::Excited
        );

        let content_state = PADState {
            pleasure: 0.5,
            arousal: 0.3,
            dominance: 0.7,
        };

        assert_eq!(
            content_state.emotional_quadrant(),
            EmotionalQuadrant::Content
        );
    }

    #[test]
    fn test_emotional_distance() {
        let mut state1 = OCCEmotionalState::new();
        state1.update_emotion(OCCEmotion::Joy, 0.8);

        let mut state2 = OCCEmotionalState::new();
        state2.update_emotion(OCCEmotion::Joy, 0.6);

        let distance = utils::emotional_distance(&state1, &state2);
        assert!(distance > 0.0);
        assert!(distance < 1.0);

        // Same states should have zero distance
        let distance_same = utils::emotional_distance(&state1, &state1);
        assert_eq!(distance_same, 0.0);
    }

    #[test]
    fn test_emotional_classification() {
        let mut state = OCCEmotionalState::new();
        state.update_emotion(OCCEmotion::Joy, 0.8);
        state.update_emotion(OCCEmotion::Pride, 0.6);

        let classification = utils::classify_emotional_state(&state);
        assert!(classification.contains("Excited") || classification.contains("Content"));
    }
}
