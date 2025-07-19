//! Advanced memory system with knowledge graphs

use async_trait::async_trait;
use emotional_agents_core::memory::*;
use emotional_agents_core::*;
use petgraph::algo::dijkstra;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::MemoryConfig;

/// Knowledge graph memory system
pub struct KnowledgeGraphMemory {
    /// Configuration
    config: MemoryConfig,

    /// Knowledge graphs per context
    graphs: Arc<RwLock<HashMap<Uuid, KnowledgeGraph>>>,

    /// Memory storage
    storage: Box<dyn MemoryStorage>,

    /// Embeddings cache
    embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

/// Knowledge graph for a specific context
pub struct KnowledgeGraph {
    /// The actual graph
    graph: DiGraph<KnowledgeNode, KnowledgeEdge>,

    /// Node index mapping
    node_map: HashMap<String, NodeIndex>,

    /// Context ID
    context_id: Uuid,

    /// Graph metadata
    metadata: GraphMetadata,
}

/// Node in the knowledge graph
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeNode {
    /// Node ID
    pub id: String,

    /// Node type
    pub node_type: KnowledgeNodeType,

    /// Node content
    pub content: String,

    /// Importance score
    pub importance: f32,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,

    /// Access count
    pub access_count: u32,

    /// Embeddings
    pub embedding: Option<Vec<f32>>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of knowledge nodes
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KnowledgeNodeType {
    /// Concept or idea
    Concept,

    /// Person
    Person,

    /// Event
    Event,

    /// Fact
    Fact,

    /// Emotion
    Emotion,

    /// Location
    Location,

    /// Object
    Object,

    /// Action
    Action,

    /// Relationship
    Relationship,

    /// Custom type
    Custom(String),
}

/// Edge in the knowledge graph
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEdge {
    /// Edge type
    pub edge_type: KnowledgeEdgeType,

    /// Edge strength (0.0 to 1.0)
    pub strength: f32,

    /// Confidence in this connection
    pub confidence: f32,

    /// Evidence for this edge
    pub evidence: Vec<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of knowledge edges
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KnowledgeEdgeType {
    /// Is-a relationship
    IsA,

    /// Has-a relationship
    HasA,

    /// Part-of relationship
    PartOf,

    /// Causes
    Causes,

    /// Caused-by
    CausedBy,

    /// Related-to
    RelatedTo,

    /// Follows
    Follows,

    /// Precedes
    Precedes,

    /// Contradicts
    Contradicts,

    /// Supports
    Supports,

    /// Emotional connection
    EmotionalConnection,

    /// Temporal connection
    TemporalConnection,

    /// Spatial connection
    SpatialConnection,

    /// Custom edge type
    Custom(String),
}

/// Graph metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphMetadata {
    /// Total nodes
    pub node_count: usize,

    /// Total edges
    pub edge_count: usize,

    /// Average node degree
    pub avg_degree: f32,

    /// Graph density
    pub density: f32,

    /// Most connected nodes
    pub hubs: Vec<String>,

    /// Last update
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Memory storage trait
#[async_trait]
pub trait MemoryStorage: Send + Sync {
    /// Store a memory
    async fn store(&self, context_id: Uuid, memory: Memory) -> Result<MemoryId>;

    /// Retrieve memories
    async fn retrieve(&self, context_id: Uuid, query: &MemoryQuery) -> Result<Vec<MemoryResult>>;

    /// Update a memory
    async fn update(&self, context_id: Uuid, memory: Memory) -> Result<()>;

    /// Delete a memory
    async fn delete(&self, context_id: Uuid, memory_id: MemoryId) -> Result<bool>;

    /// Count entries
    async fn count(&self, context_id: Uuid) -> Result<usize>;

    /// Save graph
    async fn save_graph(&self, context_id: Uuid, graph_data: Vec<u8>) -> Result<()>;

    /// Load graph
    async fn load_graph(&self, context_id: Uuid) -> Result<Option<Vec<u8>>>;
}

impl KnowledgeGraphMemory {
    /// Create new knowledge graph memory system
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        let storage: Box<dyn MemoryStorage> = match config.storage.storage_type.as_str() {
            "memory" => Box::new(InMemoryStorage::new()),
            // TODO: Add postgres support when feature is enabled
            // #[cfg(feature = "postgres")]
            // "postgres" => Box::new(PostgresStorage::new(&config.storage).await?),
            _ => return Err("Unsupported storage type".into()),
        };

        Ok(Self {
            config,
            graphs: Arc::new(RwLock::new(HashMap::new())),
            storage,
            embeddings: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new context
    pub async fn create_context(&self, _agent_id: AgentId) -> Result<Uuid> {
        let context_id = Uuid::new_v4();
        let graph = KnowledgeGraph::new(context_id);

        self.graphs.write().await.insert(context_id, graph);

        Ok(context_id)
    }

    /// Store a message in memory
    pub async fn store_message(&self, context_id: Uuid, message: Message) -> Result<MemoryId> {
        // Create memory from message
        let memory = self.message_to_memory(message).await?;

        // Store in storage backend
        let memory_id = self.storage.store(context_id, memory.clone()).await?;

        // Add to knowledge graph
        self.add_to_graph(context_id, &memory).await?;

        Ok(memory_id)
    }

    /// Retrieve relevant memories
    pub async fn retrieve_relevant(
        &self,
        context_id: Uuid,
        message: &Message,
        limit: usize,
    ) -> Result<Vec<MemoryResult>> {
        // Create query from message
        let query = MemoryQuery {
            query: message.text_content(),
            memory_types: None,
            tags: None,
            min_importance: Some(0.3),
            max_age_days: None,
            limit,
            include_emotional: true,
        };

        // Get from storage
        let mut results = self.storage.retrieve(context_id, &query).await?;

        // Enhance with graph connections
        self.enhance_with_graph_connections(context_id, &mut results)
            .await?;

        Ok(results)
    }

    /// Update knowledge graph with new information
    pub async fn update_knowledge_graph(
        &self,
        context_id: Uuid,
        message: &Message,
        responses: &[Message],
    ) -> Result<()> {
        let mut graphs = self.graphs.write().await;
        let graph = graphs.get_mut(&context_id).ok_or("Graph not found")?;

        // Extract entities and concepts
        let entities = self.extract_entities(message).await?;
        let concepts = self.extract_concepts(responses).await?;

        // Add nodes
        for entity in entities {
            graph.add_node(entity)?;
        }

        for concept in concepts {
            graph.add_node(concept)?;
        }

        // Infer relationships
        if self.config.knowledge_graph.auto_infer_relationships {
            graph.infer_relationships(self.config.knowledge_graph.similarity_threshold)?;
        }

        // Update metadata
        graph.update_metadata();

        // Persist graph
        let graph_data = bincode::serialize(&graph)?;
        self.storage.save_graph(context_id, graph_data).await?;

        Ok(())
    }

    /// Convert message to memory
    async fn message_to_memory(&self, message: Message) -> Result<Memory> {
        let importance = self.calculate_importance(&message)?;

        let memory = MemoryBuilder::new(message.text_content(), MemoryType::Episodic)
            .importance(importance)
            .add_metadata("message_id".to_string(), serde_json::json!(message.id))
            .add_metadata("role".to_string(), serde_json::json!(message.role))
            .build();

        Ok(memory)
    }

    /// Calculate importance of a message
    fn calculate_importance(&self, message: &Message) -> Result<f32> {
        // Simple heuristic - real implementation would be more sophisticated
        let mut importance = 0.5;

        // Emotional content increases importance
        if message.has_emotion() {
            importance += 0.2;
        }

        // User messages are more important
        if message.role == MessageRole::User {
            importance += 0.1;
        }

        // Longer messages might be more important
        let length_factor = (message.text_content().len() as f32 / 500.0).min(0.2);
        importance += length_factor;

        Ok(importance.clamp(0.0, 1.0))
    }

    /// Add memory to knowledge graph
    async fn add_to_graph(&self, context_id: Uuid, memory: &Memory) -> Result<()> {
        let mut graphs = self.graphs.write().await;
        let graph = graphs.get_mut(&context_id).ok_or("Graph not found")?;

        // Create node from memory
        let node = KnowledgeNode {
            id: memory.id.to_string(),
            node_type: self.memory_type_to_node_type(&memory.memory_type),
            content: memory.content.clone(),
            importance: memory.importance,
            created_at: memory.created_at,
            last_accessed: memory.last_accessed,
            access_count: memory.access_count,
            embedding: None, // Would compute embedding here
            properties: memory.metadata.clone(),
        };

        graph.add_node(node)?;

        Ok(())
    }

    /// Convert memory type to node type
    fn memory_type_to_node_type(&self, memory_type: &MemoryType) -> KnowledgeNodeType {
        match memory_type {
            MemoryType::Episodic => KnowledgeNodeType::Event,
            MemoryType::Semantic => KnowledgeNodeType::Fact,
            MemoryType::Procedural => KnowledgeNodeType::Action,
            MemoryType::Emotional => KnowledgeNodeType::Emotion,
            MemoryType::Social => KnowledgeNodeType::Person,
            _ => KnowledgeNodeType::Concept,
        }
    }

    /// Enhance results with graph connections
    async fn enhance_with_graph_connections(
        &self,
        context_id: Uuid,
        results: &mut Vec<MemoryResult>,
    ) -> Result<()> {
        let graphs = self.graphs.read().await;
        let graph = graphs.get(&context_id).ok_or("Graph not found")?;

        for result in results.iter_mut() {
            let connections = graph.get_connections(&result.memory.id.to_string(), 2)?;

            // Add connection information to retrieval context
            result.retrieval_context = format!(
                "{} (Connected to: {})",
                result.retrieval_context,
                connections.join(", ")
            );
        }

        Ok(())
    }

    /// Extract entities from message
    async fn extract_entities(&self, message: &Message) -> Result<Vec<KnowledgeNode>> {
        // Simplified entity extraction - real implementation would use NER
        let mut entities = Vec::new();

        // Extract mentioned people (simplified)
        let text = message.text_content();
        for word in text.split_whitespace() {
            if word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                entities.push(KnowledgeNode {
                    id: Uuid::new_v4().to_string(),
                    node_type: KnowledgeNodeType::Person,
                    content: word.to_string(),
                    importance: 0.5,
                    created_at: chrono::Utc::now(),
                    last_accessed: chrono::Utc::now(),
                    access_count: 1,
                    embedding: None,
                    properties: HashMap::new(),
                });
            }
        }

        Ok(entities)
    }

    /// Extract concepts from responses
    async fn extract_concepts(&self, responses: &[Message]) -> Result<Vec<KnowledgeNode>> {
        let mut concepts = Vec::new();

        for response in responses {
            // Extract key concepts (simplified)
            let text = response.text_content();

            // Would use more sophisticated concept extraction
            concepts.push(KnowledgeNode {
                id: Uuid::new_v4().to_string(),
                node_type: KnowledgeNodeType::Concept,
                content: text
                    .split_whitespace()
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(" "),
                importance: 0.4,
                created_at: chrono::Utc::now(),
                last_accessed: chrono::Utc::now(),
                access_count: 1,
                embedding: None,
                properties: HashMap::new(),
            });
        }

        Ok(concepts)
    }

    /// Count memory entries
    pub async fn count_entries(&self, context_id: Uuid) -> Result<usize> {
        self.storage.count(context_id).await
    }

    /// Save agent state
    pub async fn save_agent_state(&self, context_id: Uuid, state: serde_json::Value) -> Result<()> {
        let memory = MemoryBuilder::new("Agent state snapshot".to_string(), MemoryType::Procedural)
            .importance(1.0)
            .add_metadata("state_data".to_string(), state)
            .add_metadata("snapshot_type".to_string(), serde_json::json!("full_state"))
            .build();

        self.storage.store(context_id, memory).await?;
        Ok(())
    }

    /// Shutdown memory system
    pub async fn shutdown(&self) -> Result<()> {
        // Save all graphs
        let graphs = self.graphs.read().await;
        for (context_id, graph) in graphs.iter() {
            let graph_data = bincode::serialize(graph)?;
            self.storage.save_graph(*context_id, graph_data).await?;
        }

        Ok(())
    }
}

impl KnowledgeGraph {
    /// Create new knowledge graph
    pub fn new(context_id: Uuid) -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            context_id,
            metadata: GraphMetadata {
                node_count: 0,
                edge_count: 0,
                avg_degree: 0.0,
                density: 0.0,
                hubs: Vec::new(),
                last_updated: chrono::Utc::now(),
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: KnowledgeNode) -> Result<NodeIndex> {
        let node_id = node.id.clone();
        let index = self.graph.add_node(node);
        self.node_map.insert(node_id, index);
        self.update_metadata();
        Ok(index)
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, from_id: &str, to_id: &str, edge: KnowledgeEdge) -> Result<()> {
        let from_idx = *self.node_map.get(from_id).ok_or("Source node not found")?;
        let to_idx = *self.node_map.get(to_id).ok_or("Target node not found")?;

        self.graph.add_edge(from_idx, to_idx, edge);
        self.update_metadata();

        Ok(())
    }

    /// Get connections for a node
    pub fn get_connections(&self, node_id: &str, depth: usize) -> Result<Vec<String>> {
        let node_idx = *self.node_map.get(node_id).ok_or("Node not found")?;

        // Use Dijkstra to find all nodes within depth
        let distances = dijkstra(&self.graph, node_idx, None, |_| 1);

        let connections: Vec<String> = distances
            .into_iter()
            .filter(|(_, dist)| *dist <= depth as i32 && *dist > 0)
            .filter_map(|(idx, _)| self.graph.node_weight(idx).map(|node| node.content.clone()))
            .collect();

        Ok(connections)
    }

    /// Infer relationships between nodes
    pub fn infer_relationships(&mut self, threshold: f32) -> Result<()> {
        let node_indices: Vec<_> = self.node_map.values().cloned().collect();

        for i in 0..node_indices.len() {
            for j in i + 1..node_indices.len() {
                let node_i = &self.graph[node_indices[i]];
                let node_j = &self.graph[node_indices[j]];

                // Calculate similarity (simplified)
                let similarity = self.calculate_similarity(node_i, node_j)?;

                if similarity > threshold {
                    let edge = KnowledgeEdge {
                        edge_type: KnowledgeEdgeType::RelatedTo,
                        strength: similarity,
                        confidence: 0.7,
                        evidence: vec!["Automatic inference".to_string()],
                        created_at: chrono::Utc::now(),
                        metadata: HashMap::new(),
                    };

                    self.graph.add_edge(node_indices[i], node_indices[j], edge);
                }
            }
        }

        self.update_metadata();
        Ok(())
    }

    /// Calculate similarity between nodes
    fn calculate_similarity(&self, node1: &KnowledgeNode, node2: &KnowledgeNode) -> Result<f32> {
        // Simplified similarity - real implementation would use embeddings
        let content_similarity =
            if node1.content.contains(&node2.content) || node2.content.contains(&node1.content) {
                0.8
            } else {
                0.2
            };

        let type_similarity = if node1.node_type == node2.node_type {
            0.5
        } else {
            0.0
        };

        Ok((content_similarity + type_similarity) / 2.0)
    }

    /// Update graph metadata
    pub fn update_metadata(&mut self) {
        let node_count = self.graph.node_count();
        let edge_count = self.graph.edge_count();

        let avg_degree = if node_count > 0 {
            (2.0 * edge_count as f32) / node_count as f32
        } else {
            0.0
        };

        let max_possible_edges = node_count * (node_count - 1) / 2;
        let density = if max_possible_edges > 0 {
            edge_count as f32 / max_possible_edges as f32
        } else {
            0.0
        };

        // Find hubs (nodes with most connections)
        let mut node_degrees: Vec<(String, usize)> = self
            .node_map
            .iter()
            .map(|(id, idx)| {
                let degree = self.graph.edges(*idx).count();
                (id.clone(), degree)
            })
            .collect();

        node_degrees.sort_by(|a, b| b.1.cmp(&a.1));
        let hubs = node_degrees.into_iter().take(5).map(|(id, _)| id).collect();

        self.metadata = GraphMetadata {
            node_count,
            edge_count,
            avg_degree,
            density,
            hubs,
            last_updated: chrono::Utc::now(),
        };
    }
}

/// In-memory storage implementation
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Uuid, Vec<Memory>>>>,
    graphs: Arc<RwLock<HashMap<Uuid, Vec<u8>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            graphs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MemoryStorage for InMemoryStorage {
    async fn store(&self, context_id: Uuid, memory: Memory) -> Result<MemoryId> {
        let memory_id = memory.id;
        self.data
            .write()
            .await
            .entry(context_id)
            .or_insert_with(Vec::new)
            .push(memory);
        Ok(memory_id)
    }

    async fn retrieve(&self, context_id: Uuid, query: &MemoryQuery) -> Result<Vec<MemoryResult>> {
        let data = self.data.read().await;
        let empty_vec = Vec::new();
        let memories = data.get(&context_id).unwrap_or(&empty_vec);

        let mut results: Vec<MemoryResult> = memories
            .iter()
            .filter(|m| {
                // Apply filters
                if let Some(min_imp) = query.min_importance {
                    if m.importance < min_imp {
                        return false;
                    }
                }

                if let Some(types) = &query.memory_types {
                    if !types.contains(&m.memory_type) {
                        return false;
                    }
                }

                true
            })
            .map(|m| MemoryResult {
                memory: m.clone(),
                relevance: 0.5, // Would calculate real relevance
                retrieval_context: "In-memory retrieval".to_string(),
            })
            .collect();

        // Sort by relevance
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        // Apply limit
        results.truncate(query.limit);

        Ok(results)
    }

    async fn update(&self, context_id: Uuid, memory: Memory) -> Result<()> {
        let mut data = self.data.write().await;
        if let Some(memories) = data.get_mut(&context_id) {
            if let Some(existing) = memories.iter_mut().find(|m| m.id == memory.id) {
                *existing = memory;
            }
        }
        Ok(())
    }

    async fn delete(&self, context_id: Uuid, memory_id: MemoryId) -> Result<bool> {
        let mut data = self.data.write().await;
        if let Some(memories) = data.get_mut(&context_id) {
            let len_before = memories.len();
            memories.retain(|m| m.id != memory_id);
            Ok(memories.len() < len_before)
        } else {
            Ok(false)
        }
    }

    async fn count(&self, context_id: Uuid) -> Result<usize> {
        let data = self.data.read().await;
        Ok(data.get(&context_id).map(|m| m.len()).unwrap_or(0))
    }

    async fn save_graph(&self, context_id: Uuid, graph_data: Vec<u8>) -> Result<()> {
        self.graphs.write().await.insert(context_id, graph_data);
        Ok(())
    }

    async fn load_graph(&self, context_id: Uuid) -> Result<Option<Vec<u8>>> {
        Ok(self.graphs.read().await.get(&context_id).cloned())
    }
}

// Placeholder for bincode - would need to add to dependencies
mod bincode {
    use super::*;

    pub fn serialize<T>(_value: &T) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    pub fn deserialize<T>(_data: &[u8]) -> Result<T> {
        Err("Not implemented".into())
    }
}
