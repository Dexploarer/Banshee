use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Context, Message, Result};

/// Result of evaluation containing insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Evaluator name
    pub evaluator: String,

    /// Evaluation score (0.0 to 1.0)
    pub score: f32,

    /// Detailed insights from evaluation
    pub insights: Vec<Insight>,

    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,

    /// Whether this evaluation triggers alerts
    pub alerts: Vec<Alert>,

    /// Evaluation metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp of evaluation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Insight from evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Type of insight
    pub insight_type: InsightType,

    /// Insight description
    pub description: String,

    /// Confidence in insight (0.0 to 1.0)
    pub confidence: f32,

    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Types of insights evaluators can provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    /// Emotional pattern detected
    EmotionalPattern,

    /// Communication effectiveness
    Communication,

    /// Tool usage efficiency
    ToolUsage,

    /// Memory utilization
    Memory,

    /// Goal progress
    GoalProgress,

    /// Performance metrics
    Performance,

    /// User satisfaction
    UserSatisfaction,

    /// Learning opportunity
    Learning,

    /// Custom insight type
    Custom(String),
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Priority of recommendation
    pub priority: RecommendationPriority,

    /// Recommendation description
    pub description: String,

    /// Specific actions to take
    pub actions: Vec<String>,

    /// Expected impact
    pub expected_impact: String,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert from evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert message
    pub message: String,

    /// Alert category
    pub category: AlertCategory,

    /// Whether immediate action is required
    pub requires_action: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    Performance,
    EmotionalHealth,
    UserExperience,
    Security,
    SystemHealth,
    Custom(String),
}

/// Evaluator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    /// Evaluator name
    pub name: String,

    /// Evaluator description
    pub description: String,

    /// How often to run this evaluator
    pub frequency: EvaluationFrequency,

    /// Whether this evaluator is enabled
    pub enabled: bool,

    /// Evaluation thresholds
    pub thresholds: HashMap<String, f32>,

    /// Evaluator settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// How frequently to run evaluations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationFrequency {
    /// After every message
    PerMessage,

    /// After every N messages
    EveryNMessages(u32),

    /// Every N seconds
    Periodic(u32),

    /// On demand only
    OnDemand,

    /// Based on triggers
    Triggered(Vec<EvaluationTrigger>),
}

/// Triggers for evaluations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationTrigger {
    /// High emotional intensity
    EmotionalSpike(f32),

    /// Low user satisfaction
    LowSatisfaction(f32),

    /// Performance degradation
    PerformanceDrop(f32),

    /// Error threshold reached
    ErrorThreshold(u32),

    /// Custom trigger
    Custom(String),
}

/// Core evaluator trait
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// Get the evaluator name
    fn name(&self) -> &str;

    /// Get the evaluator description
    fn description(&self) -> &str;

    /// Get the evaluator configuration
    fn config(&self) -> &EvaluatorConfig;

    /// Evaluate the given context and conversation
    async fn evaluate(
        &self,
        context: &Context,
        conversation: &[Message],
    ) -> Result<EvaluationResult>;

    /// Check if evaluation should be triggered
    async fn should_evaluate(&self, context: &Context) -> Result<bool>;

    /// Initialize the evaluator
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the evaluator
    async fn shutdown(&mut self) -> Result<()>;
}

/// Specific evaluator types
#[async_trait]
pub trait EmotionalEvaluator: Evaluator {
    /// Evaluate emotional health and patterns
    async fn evaluate_emotional_health(&self, context: &Context) -> Result<EvaluationResult>;

    /// Detect emotional patterns
    async fn detect_patterns(&self, conversation: &[Message]) -> Result<Vec<Insight>>;
}

#[async_trait]
pub trait PerformanceEvaluator: Evaluator {
    /// Evaluate response time and efficiency
    async fn evaluate_performance(&self, context: &Context) -> Result<EvaluationResult>;

    /// Analyze resource utilization
    async fn analyze_resources(&self, context: &Context) -> Result<Vec<Insight>>;
}

#[async_trait]
pub trait QualityEvaluator: Evaluator {
    /// Evaluate response quality
    async fn evaluate_quality(
        &self,
        message: &Message,
        context: &Context,
    ) -> Result<EvaluationResult>;

    /// Check for potential issues
    async fn check_issues(&self, message: &Message) -> Result<Vec<Alert>>;
}

/// Registry for managing evaluators
pub struct EvaluatorRegistry {
    evaluators: HashMap<String, Box<dyn Evaluator>>,
}

impl EvaluatorRegistry {
    pub fn new() -> Self {
        Self {
            evaluators: HashMap::new(),
        }
    }

    /// Register a new evaluator
    pub async fn register(&mut self, mut evaluator: Box<dyn Evaluator>) -> Result<()> {
        let name = evaluator.name().to_string();
        evaluator.initialize().await?;
        self.evaluators.insert(name, evaluator);
        Ok(())
    }

    /// Get an evaluator by name
    pub fn get(&self, name: &str) -> Option<&dyn Evaluator> {
        self.evaluators.get(name).map(|e| e.as_ref())
    }

    /// Run all relevant evaluators
    pub async fn evaluate_all(
        &self,
        context: &Context,
        conversation: &[Message],
    ) -> Result<Vec<EvaluationResult>> {
        let mut results = Vec::new();

        for evaluator in self.evaluators.values() {
            if !evaluator.config().enabled {
                continue;
            }

            if evaluator.should_evaluate(context).await? {
                let result = evaluator.evaluate(context, conversation).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Run only enabled evaluators
    pub async fn evaluate_enabled(
        &self,
        context: &Context,
        conversation: &[Message],
    ) -> Result<Vec<EvaluationResult>> {
        let mut results = Vec::new();

        for evaluator in self.evaluators.values() {
            if evaluator.config().enabled && evaluator.should_evaluate(context).await? {
                let result = evaluator.evaluate(context, conversation).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get all alerts from enabled evaluators
    pub async fn get_alerts(
        &self,
        context: &Context,
        conversation: &[Message],
    ) -> Result<Vec<Alert>> {
        let results = self.evaluate_enabled(context, conversation).await?;

        let mut alerts = Vec::new();
        for result in results {
            alerts.extend(result.alerts);
        }

        // Sort by severity (critical first)
        alerts.sort_by(|a, b| {
            let a_priority = match a.severity {
                AlertSeverity::Critical => 4,
                AlertSeverity::Error => 3,
                AlertSeverity::Warning => 2,
                AlertSeverity::Info => 1,
            };
            let b_priority = match b.severity {
                AlertSeverity::Critical => 4,
                AlertSeverity::Error => 3,
                AlertSeverity::Warning => 2,
                AlertSeverity::Info => 1,
            };
            b_priority.cmp(&a_priority)
        });

        Ok(alerts)
    }
}
