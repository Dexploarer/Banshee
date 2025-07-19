//! Integration tests for the runtime package

use emotional_agents_core::{
    AgentConfig, Character, EmotionalState, Message, MessageContent, MessageRole,
};
use emotional_agents_runtime::{
    CacheConfig, ConsolidationConfig, DecisionConfig, EmotionalAgentsRuntime,
    EmotionalDecisionEngine, KnowledgeGraphConfig, KnowledgeGraphMemory, MemoryConfig,
    RelationshipConfig, RelationshipDecayConfig, RelationshipManager, RuntimeConfig,
    StandingMethod, StorageConfig, TrustConfig,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_runtime_creation() {
    let config = RuntimeConfig::default();
    let result = EmotionalAgentsRuntime::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runtime_agent_creation() {
    let config = RuntimeConfig::default();
    let runtime = EmotionalAgentsRuntime::new(config).await.unwrap();

    let agent_config = AgentConfig {
        id: Some(Uuid::new_v4()),
        character: Character::new("Test Agent".to_string(), "A test agent".to_string()),
        initial_emotions: None,
        settings: HashMap::new(),
        enabled_plugins: vec![],
    };

    let result = runtime.create_agent(agent_config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_system() {
    let config = MemoryConfig {
        storage: StorageConfig {
            storage_type: "memory".to_string(),
            connection_string: None,
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600,
                max_size_mb: 100,
            },
        },
        knowledge_graph: KnowledgeGraphConfig {
            max_nodes: 1000,
            max_edges_per_node: 50,
            similarity_threshold: 0.7,
            auto_infer_relationships: true,
        },
        consolidation: ConsolidationConfig {
            enabled: true,
            interval_hours: 24,
            importance_threshold: 0.3,
        },
    };

    let memory = KnowledgeGraphMemory::new(config).await;
    assert!(memory.is_ok());

    let memory = memory.unwrap();
    let agent_id = Uuid::new_v4();
    let context = memory.create_context(agent_id).await;
    assert!(context.is_ok());
}

#[tokio::test]
async fn test_relationship_system() {
    let config = RelationshipConfig {
        standing_method: StandingMethod::Categorical,
        decay: RelationshipDecayConfig {
            enabled: true,
            daily_decay_rate: 0.01,
            minimum_standing: -0.5,
        },
        trust: TrustConfig {
            initial_trust: 0.5,
            gain_rate: 0.1,
            loss_multiplier: 2.0,
        },
    };

    let relationship_manager = RelationshipManager::new(config).await;
    assert!(relationship_manager.is_ok());

    let manager = relationship_manager.unwrap();
    let relationship = manager
        .get_or_create("agent1".to_string(), "user1".to_string())
        .await;
    assert!(relationship.is_ok());

    let rel = relationship.unwrap();
    assert_eq!(rel.participant_a, "agent1");
    assert_eq!(rel.participant_b, "user1");
}

#[tokio::test]
async fn test_decision_engine() {
    let memory_config = MemoryConfig {
        storage: StorageConfig {
            storage_type: "memory".to_string(),
            connection_string: None,
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600,
                max_size_mb: 100,
            },
        },
        knowledge_graph: KnowledgeGraphConfig {
            max_nodes: 1000,
            max_edges_per_node: 50,
            similarity_threshold: 0.7,
            auto_infer_relationships: true,
        },
        consolidation: ConsolidationConfig {
            enabled: true,
            interval_hours: 24,
            importance_threshold: 0.3,
        },
    };

    let relationship_config = RelationshipConfig {
        standing_method: StandingMethod::Categorical,
        decay: RelationshipDecayConfig {
            enabled: true,
            daily_decay_rate: 0.01,
            minimum_standing: -0.5,
        },
        trust: TrustConfig {
            initial_trust: 0.5,
            gain_rate: 0.1,
            loss_multiplier: 2.0,
        },
    };

    let decision_config = DecisionConfig {
        emotion_weight: 0.3,
        logic_weight: 0.3,
        memory_weight: 0.2,
        relationship_weight: 0.2,
        strategies: vec![],
    };

    let memory = KnowledgeGraphMemory::new(memory_config).await.unwrap();
    let relationships = RelationshipManager::new(relationship_config).await.unwrap();

    let decision_engine = EmotionalDecisionEngine::new(
        decision_config,
        std::sync::Arc::new(memory),
        std::sync::Arc::new(relationships),
    )
    .await;

    assert!(decision_engine.is_ok());
}

#[tokio::test]
async fn test_ai_sdk_client() {
    use emotional_agents_runtime::ai_sdk_client::{
        AiSdkClient, Config, GenerateRequest, Message, MessageContent,
    };

    let config = Config {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: "test-key".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        streaming: false,
        base_url: None,
    };

    let client = AiSdkClient::new(config).await;
    assert!(client.is_ok());

    // Test request serialization
    let message = Message {
        role: "user".to_string(),
        content: MessageContent::Text("Hello, world!".to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    let request = GenerateRequest {
        messages: vec![message],
        model: Some("gpt-4".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        tools: None,
        stream: None,
    };

    let serialized = serde_json::to_string(&request);
    assert!(serialized.is_ok());
    let json_str = serialized.unwrap();
    assert!(json_str.contains("Hello, world!"));
    assert!(json_str.contains("gpt-4"));
}

#[tokio::test]
async fn test_mcp_manager() {
    use emotional_agents_runtime::mcp_manager::{
        Config, McpManager, ServerConfig, TransportConfig,
    };

    let config = Config {
        servers: vec![ServerConfig {
            name: "test-server".to_string(),
            transport: TransportConfig::Http {
                url: "http://localhost:8080".to_string(),
                headers: HashMap::new(),
            },
            enabled_tools: Some(vec!["test_tool".to_string()]),
        }],
    };

    let manager = McpManager::new(config).await;
    assert!(manager.is_ok());

    let manager = manager.unwrap();
    let tools = manager.list_all_tools().await;
    assert!(tools.is_ok());
}

#[tokio::test]
async fn test_emotional_event_processing() {
    let config = RuntimeConfig::default();
    let runtime = EmotionalAgentsRuntime::new(config).await.unwrap();

    let agent_id = Uuid::new_v4();
    let agent_config = AgentConfig {
        id: Some(agent_id),
        character: Character::new("Test Agent".to_string(), "A test agent".to_string()),
        initial_emotions: None,
        settings: HashMap::new(),
        enabled_plugins: vec![],
    };

    // This test would need the runtime to be fully integrated
    // For now, just test that we can create the runtime
    assert!(runtime.create_agent(agent_config).await.is_ok());
}

#[tokio::test]
async fn test_knowledge_graph_operations() {
    let config = MemoryConfig {
        storage: StorageConfig {
            storage_type: "memory".to_string(),
            connection_string: None,
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600,
                max_size_mb: 100,
            },
        },
        knowledge_graph: KnowledgeGraphConfig {
            max_nodes: 1000,
            max_edges_per_node: 50,
            similarity_threshold: 0.7,
            auto_infer_relationships: true,
        },
        consolidation: ConsolidationConfig {
            enabled: true,
            interval_hours: 24,
            importance_threshold: 0.3,
        },
    };

    let memory = KnowledgeGraphMemory::new(config).await.unwrap();
    let agent_id = Uuid::new_v4();
    let context_id = memory.create_context(agent_id).await.unwrap();

    // Test storing a message
    let message = Message {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: vec![MessageContent::Text {
            text: "Hello, I'm testing the knowledge graph.".to_string(),
        }],
        name: Some("TestUser".to_string()),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let memory_id = memory.store_message(context_id, message.clone()).await;
    assert!(memory_id.is_ok());

    // Test retrieving relevant memories
    let relevant = memory.retrieve_relevant(context_id, &message, 5).await;
    assert!(relevant.is_ok());

    let memories = relevant.unwrap();
    assert!(!memories.is_empty());
}

#[tokio::test]
async fn test_relationship_interactions() {
    let config = RelationshipConfig {
        standing_method: StandingMethod::Numeric {
            min: -1.0,
            max: 1.0,
        },
        decay: RelationshipDecayConfig {
            enabled: true,
            daily_decay_rate: 0.01,
            minimum_standing: -0.5,
        },
        trust: TrustConfig {
            initial_trust: 0.5,
            gain_rate: 0.1,
            loss_multiplier: 2.0,
        },
    };

    let manager = RelationshipManager::new(config).await.unwrap();
    let relationship = manager
        .get_or_create("agent1".to_string(), "user1".to_string())
        .await
        .unwrap();

    // Test updating relationship from interaction
    let message = Message {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: vec![MessageContent::Text {
            text: "Thank you for your help! That was great.".to_string(),
        }],
        name: Some("user1".to_string()),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let emotional_state = EmotionalState::default();

    let result = manager
        .update_from_interaction(&relationship.id, &message, &emotional_state)
        .await;

    assert!(result.is_ok());

    // Check that the relationship was updated
    let updated_relationship = manager.get(&relationship.id).await.unwrap();
    assert!(updated_relationship.is_some());

    let rel = updated_relationship.unwrap();
    assert_eq!(rel.interaction_count, 1);
}
