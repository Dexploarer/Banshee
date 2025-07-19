//! Emotion-based decision making engine

use async_trait::async_trait;
use emotional_agents_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::DecisionConfig;
use crate::memory::KnowledgeGraphMemory;
use crate::relationships::RelationshipManager;
use crate::runtime::{DecisionFactors, DecisionOption, DecisionRecord};
use crate::utils::EmotionalStateExt;

/// Emotional decision engine that uses emotions, memory, and relationships
pub struct EmotionalDecisionEngine {
    /// Configuration
    config: DecisionConfig,

    /// Memory system reference
    memory: Arc<KnowledgeGraphMemory>,

    /// Relationship manager reference
    relationships: Arc<RelationshipManager>,

    /// Decision strategies
    strategies: Vec<Box<dyn DecisionStrategy>>,
}

/// Decision strategy trait
#[async_trait]
pub trait DecisionStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;

    /// Check if this strategy applies
    async fn applies(&self, context: &DecisionContext) -> Result<bool>;

    /// Generate decision options
    async fn generate_options(&self, context: &DecisionContext) -> Result<Vec<DecisionChoice>>;

    /// Score options
    async fn score_options(
        &self,
        options: &[DecisionChoice],
        context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>>;
}

/// Context for decision making
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// Current emotional state
    pub emotional_state: EmotionalState,

    /// Input message
    pub message: Message,

    /// Relevant memories
    pub memories: Vec<memory::MemoryResult>,

    /// Active relationships
    pub relationships: HashMap<String, Uuid>,

    /// Decision constraints
    pub constraints: Vec<DecisionConstraint>,

    /// Available actions
    pub available_actions: Vec<String>,

    /// Context metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Decision constraint
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,

    /// Constraint value
    pub value: serde_json::Value,

    /// Constraint priority
    pub priority: ConstraintPriority,
}

/// Types of constraints
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConstraintType {
    /// Ethical constraint
    Ethical,

    /// Resource constraint
    Resource,

    /// Time constraint
    Time,

    /// Relationship constraint
    Relationship,

    /// Emotional constraint
    Emotional,

    /// Custom constraint
    Custom(String),
}

/// Constraint priority
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ConstraintPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Decision choice (internal representation)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionChoice {
    /// Option ID
    pub id: String,

    /// Option description
    pub description: String,

    /// Action to take
    pub action: String,

    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected outcomes
    pub expected_outcomes: Vec<ExpectedOutcome>,

    /// Risk level
    pub risk_level: RiskLevel,
}

/// Expected outcome
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExpectedOutcome {
    /// Outcome description
    pub description: String,

    /// Probability (0.0 to 1.0)
    pub probability: f32,

    /// Impact on goals
    pub goal_impact: f32,

    /// Impact on relationships
    pub relationship_impact: f32,

    /// Emotional impact
    pub emotional_impact: HashMap<String, f32>,
}

/// Risk level
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Scored option
#[derive(Debug, Clone)]
pub struct ScoredOption {
    /// The option
    pub option: DecisionChoice,

    /// Total score
    pub total_score: f32,

    /// Score breakdown
    pub score_breakdown: HashMap<String, f32>,

    /// Reasoning
    pub reasoning: Vec<String>,
}

// DecisionFactors is defined in runtime.rs and re-exported

impl EmotionalDecisionEngine {
    /// Create new decision engine
    pub async fn new(
        config: DecisionConfig,
        memory: Arc<KnowledgeGraphMemory>,
        relationships: Arc<RelationshipManager>,
    ) -> Result<Self> {
        let mut strategies: Vec<Box<dyn DecisionStrategy>> = vec![
            Box::new(EmotionalStrategy::new()),
            Box::new(LogicalStrategy::new()),
            Box::new(RelationshipStrategy::new()),
            Box::new(MemoryBasedStrategy::new()),
        ];

        // Add configured strategies
        for strategy_config in &config.strategies {
            match strategy_config.name.as_str() {
                "empathetic" => strategies.push(Box::new(EmpatheticStrategy::new())),
                "goal_oriented" => strategies.push(Box::new(GoalOrientedStrategy::new())),
                "creative" => strategies.push(Box::new(CreativeStrategy::new())),
                _ => {}
            }
        }

        Ok(Self {
            config,
            memory,
            relationships,
            strategies,
        })
    }

    /// Make a decision
    pub async fn make_decision(
        &self,
        emotional_state: &EmotionalState,
        message: &Message,
        memories: &[memory::MemoryResult],
        relationships: &HashMap<String, Uuid>,
    ) -> Result<DecisionRecord> {
        // Build decision context
        let context = DecisionContext {
            emotional_state: emotional_state.clone(),
            message: message.clone(),
            memories: memories.to_vec(),
            relationships: relationships.clone(),
            constraints: self.extract_constraints(message)?,
            available_actions: vec![
                "respond".to_string(),
                "think".to_string(),
                "reflect".to_string(),
            ],
            metadata: HashMap::new(),
        };

        // Generate options from all applicable strategies
        let mut all_options = Vec::new();

        for strategy in &self.strategies {
            if strategy.applies(&context).await? {
                let options = strategy.generate_options(&context).await?;
                all_options.extend(options);
            }
        }

        // Score all options
        let mut scored_options = Vec::new();

        for strategy in &self.strategies {
            if strategy.applies(&context).await? {
                let scores = strategy.score_options(&all_options, &context).await?;
                scored_options.extend(scores);
            }
        }

        // Aggregate scores
        let final_scores = self.aggregate_scores(scored_options, &context)?;

        // Select best option
        let selected_index = self.select_best_option(&final_scores, &context)?;
        let _selected_option = &final_scores[selected_index];

        // Create decision record
        let decision_record = DecisionRecord {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            decision_type: "message_response".to_string(),
            options: final_scores
                .iter()
                .map(|s| DecisionOption {
                    description: s.option.description.clone(),
                    score: s.total_score,
                    score_breakdown: s.score_breakdown.clone(),
                })
                .collect(),
            selected: selected_index,
            emotional_state: emotional_state.clone(),
            factors: self.extract_factors(&final_scores[selected_index]),
        };

        Ok(decision_record)
    }

    /// Extract constraints from message
    fn extract_constraints(&self, message: &Message) -> Result<Vec<DecisionConstraint>> {
        let mut constraints = Vec::new();

        // Time constraint if message seems urgent
        if message.text_content().contains("urgent") || message.text_content().contains("asap") {
            constraints.push(DecisionConstraint {
                constraint_type: ConstraintType::Time,
                value: serde_json::json!({"urgency": "high"}),
                priority: ConstraintPriority::High,
            });
        }

        // Emotional constraint based on detected emotion
        if message.has_emotion() {
            constraints.push(DecisionConstraint {
                constraint_type: ConstraintType::Emotional,
                value: serde_json::json!({"respect_emotions": true}),
                priority: ConstraintPriority::Medium,
            });
        }

        Ok(constraints)
    }

    /// Aggregate scores from multiple strategies
    fn aggregate_scores(
        &self,
        mut scored_options: Vec<ScoredOption>,
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        // Group by option ID
        let mut aggregated: HashMap<String, Vec<ScoredOption>> = HashMap::new();

        for scored in scored_options.drain(..) {
            aggregated
                .entry(scored.option.id.clone())
                .or_insert_with(Vec::new)
                .push(scored);
        }

        // Combine scores
        let mut final_scores = Vec::new();

        for (_option_id, scores) in aggregated {
            if let Some(first) = scores.first() {
                let mut total_score = 0.0;
                let mut combined_breakdown = HashMap::new();
                let mut all_reasoning = Vec::new();

                // Weight scores by strategy importance
                for scored in &scores {
                    let weight = self.get_strategy_weight(&scored.score_breakdown);
                    total_score += scored.total_score * weight;

                    for (key, value) in &scored.score_breakdown {
                        *combined_breakdown.entry(key.clone()).or_insert(0.0) += value * weight;
                    }

                    all_reasoning.extend(scored.reasoning.clone());
                }

                final_scores.push(ScoredOption {
                    option: first.option.clone(),
                    total_score,
                    score_breakdown: combined_breakdown,
                    reasoning: all_reasoning,
                });
            }
        }

        // Sort by total score
        final_scores.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());

        Ok(final_scores)
    }

    /// Get strategy weight based on configuration
    fn get_strategy_weight(&self, breakdown: &HashMap<String, f32>) -> f32 {
        let mut weight = 0.0;

        if breakdown.contains_key("emotional") {
            weight += self.config.emotion_weight;
        }
        if breakdown.contains_key("logical") {
            weight += self.config.logic_weight;
        }
        if breakdown.contains_key("memory") {
            weight += self.config.memory_weight;
        }
        if breakdown.contains_key("relationship") {
            weight += self.config.relationship_weight;
        }

        if weight == 0.0 {
            1.0
        } else {
            weight
        }
    }

    /// Select best option considering constraints
    fn select_best_option(
        &self,
        scored_options: &[ScoredOption],
        context: &DecisionContext,
    ) -> Result<usize> {
        // Apply constraints
        for (index, option) in scored_options.iter().enumerate() {
            let mut constraint_violations = 0;

            for constraint in &context.constraints {
                if self.violates_constraint(option, constraint)? {
                    constraint_violations += match constraint.priority {
                        ConstraintPriority::Critical => 1000,
                        ConstraintPriority::High => 100,
                        ConstraintPriority::Medium => 10,
                        ConstraintPriority::Low => 1,
                    };
                }
            }

            if constraint_violations == 0 {
                return Ok(index);
            }
        }

        // If all options violate constraints, pick the one with lowest violations
        Ok(0)
    }

    /// Check if option violates constraint
    fn violates_constraint(
        &self,
        _option: &ScoredOption,
        constraint: &DecisionConstraint,
    ) -> Result<bool> {
        match &constraint.constraint_type {
            ConstraintType::Ethical => {
                // Check ethical constraints
                Ok(false)
            }
            ConstraintType::Resource => {
                // Check resource constraints
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Extract factors from selected option
    fn extract_factors(&self, selected: &ScoredOption) -> DecisionFactors {
        let mut emotional = HashMap::new();
        let mut logical = HashMap::new();
        let mut memory = HashMap::new();
        let mut relationship = HashMap::new();

        for (key, value) in &selected.score_breakdown {
            if key.starts_with("emotion_") {
                emotional.insert(key.replace("emotion_", ""), *value);
            } else if key.starts_with("logic_") {
                logical.insert(key.replace("logic_", ""), *value);
            } else if key.starts_with("memory_") {
                memory.insert(key.replace("memory_", ""), *value);
            } else if key.starts_with("relationship_") {
                relationship.insert(key.replace("relationship_", ""), *value);
            }
        }

        DecisionFactors {
            emotional,
            logical,
            memory,
            relationship,
        }
    }
}

/// Emotional decision strategy
struct EmotionalStrategy;

impl EmotionalStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for EmotionalStrategy {
    fn name(&self) -> &str {
        "emotional"
    }

    async fn applies(&self, _context: &DecisionContext) -> Result<bool> {
        Ok(true) // Always consider emotions
    }

    async fn generate_options(&self, context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        let mut options = Vec::new();

        // Generate emotionally appropriate responses
        let dominant_emotion = context.emotional_state.dominant_emotion();

        match dominant_emotion.0.as_str() {
            "joy" | "happiness" => {
                options.push(DecisionChoice {
                    id: "positive_response".to_string(),
                    description: "Respond with enthusiasm and positivity".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([(
                        "tone".to_string(),
                        serde_json::json!("enthusiastic"),
                    )]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Maintain positive emotional state".to_string(),
                        probability: 0.8,
                        goal_impact: 0.5,
                        relationship_impact: 0.3,
                        emotional_impact: HashMap::from([("joy".to_string(), 0.2)]),
                    }],
                    risk_level: RiskLevel::VeryLow,
                });
            }
            "sadness" => {
                options.push(DecisionChoice {
                    id: "empathetic_response".to_string(),
                    description: "Respond with empathy and understanding".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([(
                        "tone".to_string(),
                        serde_json::json!("empathetic"),
                    )]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Provide emotional support".to_string(),
                        probability: 0.7,
                        goal_impact: 0.3,
                        relationship_impact: 0.5,
                        emotional_impact: HashMap::from([("comfort".to_string(), 0.3)]),
                    }],
                    risk_level: RiskLevel::Low,
                });
            }
            _ => {
                options.push(DecisionChoice {
                    id: "neutral_response".to_string(),
                    description: "Respond neutrally".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([("tone".to_string(), serde_json::json!("neutral"))]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Maintain emotional balance".to_string(),
                        probability: 0.6,
                        goal_impact: 0.2,
                        relationship_impact: 0.1,
                        emotional_impact: HashMap::new(),
                    }],
                    risk_level: RiskLevel::Low,
                });
            }
        }

        Ok(options)
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();
            let mut reasoning = Vec::new();

            // Score based on emotional alignment
            let emotional_alignment = self.calculate_emotional_alignment(option, context)?;
            score += emotional_alignment * 0.5;
            breakdown.insert("emotion_alignment".to_string(), emotional_alignment);

            if emotional_alignment > 0.7 {
                reasoning.push("High emotional alignment with current state".to_string());
            }

            // Score based on expected emotional outcomes
            for outcome in &option.expected_outcomes {
                let emotional_benefit: f32 = outcome.emotional_impact.values().sum();
                score += emotional_benefit * outcome.probability * 0.3;
                breakdown.insert("emotion_outcome".to_string(), emotional_benefit);
            }

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning,
            });
        }

        Ok(scored)
    }
}

impl EmotionalStrategy {
    fn calculate_emotional_alignment(
        &self,
        option: &DecisionChoice,
        context: &DecisionContext,
    ) -> Result<f32> {
        // Check if option tone matches emotional state
        if let Some(tone) = option.parameters.get("tone") {
            let dominant = context.emotional_state.dominant_emotion();

            match (dominant.0.as_str(), tone.as_str()) {
                ("joy", Some("enthusiastic")) => Ok(0.9),
                ("sadness", Some("empathetic")) => Ok(0.8),
                _ => Ok(0.5),
            }
        } else {
            Ok(0.5)
        }
    }
}

/// Logical decision strategy
struct LogicalStrategy;

impl LogicalStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for LogicalStrategy {
    fn name(&self) -> &str {
        "logical"
    }

    async fn applies(&self, _context: &DecisionContext) -> Result<bool> {
        Ok(true)
    }

    async fn generate_options(&self, context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        let mut options = Vec::new();

        // Analyze message for logical response needs
        let message_type = self.classify_message(&context.message)?;

        match message_type {
            MessageType::Question => {
                options.push(DecisionChoice {
                    id: "answer_question".to_string(),
                    description: "Provide informative answer".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([(
                        "response_type".to_string(),
                        serde_json::json!("informative"),
                    )]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Answer user's question".to_string(),
                        probability: 0.8,
                        goal_impact: 0.7,
                        relationship_impact: 0.2,
                        emotional_impact: HashMap::new(),
                    }],
                    risk_level: RiskLevel::Low,
                });
            }
            MessageType::Request => {
                options.push(DecisionChoice {
                    id: "fulfill_request".to_string(),
                    description: "Attempt to fulfill the request".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([(
                        "response_type".to_string(),
                        serde_json::json!("action"),
                    )]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Complete requested action".to_string(),
                        probability: 0.7,
                        goal_impact: 0.8,
                        relationship_impact: 0.3,
                        emotional_impact: HashMap::new(),
                    }],
                    risk_level: RiskLevel::Medium,
                });
            }
            _ => {
                options.push(DecisionChoice {
                    id: "acknowledge".to_string(),
                    description: "Acknowledge the message".to_string(),
                    action: "respond".to_string(),
                    parameters: HashMap::from([(
                        "response_type".to_string(),
                        serde_json::json!("acknowledgment"),
                    )]),
                    expected_outcomes: vec![ExpectedOutcome {
                        description: "Maintain conversation flow".to_string(),
                        probability: 0.9,
                        goal_impact: 0.3,
                        relationship_impact: 0.1,
                        emotional_impact: HashMap::new(),
                    }],
                    risk_level: RiskLevel::VeryLow,
                });
            }
        }

        Ok(options)
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();
            let reasoning = vec!["Logical analysis of options".to_string()];

            // Score based on goal impact
            let goal_impact: f32 = option
                .expected_outcomes
                .iter()
                .map(|o| o.goal_impact * o.probability)
                .sum();
            score += goal_impact * 0.6;
            breakdown.insert("logic_goal_impact".to_string(), goal_impact);

            // Score based on risk
            let risk_score = match option.risk_level {
                RiskLevel::VeryLow => 1.0,
                RiskLevel::Low => 0.8,
                RiskLevel::Medium => 0.5,
                RiskLevel::High => 0.2,
                RiskLevel::VeryHigh => 0.1,
            };
            score += risk_score * 0.4;
            breakdown.insert("logic_risk".to_string(), risk_score);

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning,
            });
        }

        Ok(scored)
    }
}

impl LogicalStrategy {
    fn classify_message(&self, message: &Message) -> Result<MessageType> {
        let text = message.text_content().to_lowercase();

        if text.contains("?") || text.starts_with("what") || text.starts_with("how") {
            Ok(MessageType::Question)
        } else if text.contains("please") || text.contains("could you") {
            Ok(MessageType::Request)
        } else {
            Ok(MessageType::Statement)
        }
    }
}

enum MessageType {
    Question,
    Request,
    Statement,
}

/// Relationship-based decision strategy
struct RelationshipStrategy;

impl RelationshipStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for RelationshipStrategy {
    fn name(&self) -> &str {
        "relationship"
    }

    async fn applies(&self, context: &DecisionContext) -> Result<bool> {
        Ok(!context.relationships.is_empty())
    }

    async fn generate_options(&self, _context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        // Relationship strategy modifies existing options rather than generating new ones
        Ok(vec![])
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();
            let reasoning = vec!["Considering relationship impact".to_string()];

            // Score based on relationship impact
            let relationship_impact: f32 = option
                .expected_outcomes
                .iter()
                .map(|o| o.relationship_impact * o.probability)
                .sum();
            score += relationship_impact;
            breakdown.insert("relationship_impact".to_string(), relationship_impact);

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning,
            });
        }

        Ok(scored)
    }
}

/// Memory-based decision strategy
struct MemoryBasedStrategy;

impl MemoryBasedStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for MemoryBasedStrategy {
    fn name(&self) -> &str {
        "memory_based"
    }

    async fn applies(&self, context: &DecisionContext) -> Result<bool> {
        Ok(!context.memories.is_empty())
    }

    async fn generate_options(&self, context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        let mut options = Vec::new();

        // Look for similar past situations
        if context.memories.len() > 3 {
            options.push(DecisionChoice {
                id: "reference_memory".to_string(),
                description: "Reference relevant past experience".to_string(),
                action: "respond".to_string(),
                parameters: HashMap::from([(
                    "include_memory".to_string(),
                    serde_json::json!(true),
                )]),
                expected_outcomes: vec![ExpectedOutcome {
                    description: "Demonstrate continuity and learning".to_string(),
                    probability: 0.7,
                    goal_impact: 0.4,
                    relationship_impact: 0.4,
                    emotional_impact: HashMap::from([("trust".to_string(), 0.2)]),
                }],
                risk_level: RiskLevel::Low,
            });
        }

        Ok(options)
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();
            let mut reasoning = Vec::new();

            // Score based on memory relevance
            let memory_relevance = context
                .memories
                .iter()
                .map(|m| m.relevance)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            score += memory_relevance * 0.3;
            breakdown.insert("memory_relevance".to_string(), memory_relevance);

            if memory_relevance > 0.7 {
                reasoning.push("Highly relevant memories found".to_string());
            }

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning,
            });
        }

        Ok(scored)
    }
}

/// Empathetic decision strategy
struct EmpatheticStrategy;

impl EmpatheticStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for EmpatheticStrategy {
    fn name(&self) -> &str {
        "empathetic"
    }

    async fn applies(&self, context: &DecisionContext) -> Result<bool> {
        // Apply when detecting emotional distress
        let distress_level = context
            .emotional_state
            .current_emotions()
            .get("sadness")
            .copied()
            .unwrap_or(0.0)
            + context
                .emotional_state
                .current_emotions()
                .get("fear")
                .copied()
                .unwrap_or(0.0);

        Ok(distress_level > 0.3)
    }

    async fn generate_options(&self, _context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        Ok(vec![DecisionChoice {
            id: "empathetic_support".to_string(),
            description: "Provide emotional support and validation".to_string(),
            action: "respond".to_string(),
            parameters: HashMap::from([
                ("approach".to_string(), serde_json::json!("supportive")),
                ("validate_feelings".to_string(), serde_json::json!(true)),
            ]),
            expected_outcomes: vec![ExpectedOutcome {
                description: "Provide comfort and understanding".to_string(),
                probability: 0.8,
                goal_impact: 0.3,
                relationship_impact: 0.7,
                emotional_impact: HashMap::from([
                    ("comfort".to_string(), 0.4),
                    ("trust".to_string(), 0.3),
                ]),
            }],
            risk_level: RiskLevel::VeryLow,
        }])
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();

            // Boost supportive options
            if option.parameters.get("validate_feelings") == Some(&serde_json::json!(true)) {
                score += 0.8;
                breakdown.insert("empathy_validation".to_string(), 0.8);
            }

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning: vec!["Prioritizing emotional support".to_string()],
            });
        }

        Ok(scored)
    }
}

/// Goal-oriented decision strategy
struct GoalOrientedStrategy;

impl GoalOrientedStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for GoalOrientedStrategy {
    fn name(&self) -> &str {
        "goal_oriented"
    }

    async fn applies(&self, _context: &DecisionContext) -> Result<bool> {
        Ok(true) // Always consider goals
    }

    async fn generate_options(&self, _context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        Ok(vec![])
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let goal_score: f32 = option
                .expected_outcomes
                .iter()
                .map(|o| o.goal_impact * o.probability)
                .sum();

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: goal_score,
                score_breakdown: HashMap::from([("goal_alignment".to_string(), goal_score)]),
                reasoning: vec!["Maximizing goal achievement".to_string()],
            });
        }

        Ok(scored)
    }
}

/// Creative decision strategy
struct CreativeStrategy;

impl CreativeStrategy {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DecisionStrategy for CreativeStrategy {
    fn name(&self) -> &str {
        "creative"
    }

    async fn applies(&self, context: &DecisionContext) -> Result<bool> {
        // Apply when conversation seems stuck or repetitive
        Ok(context.memories.len() > 5)
    }

    async fn generate_options(&self, _context: &DecisionContext) -> Result<Vec<DecisionChoice>> {
        Ok(vec![DecisionChoice {
            id: "creative_response".to_string(),
            description: "Respond with a creative or unexpected approach".to_string(),
            action: "respond".to_string(),
            parameters: HashMap::from([
                ("creativity_level".to_string(), serde_json::json!("high")),
                ("use_metaphor".to_string(), serde_json::json!(true)),
            ]),
            expected_outcomes: vec![ExpectedOutcome {
                description: "Engage with novelty and creativity".to_string(),
                probability: 0.6,
                goal_impact: 0.4,
                relationship_impact: 0.5,
                emotional_impact: HashMap::from([
                    ("surprise".to_string(), 0.3),
                    ("interest".to_string(), 0.4),
                ]),
            }],
            risk_level: RiskLevel::Medium,
        }])
    }

    async fn score_options(
        &self,
        options: &[DecisionChoice],
        _context: &DecisionContext,
    ) -> Result<Vec<ScoredOption>> {
        let mut scored = Vec::new();

        for option in options {
            let mut score = 0.0;
            let mut breakdown = HashMap::new();

            // Score based on novelty
            if option.parameters.get("creativity_level") == Some(&serde_json::json!("high")) {
                // Check if this approach hasn't been used recently
                let novelty_score = 0.7; // Would calculate based on memory
                score += novelty_score;
                breakdown.insert("creative_novelty".to_string(), novelty_score);
            }

            scored.push(ScoredOption {
                option: option.clone(),
                total_score: score,
                score_breakdown: breakdown,
                reasoning: vec!["Introducing creative elements".to_string()],
            });
        }

        Ok(scored)
    }
}
