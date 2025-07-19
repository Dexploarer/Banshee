//! Relationship management system with standing mechanics

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use emotional_agents_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::{RelationshipConfig, StandingMethod};
use crate::utils::EmotionalStateExt;

/// Relationship manager for tracking interpersonal connections
pub struct RelationshipManager {
    /// Configuration
    config: RelationshipConfig,

    /// Active relationships
    relationships: Arc<RwLock<HashMap<Uuid, Relationship>>>,

    /// Relationship index by participants
    index: Arc<RwLock<HashMap<(String, String), Uuid>>>,

    /// Standing calculator
    standing_calculator: Box<dyn StandingCalculator>,

    /// Storage backend
    storage: Box<dyn RelationshipStorage>,
}

/// A relationship between two entities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Relationship {
    /// Unique relationship ID
    pub id: Uuid,

    /// First participant (usually the agent)
    pub participant_a: String,

    /// Second participant (usually the user)
    pub participant_b: String,

    /// Current standing
    pub standing: RelationshipStanding,

    /// Trust level (0.0 to 1.0)
    pub trust: f32,

    /// Interaction history
    pub history: Vec<InteractionRecord>,

    /// Emotional associations
    pub emotional_associations: HashMap<String, f32>,

    /// Shared memories
    pub shared_memories: Vec<Uuid>,

    /// Relationship tags
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last interaction
    pub last_interaction: DateTime<Utc>,

    /// Total interactions
    pub interaction_count: u32,

    /// Relationship metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Relationship standing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RelationshipStanding {
    /// Numeric standing
    Numeric(f32),

    /// Categorical standing
    Categorical(StandingCategory),

    /// Multi-dimensional standing
    MultiDimensional(HashMap<String, f32>),
}

/// Standing categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StandingCategory {
    /// Excellent relationship
    Excellent,

    /// Good relationship
    Good,

    /// Neutral relationship
    Neutral,

    /// Poor relationship
    Poor,

    /// Bad relationship
    Bad,

    /// Hostile relationship
    Hostile,
}

/// Record of an interaction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InteractionRecord {
    /// Interaction ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Interaction type
    pub interaction_type: InteractionType,

    /// Sentiment (-1.0 to 1.0)
    pub sentiment: f32,

    /// Impact on standing
    pub standing_impact: f32,

    /// Impact on trust
    pub trust_impact: f32,

    /// Emotional context
    pub emotional_context: HashMap<String, f32>,

    /// Interaction content summary
    pub summary: String,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of interactions
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InteractionType {
    /// Normal conversation
    Conversation,

    /// Conflict or disagreement
    Conflict,

    /// Cooperation
    Cooperation,

    /// Help given
    HelpGiven,

    /// Help received
    HelpReceived,

    /// Shared experience
    SharedExperience,

    /// Trust building
    TrustBuilding,

    /// Trust breaking
    TrustBreaking,

    /// Emotional support
    EmotionalSupport,

    /// Custom interaction
    Custom(String),
}

/// Standing calculator trait
#[async_trait]
pub trait StandingCalculator: Send + Sync {
    /// Calculate standing from history
    async fn calculate_standing(
        &self,
        history: &[InteractionRecord],
        current_standing: &RelationshipStanding,
    ) -> Result<RelationshipStanding>;

    /// Apply decay to standing
    async fn apply_decay(
        &self,
        standing: &RelationshipStanding,
        days_since_interaction: f32,
        config: &RelationshipConfig,
    ) -> Result<RelationshipStanding>;

    /// Convert standing to category
    fn to_category(&self, standing: &RelationshipStanding) -> StandingCategory;
}

/// Storage trait for relationships
#[async_trait]
pub trait RelationshipStorage: Send + Sync {
    /// Save relationship
    async fn save(&self, relationship: &Relationship) -> Result<()>;

    /// Load relationship
    async fn load(&self, id: Uuid) -> Result<Option<Relationship>>;

    /// Find relationship by participants
    async fn find_by_participants(
        &self,
        participant_a: &str,
        participant_b: &str,
    ) -> Result<Option<Relationship>>;

    /// List all relationships for a participant
    async fn list_for_participant(&self, participant: &str) -> Result<Vec<Relationship>>;

    /// Delete relationship
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

impl RelationshipManager {
    /// Create new relationship manager
    pub async fn new(config: RelationshipConfig) -> Result<Self> {
        let standing_calculator: Box<dyn StandingCalculator> = match &config.standing_method {
            StandingMethod::Numeric { .. } => Box::new(NumericStandingCalculator),
            StandingMethod::Categorical => Box::new(CategoricalStandingCalculator),
            StandingMethod::MultiDimensional { dimensions } => {
                Box::new(MultiDimensionalStandingCalculator::new(dimensions.clone()))
            }
        };

        let storage = Box::new(InMemoryRelationshipStorage::new());

        Ok(Self {
            config,
            relationships: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
            standing_calculator,
            storage,
        })
    }

    /// Get or create a relationship
    pub async fn get_or_create(
        &self,
        participant_a: String,
        participant_b: String,
    ) -> Result<Relationship> {
        // Check index
        let key = self.make_key(&participant_a, &participant_b);

        if let Some(id) = self.index.read().await.get(&key) {
            if let Some(relationship) = self.relationships.read().await.get(id) {
                return Ok(relationship.clone());
            }
        }

        // Check storage
        if let Some(relationship) = self
            .storage
            .find_by_participants(&participant_a, &participant_b)
            .await?
        {
            self.cache_relationship(relationship.clone()).await?;
            return Ok(relationship);
        }

        // Create new relationship
        let relationship = self
            .create_new_relationship(participant_a, participant_b)
            .await?;
        self.cache_relationship(relationship.clone()).await?;
        self.storage.save(&relationship).await?;

        Ok(relationship)
    }

    /// Update relationship from interaction
    pub async fn update_from_interaction(
        &self,
        relationship_id: &Uuid,
        message: &Message,
        emotional_state: &EmotionalState,
    ) -> Result<()> {
        let mut relationships = self.relationships.write().await;
        let relationship = relationships
            .get_mut(relationship_id)
            .ok_or("Relationship not found")?;

        // Analyze interaction
        let sentiment = self.analyze_sentiment(message)?;
        let interaction_type = self.determine_interaction_type(message, sentiment)?;

        // Calculate impacts
        let (standing_impact, trust_impact) =
            self.calculate_impacts(&interaction_type, sentiment, emotional_state)?;

        // Create interaction record
        let interaction = InteractionRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            interaction_type,
            sentiment,
            standing_impact,
            trust_impact,
            emotional_context: emotional_state.current_emotions(),
            summary: message.text_content().chars().take(100).collect(),
            metadata: HashMap::new(),
        };

        // Update relationship
        relationship.history.push(interaction);
        relationship.last_interaction = Utc::now();
        relationship.interaction_count += 1;

        // Update trust
        let new_trust = self.update_trust(relationship.trust, trust_impact, &self.config.trust);
        relationship.trust = new_trust;

        // Recalculate standing
        let new_standing = self
            .standing_calculator
            .calculate_standing(&relationship.history, &relationship.standing)
            .await?;
        relationship.standing = new_standing;

        // Update emotional associations
        self.update_emotional_associations(
            &mut relationship.emotional_associations,
            emotional_state,
            sentiment,
        )?;

        // Save to storage
        self.storage.save(relationship).await?;

        Ok(())
    }

    /// Get relationship by ID
    pub async fn get(&self, id: &Uuid) -> Result<Option<Relationship>> {
        if let Some(relationship) = self.relationships.read().await.get(id) {
            return Ok(Some(relationship.clone()));
        }

        if let Some(relationship) = self.storage.load(*id).await? {
            self.cache_relationship(relationship.clone()).await?;
            return Ok(Some(relationship));
        }

        Ok(None)
    }

    /// Get standing category
    pub fn get_standing_category(&self, relationship: &Relationship) -> StandingCategory {
        self.standing_calculator.to_category(&relationship.standing)
    }

    /// Apply decay to all relationships
    pub async fn apply_decay_all(&self) -> Result<()> {
        if !self.config.decay.enabled {
            return Ok(());
        }

        let mut relationships = self.relationships.write().await;

        for relationship in relationships.values_mut() {
            let days_since = (Utc::now() - relationship.last_interaction).num_days() as f32;

            let new_standing = self
                .standing_calculator
                .apply_decay(&relationship.standing, days_since, &self.config)
                .await?;

            relationship.standing = new_standing;
            self.storage.save(relationship).await?;
        }

        Ok(())
    }

    /// Create new relationship
    async fn create_new_relationship(
        &self,
        participant_a: String,
        participant_b: String,
    ) -> Result<Relationship> {
        let initial_standing = match &self.config.standing_method {
            StandingMethod::Numeric { min, max } => {
                RelationshipStanding::Numeric((min + max) / 2.0)
            }
            StandingMethod::Categorical => {
                RelationshipStanding::Categorical(StandingCategory::Neutral)
            }
            StandingMethod::MultiDimensional { dimensions } => {
                let mut values = HashMap::new();
                for dim in dimensions {
                    values.insert(dim.clone(), 0.0);
                }
                RelationshipStanding::MultiDimensional(values)
            }
        };

        Ok(Relationship {
            id: Uuid::new_v4(),
            participant_a,
            participant_b,
            standing: initial_standing,
            trust: self.config.trust.initial_trust,
            history: Vec::new(),
            emotional_associations: HashMap::new(),
            shared_memories: Vec::new(),
            tags: Vec::new(),
            created_at: Utc::now(),
            last_interaction: Utc::now(),
            interaction_count: 0,
            metadata: HashMap::new(),
        })
    }

    /// Cache relationship in memory
    async fn cache_relationship(&self, relationship: Relationship) -> Result<()> {
        let key = self.make_key(&relationship.participant_a, &relationship.participant_b);
        self.index.write().await.insert(key, relationship.id);
        self.relationships
            .write()
            .await
            .insert(relationship.id, relationship);
        Ok(())
    }

    /// Make key for index
    fn make_key(&self, a: &str, b: &str) -> (String, String) {
        // Ensure consistent ordering
        if a < b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    /// Analyze sentiment of message
    fn analyze_sentiment(&self, message: &Message) -> Result<f32> {
        // Simplified sentiment analysis
        let text = message.text_content().to_lowercase();

        let positive_words = ["good", "great", "thanks", "love", "happy", "wonderful"];
        let negative_words = ["bad", "hate", "angry", "terrible", "awful", "horrible"];

        let positive_count = positive_words
            .iter()
            .filter(|&&word| text.contains(word))
            .count();

        let negative_count = negative_words
            .iter()
            .filter(|&&word| text.contains(word))
            .count();

        let sentiment = (positive_count as f32 - negative_count as f32)
            / (positive_count + negative_count + 1) as f32;

        Ok(sentiment.clamp(-1.0, 1.0))
    }

    /// Determine interaction type
    fn determine_interaction_type(
        &self,
        message: &Message,
        sentiment: f32,
    ) -> Result<InteractionType> {
        let text = message.text_content().to_lowercase();

        if text.contains("help") || text.contains("assist") {
            Ok(InteractionType::HelpGiven)
        } else if text.contains("disagree") || text.contains("wrong") {
            Ok(InteractionType::Conflict)
        } else if text.contains("together") || text.contains("collaborate") {
            Ok(InteractionType::Cooperation)
        } else if sentiment > 0.5 {
            Ok(InteractionType::EmotionalSupport)
        } else if sentiment < -0.5 {
            Ok(InteractionType::Conflict)
        } else {
            Ok(InteractionType::Conversation)
        }
    }

    /// Calculate impacts on standing and trust
    fn calculate_impacts(
        &self,
        interaction_type: &InteractionType,
        sentiment: f32,
        _emotional_state: &EmotionalState,
    ) -> Result<(f32, f32)> {
        let base_standing_impact = match interaction_type {
            InteractionType::Cooperation => 0.1,
            InteractionType::HelpGiven => 0.15,
            InteractionType::HelpReceived => 0.1,
            InteractionType::EmotionalSupport => 0.12,
            InteractionType::TrustBuilding => 0.2,
            InteractionType::Conflict => -0.15,
            InteractionType::TrustBreaking => -0.3,
            _ => 0.05,
        };

        let base_trust_impact = match interaction_type {
            InteractionType::TrustBuilding => 0.1,
            InteractionType::TrustBreaking => -0.2,
            InteractionType::Cooperation => 0.05,
            InteractionType::Conflict => -0.05,
            _ => 0.01,
        };

        // Modify by sentiment
        let standing_impact = base_standing_impact * (1.0 + sentiment * 0.5);
        let trust_impact = base_trust_impact * (1.0 + sentiment * 0.3);

        Ok((standing_impact, trust_impact))
    }

    /// Update trust level
    fn update_trust(
        &self,
        current_trust: f32,
        impact: f32,
        config: &crate::config::TrustConfig,
    ) -> f32 {
        let new_trust = if impact > 0.0 {
            current_trust + (impact * config.gain_rate)
        } else {
            current_trust + (impact * config.loss_multiplier)
        };

        new_trust.clamp(0.0, 1.0)
    }

    /// Update emotional associations
    fn update_emotional_associations(
        &self,
        associations: &mut HashMap<String, f32>,
        emotional_state: &EmotionalState,
        sentiment: f32,
    ) -> Result<()> {
        for (emotion, intensity) in emotional_state.current_emotions() {
            let current = associations.get(&emotion).copied().unwrap_or(0.0);
            let update = intensity * sentiment * 0.1;
            associations.insert(emotion, (current + update).clamp(-1.0, 1.0));
        }

        Ok(())
    }

    /// Shutdown relationship manager
    pub async fn shutdown(&self) -> Result<()> {
        // Save all relationships
        let relationships = self.relationships.read().await;
        for relationship in relationships.values() {
            self.storage.save(relationship).await?;
        }

        Ok(())
    }
}

/// Numeric standing calculator
struct NumericStandingCalculator;

#[async_trait]
impl StandingCalculator for NumericStandingCalculator {
    async fn calculate_standing(
        &self,
        history: &[InteractionRecord],
        current_standing: &RelationshipStanding,
    ) -> Result<RelationshipStanding> {
        let current_value = match current_standing {
            RelationshipStanding::Numeric(v) => *v,
            _ => 0.0,
        };

        let total_impact: f32 = history.iter().map(|record| record.standing_impact).sum();

        let new_value = (current_value + total_impact).clamp(-1.0, 1.0);

        Ok(RelationshipStanding::Numeric(new_value))
    }

    async fn apply_decay(
        &self,
        standing: &RelationshipStanding,
        days_since_interaction: f32,
        config: &RelationshipConfig,
    ) -> Result<RelationshipStanding> {
        match standing {
            RelationshipStanding::Numeric(value) => {
                let decay = config.decay.daily_decay_rate * days_since_interaction;
                let new_value = (value - decay).max(config.decay.minimum_standing);
                Ok(RelationshipStanding::Numeric(new_value))
            }
            _ => Ok(standing.clone()),
        }
    }

    fn to_category(&self, standing: &RelationshipStanding) -> StandingCategory {
        match standing {
            RelationshipStanding::Numeric(value) => {
                if *value > 0.8 {
                    StandingCategory::Excellent
                } else if *value > 0.5 {
                    StandingCategory::Good
                } else if *value > 0.0 {
                    StandingCategory::Neutral
                } else if *value > -0.5 {
                    StandingCategory::Poor
                } else if *value > -0.8 {
                    StandingCategory::Bad
                } else {
                    StandingCategory::Hostile
                }
            }
            RelationshipStanding::Categorical(cat) => *cat,
            _ => StandingCategory::Neutral,
        }
    }
}

/// Categorical standing calculator
struct CategoricalStandingCalculator;

#[async_trait]
impl StandingCalculator for CategoricalStandingCalculator {
    async fn calculate_standing(
        &self,
        history: &[InteractionRecord],
        current_standing: &RelationshipStanding,
    ) -> Result<RelationshipStanding> {
        let current_cat = match current_standing {
            RelationshipStanding::Categorical(cat) => *cat,
            _ => StandingCategory::Neutral,
        };

        let total_impact: f32 = history.iter().map(|record| record.standing_impact).sum();

        let new_cat = if total_impact > 0.5 {
            self.improve_category(current_cat)
        } else if total_impact < -0.5 {
            self.degrade_category(current_cat)
        } else {
            current_cat
        };

        Ok(RelationshipStanding::Categorical(new_cat))
    }

    async fn apply_decay(
        &self,
        standing: &RelationshipStanding,
        _days_since_interaction: f32,
        _config: &RelationshipConfig,
    ) -> Result<RelationshipStanding> {
        // Categorical doesn't decay to prevent flip-flopping
        Ok(standing.clone())
    }

    fn to_category(&self, standing: &RelationshipStanding) -> StandingCategory {
        match standing {
            RelationshipStanding::Categorical(cat) => *cat,
            _ => StandingCategory::Neutral,
        }
    }
}

impl CategoricalStandingCalculator {
    fn improve_category(&self, category: StandingCategory) -> StandingCategory {
        match category {
            StandingCategory::Hostile => StandingCategory::Bad,
            StandingCategory::Bad => StandingCategory::Poor,
            StandingCategory::Poor => StandingCategory::Neutral,
            StandingCategory::Neutral => StandingCategory::Good,
            StandingCategory::Good => StandingCategory::Excellent,
            StandingCategory::Excellent => StandingCategory::Excellent,
        }
    }

    fn degrade_category(&self, category: StandingCategory) -> StandingCategory {
        match category {
            StandingCategory::Excellent => StandingCategory::Good,
            StandingCategory::Good => StandingCategory::Neutral,
            StandingCategory::Neutral => StandingCategory::Poor,
            StandingCategory::Poor => StandingCategory::Bad,
            StandingCategory::Bad => StandingCategory::Hostile,
            StandingCategory::Hostile => StandingCategory::Hostile,
        }
    }
}

/// Multi-dimensional standing calculator
struct MultiDimensionalStandingCalculator {
    dimensions: Vec<String>,
}

impl MultiDimensionalStandingCalculator {
    fn new(dimensions: Vec<String>) -> Self {
        Self { dimensions }
    }
}

#[async_trait]
impl StandingCalculator for MultiDimensionalStandingCalculator {
    async fn calculate_standing(
        &self,
        history: &[InteractionRecord],
        current_standing: &RelationshipStanding,
    ) -> Result<RelationshipStanding> {
        let mut values = match current_standing {
            RelationshipStanding::MultiDimensional(v) => v.clone(),
            _ => {
                let mut v = HashMap::new();
                for dim in &self.dimensions {
                    v.insert(dim.clone(), 0.0);
                }
                v
            }
        };

        // Update each dimension based on interaction type
        for record in history {
            match &record.interaction_type {
                InteractionType::TrustBuilding => {
                    *values.get_mut("trust").unwrap_or(&mut 0.0) += record.standing_impact;
                }
                InteractionType::Cooperation => {
                    *values.get_mut("cooperation").unwrap_or(&mut 0.0) += record.standing_impact;
                }
                InteractionType::EmotionalSupport => {
                    *values.get_mut("emotional").unwrap_or(&mut 0.0) += record.standing_impact;
                }
                _ => {
                    // Distribute impact across all dimensions
                    for value in values.values_mut() {
                        *value += record.standing_impact / self.dimensions.len() as f32;
                    }
                }
            }
        }

        // Clamp values
        for value in values.values_mut() {
            *value = value.clamp(-1.0, 1.0);
        }

        Ok(RelationshipStanding::MultiDimensional(values))
    }

    async fn apply_decay(
        &self,
        standing: &RelationshipStanding,
        days_since_interaction: f32,
        config: &RelationshipConfig,
    ) -> Result<RelationshipStanding> {
        match standing {
            RelationshipStanding::MultiDimensional(values) => {
                let mut new_values = values.clone();
                let decay = config.decay.daily_decay_rate * days_since_interaction;

                for value in new_values.values_mut() {
                    *value = (*value - decay).max(config.decay.minimum_standing);
                }

                Ok(RelationshipStanding::MultiDimensional(new_values))
            }
            _ => Ok(standing.clone()),
        }
    }

    fn to_category(&self, standing: &RelationshipStanding) -> StandingCategory {
        match standing {
            RelationshipStanding::MultiDimensional(values) => {
                let avg: f32 = values.values().sum::<f32>() / values.len() as f32;

                if avg > 0.8 {
                    StandingCategory::Excellent
                } else if avg > 0.5 {
                    StandingCategory::Good
                } else if avg > 0.0 {
                    StandingCategory::Neutral
                } else if avg > -0.5 {
                    StandingCategory::Poor
                } else if avg > -0.8 {
                    StandingCategory::Bad
                } else {
                    StandingCategory::Hostile
                }
            }
            _ => StandingCategory::Neutral,
        }
    }
}

/// In-memory relationship storage
struct InMemoryRelationshipStorage {
    data: Arc<RwLock<HashMap<Uuid, Relationship>>>,
}

impl InMemoryRelationshipStorage {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RelationshipStorage for InMemoryRelationshipStorage {
    async fn save(&self, relationship: &Relationship) -> Result<()> {
        self.data
            .write()
            .await
            .insert(relationship.id, relationship.clone());
        Ok(())
    }

    async fn load(&self, id: Uuid) -> Result<Option<Relationship>> {
        Ok(self.data.read().await.get(&id).cloned())
    }

    async fn find_by_participants(
        &self,
        participant_a: &str,
        participant_b: &str,
    ) -> Result<Option<Relationship>> {
        let data = self.data.read().await;

        for relationship in data.values() {
            if (relationship.participant_a == participant_a
                && relationship.participant_b == participant_b)
                || (relationship.participant_a == participant_b
                    && relationship.participant_b == participant_a)
            {
                return Ok(Some(relationship.clone()));
            }
        }

        Ok(None)
    }

    async fn list_for_participant(&self, participant: &str) -> Result<Vec<Relationship>> {
        let data = self.data.read().await;

        let relationships = data
            .values()
            .filter(|r| r.participant_a == participant || r.participant_b == participant)
            .cloned()
            .collect();

        Ok(relationships)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        Ok(self.data.write().await.remove(&id).is_some())
    }
}
