//! Core providers offered by the bootstrap plugin

use async_trait::async_trait;
use banshee_core::evaluator::*;
use banshee_core::provider::*;
use banshee_core::*;
use std::collections::HashMap;

/// Provider for conversation context
pub struct ConversationProvider {
    config: ProviderConfig,
}

impl ConversationProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig {
                name: "conversation".to_string(),
                description: "Provides conversation context and history".to_string(),
                priority: 80,
                enabled: true,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Provider for ConversationProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn provide(&self, context: &Context) -> Result<Vec<ProviderResult>> {
        let mut results = Vec::new();

        // Provide current conversation context
        if let Some(current_msg) = context.latest_message() {
            results.push(ProviderResult {
                provider: self.name().to_string(),
                data: serde_json::json!({
                    "current_message": current_msg,
                    "context_type": "current_message"
                }),
                relevance: 1.0,
                confidence: 0.9,
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
            });
        }

        // Provide conversation history summary
        results.push(ProviderResult {
            provider: self.name().to_string(),
            data: serde_json::json!({
                "conversation_length": context.conversation.len(),
                "context_type": "conversation_summary"
            }),
            relevance: 0.8,
            confidence: 0.85,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        Ok(context.latest_message().is_some() || !context.conversation.is_empty())
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Conversation provider initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Conversation provider shutting down");
        Ok(())
    }
}

#[allow(async_fn_in_trait)]
impl ContextProvider for ConversationProvider {
    async fn get_conversation_context(&self, context: &Context) -> Result<ProviderResult> {
        Ok(ProviderResult {
            provider: self.name().to_string(),
            data: serde_json::json!({
                "conversation": context.conversation,
                "latest_message": context.latest_message(),
                "session_id": context.session_id
            }),
            relevance: 1.0,
            confidence: 0.95,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn get_user_context(&self, user_id: &str) -> Result<ProviderResult> {
        Ok(ProviderResult {
            provider: self.name().to_string(),
            data: serde_json::json!({
                "user_id": user_id,
                "context_type": "user_basic"
            }),
            relevance: 0.7,
            confidence: 0.8,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Provider for user information
pub struct UserProvider {
    config: ProviderConfig,
}

impl UserProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig {
                name: "user".to_string(),
                description: "Provides user context and preferences".to_string(),
                priority: 70,
                enabled: true,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Provider for UserProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn provide(&self, context: &Context) -> Result<Vec<ProviderResult>> {
        let mut results = Vec::new();

        // Provide user information if available
        if let Some(user_id) = &context.user_id {
            results.push(ProviderResult {
                provider: self.name().to_string(),
                data: serde_json::json!({
                    "user_id": user_id,
                    "preferences": {
                        "communication_style": "friendly",
                        "response_length": "medium"
                    }
                }),
                relevance: 0.9,
                confidence: 0.7,
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(results)
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        Ok(context.user_id.is_some())
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("User provider initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("User provider shutting down");
        Ok(())
    }
}

/// Basic performance evaluator
pub struct BasicPerformanceEvaluator {
    config: EvaluatorConfig,
}

impl BasicPerformanceEvaluator {
    pub fn new() -> Self {
        Self {
            config: EvaluatorConfig {
                name: "basic_performance".to_string(),
                description: "Basic performance monitoring and evaluation".to_string(),
                frequency: EvaluationFrequency::EveryNMessages(5),
                enabled: true,
                thresholds: {
                    let mut thresholds = HashMap::new();
                    thresholds.insert("response_time_ms".to_string(), 1000.0);
                    thresholds.insert("min_quality_score".to_string(), 0.7);
                    thresholds
                },
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Evaluator for BasicPerformanceEvaluator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &EvaluatorConfig {
        &self.config
    }

    async fn evaluate(
        &self,
        _context: &Context,
        conversation: &[Message],
    ) -> Result<EvaluationResult> {
        let conversation_length = conversation.len();
        let quality_score = if conversation_length > 0 { 0.8 } else { 0.5 };

        Ok(EvaluationResult {
            evaluator: self.name().to_string(),
            score: quality_score,
            insights: vec![Insight {
                insight_type: InsightType::Performance,
                description: format!("Conversation has {} messages", conversation_length),
                confidence: 0.9,
                evidence: vec![format!("Message count: {}", conversation_length)],
            }],
            recommendations: if quality_score < 0.7 {
                vec![Recommendation {
                    priority: RecommendationPriority::Medium,
                    description: "Consider improving response quality".to_string(),
                    actions: vec!["Review conversation patterns".to_string()],
                    expected_impact: "Improved user satisfaction".to_string(),
                }]
            } else {
                vec![]
            },
            alerts: vec![],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn should_evaluate(&self, _context: &Context) -> Result<bool> {
        Ok(true) // Simple implementation always evaluates
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Basic performance evaluator initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Basic performance evaluator shutting down");
        Ok(())
    }
}
