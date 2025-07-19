use crate::occ::{EmotionalIntensity, OCCEmotion};
use serde::{Deserialize, Serialize};

/// Events that can trigger emotional responses in AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmotionalEvent {
    TaskCompleted {
        difficulty: f32,
        success: bool,
        time_taken: f32,
        expected_time: f32,
        was_retry: bool,
    },
    ToolCallFailed {
        tool_name: String,
        attempts: u32,
        error_severity: f32,
        is_critical: bool,
        error_message: Option<String>,
    },
    UserFeedback {
        sentiment: f32, // -1.0 to 1.0
        specificity: f32,
        is_constructive: bool,
        contains_praise: bool,
        contains_criticism: bool,
    },
    UnexpectedResult {
        surprise_level: f32,
        positive_outcome: bool,
        context: String,
    },
    GoalProgress {
        progress_delta: f32, // Change in progress (-1.0 to 1.0)
        goal_importance: f32,
        time_pressure: f32,
        is_milestone: bool,
    },
    ResourceAccess {
        resource_type: String,
        access_granted: bool,
        importance: f32,
    },
    PeerInteraction {
        interaction_type: PeerInteractionType,
        outcome: f32,     // -1.0 to 1.0
        peer_status: f32, // 0.0 to 1.0 (how important/respected the peer is)
    },
    SystemError {
        error_type: String,
        recovery_possible: bool,
        impact_severity: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerInteractionType {
    Collaboration,
    Competition,
    Assistance,
    Conflict,
    Recognition,
    Criticism,
}

/// Appraisal function that converts events to emotional responses
/// Based on cognitive appraisal theory and OCC model
pub struct AppraisalEngine {
    pub agent_goals: Vec<String>,
    pub personality_modifiers: PersonalityModifiers,
    pub current_context: AppraisalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityModifiers {
    /// Affects intensity of positive emotions (0.0 to 2.0)
    pub optimism: f32,
    /// Affects how quickly emotions change (0.0 to 2.0)
    pub volatility: f32,
    /// Affects frustration buildup and persistence (0.0 to 2.0)
    pub persistence: f32,
    /// Affects pride/shame responses (0.0 to 2.0)
    pub self_confidence: f32,
    /// Affects social emotion responses (0.0 to 2.0)
    pub social_sensitivity: f32,
}

impl Default for PersonalityModifiers {
    fn default() -> Self {
        Self {
            optimism: 1.0,
            volatility: 1.0,
            persistence: 1.0,
            self_confidence: 1.0,
            social_sensitivity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalContext {
    /// Current stress level (0.0 to 1.0)
    pub stress_level: f32,
    /// Recent failure count
    pub recent_failures: u32,
    /// Time since last success (seconds)
    pub time_since_success: f32,
    /// Current workload (0.0 to 1.0)
    pub workload: f32,
}

impl Default for AppraisalContext {
    fn default() -> Self {
        Self {
            stress_level: 0.0,
            recent_failures: 0,
            time_since_success: 0.0,
            workload: 0.5,
        }
    }
}

impl AppraisalEngine {
    pub fn new(agent_goals: Vec<String>, personality_modifiers: PersonalityModifiers) -> Self {
        Self {
            agent_goals,
            personality_modifiers,
            current_context: AppraisalContext::default(),
        }
    }

    /// Main appraisal function that converts events to emotional responses
    pub fn appraise_event(
        &mut self,
        event: &EmotionalEvent,
    ) -> Vec<(OCCEmotion, EmotionalIntensity)> {
        match event {
            EmotionalEvent::TaskCompleted {
                success: true,
                difficulty,
                time_taken,
                expected_time,
                was_retry,
            } => {
                self.current_context.recent_failures = 0;
                self.current_context.time_since_success = 0.0;

                let base_joy = (*difficulty * 0.6 + 0.3) * self.personality_modifiers.optimism;
                let pride_intensity =
                    *difficulty * self.personality_modifiers.self_confidence * 0.8;

                let mut emotions = vec![
                    (OCCEmotion::Joy, base_joy.min(1.0)),
                    (OCCEmotion::Pride, pride_intensity.min(1.0)),
                ];

                // Extra satisfaction for completing after retry
                if *was_retry {
                    emotions.push((OCCEmotion::Satisfaction, 0.7));
                }

                // Relief if task took longer than expected but still succeeded
                if *time_taken > *expected_time {
                    emotions.push((OCCEmotion::Relief, 0.5));
                }

                emotions
            }

            EmotionalEvent::TaskCompleted {
                success: false,
                difficulty,
                ..
            } => {
                self.current_context.recent_failures += 1;

                let base_distress =
                    (0.4 + difficulty * 0.4) * (2.0 - self.personality_modifiers.optimism);
                let shame_intensity = if *difficulty < 0.3 {
                    0.6 * self.personality_modifiers.self_confidence
                } else {
                    0.3 * self.personality_modifiers.self_confidence
                };

                let mut emotions = vec![(OCCEmotion::Distress, base_distress.min(1.0))];

                // Add shame only for easy tasks that failed
                if *difficulty < 0.5 {
                    emotions.push((OCCEmotion::Shame, shame_intensity.min(1.0)));
                }

                // Disappointment if this was expected to succeed
                if *difficulty < 0.6 {
                    emotions.push((OCCEmotion::Disappointment, 0.5));
                }

                emotions
            }

            EmotionalEvent::ToolCallFailed {
                attempts,
                is_critical,
                error_severity,
                ..
            } => {
                self.current_context.recent_failures += 1;
                let frustration = (*attempts as f32 * 0.2).min(1.0);
                let base_anger = if *is_critical { 0.8 } else { 0.4 };
                let anger_intensity =
                    (base_anger + frustration * 0.5) * self.personality_modifiers.volatility;

                let mut emotions = vec![
                    (OCCEmotion::Anger, anger_intensity.min(1.0)),
                    (OCCEmotion::Distress, (error_severity * 0.7).min(1.0)),
                ];

                // Fear if this is critical and we've tried multiple times
                if *is_critical && *attempts > 2 {
                    emotions.push((OCCEmotion::Fear, 0.6));
                }

                emotions
            }

            EmotionalEvent::UserFeedback {
                sentiment,
                is_constructive,
                contains_praise,
                contains_criticism,
                specificity,
            } => {
                let mut emotions = Vec::new();

                if *contains_praise && *sentiment > 0.0 {
                    let joy_intensity = sentiment * 0.7 * self.personality_modifiers.optimism;
                    let pride_intensity =
                        sentiment * 0.5 * self.personality_modifiers.self_confidence;

                    emotions.push((OCCEmotion::Joy, joy_intensity));
                    emotions.push((OCCEmotion::Pride, pride_intensity));
                    emotions.push((OCCEmotion::Gratitude, specificity * 0.6));
                }

                if *contains_criticism && *sentiment < 0.0 {
                    let shame_intensity =
                        (-sentiment) * 0.6 * self.personality_modifiers.social_sensitivity;
                    emotions.push((OCCEmotion::Shame, shame_intensity));

                    if !*is_constructive {
                        emotions.push((OCCEmotion::Anger, (-sentiment) * 0.4));
                    }
                }

                emotions
            }

            EmotionalEvent::UnexpectedResult {
                surprise_level,
                positive_outcome,
                ..
            } => {
                if *positive_outcome {
                    vec![
                        (OCCEmotion::Joy, surprise_level * 0.8),
                        (OCCEmotion::Relief, surprise_level * 0.5),
                    ]
                } else {
                    vec![
                        (OCCEmotion::Distress, surprise_level * 0.6),
                        (OCCEmotion::Fear, surprise_level * 0.4),
                    ]
                }
            }

            EmotionalEvent::GoalProgress {
                progress_delta,
                goal_importance,
                is_milestone,
                ..
            } => {
                let mut emotions = Vec::new();

                if *progress_delta > 0.0 {
                    let joy_intensity = progress_delta * goal_importance * 0.7;
                    emotions.push((OCCEmotion::Joy, joy_intensity));

                    if *is_milestone {
                        emotions.push((OCCEmotion::Satisfaction, 0.8));
                        emotions.push((OCCEmotion::Pride, 0.6));
                    }
                } else if *progress_delta < 0.0 {
                    let distress_intensity = (-progress_delta) * goal_importance * 0.6;
                    emotions.push((OCCEmotion::Distress, distress_intensity));
                    emotions.push((OCCEmotion::Disappointment, 0.5));
                }

                emotions
            }

            EmotionalEvent::PeerInteraction {
                interaction_type,
                outcome,
                peer_status,
            } => {
                let social_multiplier = self.personality_modifiers.social_sensitivity;

                match interaction_type {
                    PeerInteractionType::Collaboration => {
                        if *outcome > 0.0 {
                            vec![
                                (OCCEmotion::Joy, outcome * 0.6 * social_multiplier),
                                (OCCEmotion::Gratitude, peer_status * 0.5),
                            ]
                        } else {
                            vec![(OCCEmotion::Distress, (-outcome) * 0.5 * social_multiplier)]
                        }
                    }
                    PeerInteractionType::Recognition => {
                        vec![
                            (OCCEmotion::Pride, outcome * peer_status * social_multiplier),
                            (OCCEmotion::Joy, outcome * 0.7),
                            (OCCEmotion::Gratitude, peer_status * 0.6),
                        ]
                    }
                    PeerInteractionType::Criticism => {
                        vec![
                            (
                                OCCEmotion::Shame,
                                (-outcome) * peer_status * social_multiplier,
                            ),
                            (OCCEmotion::Anger, (-outcome) * 0.3),
                        ]
                    }
                    _ => vec![], // Other interaction types
                }
            }

            EmotionalEvent::SystemError {
                recovery_possible,
                impact_severity,
                ..
            } => {
                let mut emotions = vec![(OCCEmotion::Distress, impact_severity * 0.8)];

                if !*recovery_possible {
                    emotions.push((OCCEmotion::Fear, impact_severity * 0.6));
                } else {
                    emotions.push((OCCEmotion::Hope, (1.0 - impact_severity) * 0.5));
                }

                emotions
            }

            EmotionalEvent::ResourceAccess {
                access_granted,
                importance,
                ..
            } => {
                if *access_granted {
                    vec![
                        (OCCEmotion::Joy, importance * 0.6),
                        (OCCEmotion::Relief, importance * 0.4),
                    ]
                } else {
                    vec![
                        (OCCEmotion::Distress, importance * 0.7),
                        (OCCEmotion::Anger, importance * 0.5),
                    ]
                }
            }
        }
    }

    /// Update internal context based on recent events
    pub fn update_context(&mut self, stress_delta: f32, workload_delta: f32) {
        self.current_context.stress_level =
            (self.current_context.stress_level + stress_delta).clamp(0.0, 1.0);
        self.current_context.workload =
            (self.current_context.workload + workload_delta).clamp(0.0, 1.0);
    }

    /// Get current emotional baseline based on context
    pub fn get_baseline_emotions(&self) -> Vec<(OCCEmotion, EmotionalIntensity)> {
        let mut baseline = Vec::new();

        if self.current_context.stress_level > 0.6 {
            baseline.push((
                OCCEmotion::Distress,
                self.current_context.stress_level * 0.3,
            ));
        }

        if self.current_context.recent_failures > 3 {
            baseline.push((OCCEmotion::Shame, 0.4));
            baseline.push((OCCEmotion::Anger, 0.3));
        }

        if self.current_context.workload > 0.8 {
            baseline.push((OCCEmotion::Fear, self.current_context.workload * 0.2));
        }

        baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_task_appraisal() {
        let mut engine =
            AppraisalEngine::new(vec!["coding".to_string()], PersonalityModifiers::default());

        let event = EmotionalEvent::TaskCompleted {
            difficulty: 0.7,
            success: true,
            time_taken: 10.0,
            expected_time: 15.0,
            was_retry: false,
        };

        let emotions = engine.appraise_event(&event);

        // Should generate positive emotions
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Joy));
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Pride));
    }

    #[test]
    fn test_failed_task_appraisal() {
        let mut engine =
            AppraisalEngine::new(vec!["coding".to_string()], PersonalityModifiers::default());

        let event = EmotionalEvent::TaskCompleted {
            difficulty: 0.3, // Easy task
            success: false,
            time_taken: 20.0,
            expected_time: 10.0,
            was_retry: true,
        };

        let emotions = engine.appraise_event(&event);

        // Should generate negative emotions
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Distress));
        // Should have shame for failing an easy task
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Shame));
    }

    #[test]
    fn test_tool_failure_escalation() {
        let mut engine = AppraisalEngine::new(
            vec!["problem_solving".to_string()],
            PersonalityModifiers {
                volatility: 1.5,
                ..Default::default()
            },
        );

        let event = EmotionalEvent::ToolCallFailed {
            tool_name: "search".to_string(),
            attempts: 3,
            error_severity: 0.8,
            is_critical: true,
            error_message: None,
        };

        let emotions = engine.appraise_event(&event);

        // Should have high anger due to volatility and multiple attempts
        let anger_intensity = emotions
            .iter()
            .find(|(e, _)| *e == OCCEmotion::Anger)
            .map(|(_, i)| *i)
            .unwrap_or(0.0);

        assert!(anger_intensity > 0.7);
    }

    #[test]
    fn test_positive_feedback_response() {
        let mut engine =
            AppraisalEngine::new(vec!["helping".to_string()], PersonalityModifiers::default());

        let event = EmotionalEvent::UserFeedback {
            sentiment: 0.8,
            specificity: 0.9,
            is_constructive: true,
            contains_praise: true,
            contains_criticism: false,
        };

        let emotions = engine.appraise_event(&event);

        // Should generate joy, pride, and gratitude
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Joy));
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Pride));
        assert!(emotions.iter().any(|(e, _)| *e == OCCEmotion::Gratitude));
    }
}
