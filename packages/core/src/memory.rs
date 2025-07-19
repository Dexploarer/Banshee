use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory identifier
pub type MemoryId = Uuid;

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique memory identifier
    pub id: MemoryId,

    /// Memory content
    pub content: String,

    /// Memory type
    pub memory_type: MemoryType,

    /// Importance score (0.0 to 1.0)
    pub importance: f32,

    /// Emotional content associated with memory
    pub emotional_content: HashMap<String, f32>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// When memory was created
    pub created_at: DateTime<Utc>,

    /// Last time memory was accessed
    pub last_accessed: DateTime<Utc>,

    /// Number of times accessed
    pub access_count: u32,

    /// Memory metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of memories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Episodic memory (events and experiences)
    Episodic,

    /// Semantic memory (facts and knowledge)
    Semantic,

    /// Procedural memory (skills and procedures)
    Procedural,

    /// Emotional memory (emotional associations)
    Emotional,

    /// Working memory (temporary information)
    Working,

    /// Autobiographical memory (personal experiences)
    Autobiographical,

    /// Social memory (information about others)
    Social,

    /// Custom memory type
    Custom(String),
}

/// Memory query for retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Query text
    pub query: String,

    /// Memory types to search
    pub memory_types: Option<Vec<MemoryType>>,

    /// Tags to filter by
    pub tags: Option<Vec<String>>,

    /// Minimum importance threshold
    pub min_importance: Option<f32>,

    /// Maximum age in days
    pub max_age_days: Option<u32>,

    /// Maximum number of results
    pub limit: usize,

    /// Include emotional content in results
    pub include_emotional: bool,
}

/// Memory retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    /// The memory
    pub memory: Memory,

    /// Relevance score to query (0.0 to 1.0)
    pub relevance: f32,

    /// Context why this memory was retrieved
    pub retrieval_context: String,
}

/// Memory storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum number of memories to store
    pub max_memories: Option<usize>,

    /// Maximum age before auto-deletion (days)
    pub max_age_days: Option<u32>,

    /// Minimum importance threshold for storage
    pub min_importance: f32,

    /// Enable emotional memory processing
    pub emotional_processing: bool,

    /// Auto-consolidation settings
    pub consolidation: ConsolidationConfig,
}

/// Memory consolidation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Enable automatic consolidation
    pub enabled: bool,

    /// How often to run consolidation (hours)
    pub frequency_hours: u32,

    /// Minimum access count for consolidation
    pub min_access_count: u32,

    /// Similarity threshold for merging memories
    pub similarity_threshold: f32,
}

/// Core memory provider trait
#[async_trait]
pub trait MemoryProvider: Send + Sync {
    /// Store a new memory
    async fn store(&mut self, memory: Memory) -> crate::Result<MemoryId>;

    /// Retrieve memories matching query
    async fn retrieve(&self, query: &MemoryQuery) -> crate::Result<Vec<MemoryResult>>;

    /// Get a specific memory by ID
    async fn get(&self, id: MemoryId) -> crate::Result<Option<Memory>>;

    /// Update an existing memory
    async fn update(&mut self, memory: Memory) -> crate::Result<()>;

    /// Delete a memory
    async fn delete(&mut self, id: MemoryId) -> crate::Result<bool>;

    /// Search memories by content
    async fn search(&self, text: &str, limit: usize) -> crate::Result<Vec<MemoryResult>>;

    /// Get memories by tags
    async fn get_by_tags(&self, tags: &[String], limit: usize) -> crate::Result<Vec<Memory>>;

    /// Get recent memories
    async fn get_recent(&self, limit: usize) -> crate::Result<Vec<Memory>>;

    /// Get most important memories
    async fn get_important(&self, limit: usize) -> crate::Result<Vec<Memory>>;

    /// Consolidate memories (merge similar ones)
    async fn consolidate(&mut self) -> crate::Result<u32>;

    /// Clean up old or unimportant memories
    async fn cleanup(&mut self, config: &MemoryConfig) -> crate::Result<u32>;
}

/// Memory importance calculator
pub trait ImportanceCalculator: Send + Sync {
    /// Calculate importance score for memory content
    fn calculate_importance(&self, content: &str, context: &str) -> f32;

    /// Update importance based on access patterns
    fn update_importance(&self, memory: &Memory, access_context: &str) -> f32;
}

/// Default importance calculator
pub struct DefaultImportanceCalculator {
    /// Keywords that increase importance
    pub important_keywords: Vec<String>,

    /// Emotional weight factor
    pub emotional_weight: f32,
}

impl ImportanceCalculator for DefaultImportanceCalculator {
    fn calculate_importance(&self, content: &str, context: &str) -> f32 {
        let mut importance = 0.5; // Base importance

        // Check for important keywords
        let content_lower = content.to_lowercase();
        for keyword in &self.important_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                importance += 0.1;
            }
        }

        // Context factors
        if context.contains("error") || context.contains("problem") {
            importance += 0.2;
        }

        if context.contains("success") || context.contains("achievement") {
            importance += 0.15;
        }

        // Length factor (longer content might be more important)
        let length_factor = (content.len() as f32 / 1000.0).min(0.2);
        importance += length_factor;

        importance.clamp(0.0, 1.0)
    }

    fn update_importance(&self, memory: &Memory, _access_context: &str) -> f32 {
        let mut importance = memory.importance;

        // Increase importance based on access frequency
        let access_boost = (memory.access_count as f32 * 0.01).min(0.2);
        importance += access_boost;

        // Recency boost
        let days_since_access = (Utc::now() - memory.last_accessed).num_days() as f32;
        if days_since_access < 1.0 {
            importance += 0.1;
        }

        importance.clamp(0.0, 1.0)
    }
}

/// Memory builder for easy construction
pub struct MemoryBuilder {
    content: String,
    memory_type: MemoryType,
    importance: f32,
    emotional_content: HashMap<String, f32>,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl MemoryBuilder {
    pub fn new(content: String, memory_type: MemoryType) -> Self {
        Self {
            content,
            memory_type,
            importance: 0.5,
            emotional_content: HashMap::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    pub fn add_emotion(mut self, emotion: String, intensity: f32) -> Self {
        self.emotional_content.insert(emotion, intensity);
        self
    }

    pub fn add_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn build(self) -> Memory {
        let now = Utc::now();
        Memory {
            id: Uuid::new_v4(),
            content: self.content,
            memory_type: self.memory_type,
            importance: self.importance,
            emotional_content: self.emotional_content,
            tags: self.tags,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: self.metadata,
        }
    }
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            memory_types: None,
            tags: None,
            min_importance: None,
            max_age_days: None,
            limit: 10,
            include_emotional: false,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memories: Some(10000),
            max_age_days: Some(365),
            min_importance: 0.1,
            emotional_processing: true,
            consolidation: ConsolidationConfig::default(),
        }
    }
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency_hours: 24,
            min_access_count: 2,
            similarity_threshold: 0.8,
        }
    }
}

impl Default for DefaultImportanceCalculator {
    fn default() -> Self {
        Self {
            important_keywords: vec![
                "important".to_string(),
                "critical".to_string(),
                "remember".to_string(),
                "note".to_string(),
                "key".to_string(),
                "essential".to_string(),
            ],
            emotional_weight: 1.5,
        }
    }
}
