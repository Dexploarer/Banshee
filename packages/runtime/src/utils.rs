//! Utility functions and extensions for runtime

use emotional_agents_core::*;
use std::collections::HashMap;

/// Extension trait for EmotionalState
pub trait EmotionalStateExt {
    /// Get current emotions as a hashmap of strings to intensities
    fn current_emotions(&self) -> HashMap<String, f32>;

    /// Get the dominant emotion
    fn dominant_emotion(&self) -> (String, f32);

    /// Apply an emotional event
    fn apply_event(&mut self, event: EmotionalEvent);
}

impl EmotionalStateExt for EmotionalState {
    fn current_emotions(&self) -> HashMap<String, f32> {
        self.emotions
            .iter()
            .map(|(emotion, intensity)| (format!("{:?}", emotion).to_lowercase(), *intensity))
            .collect()
    }

    fn dominant_emotion(&self) -> (String, f32) {
        self.emotions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(emotion, intensity)| (format!("{:?}", emotion).to_lowercase(), *intensity))
            .unwrap_or(("neutral".to_string(), 0.0))
    }

    fn apply_event(&mut self, event: EmotionalEvent) {
        // Simple implementation - would be more sophisticated in reality
        match event {
            EmotionalEvent::UserFeedback { sentiment, .. } => {
                if sentiment > 0.0 {
                    self.update_emotion(Emotion::Joy, sentiment);
                } else {
                    self.update_emotion(Emotion::Distress, -sentiment);
                }
            }
            EmotionalEvent::TaskCompleted { success, .. } => {
                if success {
                    self.update_emotion(Emotion::Satisfaction, 0.7);
                } else {
                    self.update_emotion(Emotion::Disappointment, 0.5);
                }
            }
            _ => {}
        }
    }
}

/// Extension trait for DecisionRecord
pub trait DecisionRecordExt {
    /// Get the selected option
    fn selected_option(&self) -> &crate::runtime::DecisionOption;
}

impl DecisionRecordExt for crate::runtime::DecisionRecord {
    fn selected_option(&self) -> &crate::runtime::DecisionOption {
        &self.options[self.selected]
    }
}
