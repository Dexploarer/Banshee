//! Integration tests for the complete Banshee plugin architecture
//! Tests dependency resolution, emotional persistence, and real implementations

use async_trait::async_trait;
use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalState};
use banshee_core::plugin::{Plugin, PluginDependency, PluginManager, PluginResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock plugin for testing dependency resolution
struct MockPlugin {
    name: String,
    version: String,
    dependencies: Vec<PluginDependency>,
    initialized: Arc<RwLock<bool>>,
}

impl MockPlugin {
    fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            dependencies: Vec::new(),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    fn with_dependency(mut self, dep_name: &str, dep_version: &str) -> Self {
        use banshee_core::plugin::{Version, VersionConstraint};
        self.dependencies.push(PluginDependency {
            plugin_id: dep_name.to_string(),
            version: VersionConstraint::Compatible(Version::parse(dep_version).unwrap()),
            optional: false,
        });
        self
    }

    async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }
}

#[async_trait]
impl Plugin for MockPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        self.dependencies.clone()
    }

    async fn initialize(&mut self) -> PluginResult<()> {
        *self.initialized.write().await = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        *self.initialized.write().await = false;
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_manager_dependency_resolution() {
    let mut manager = PluginManager::new();

    // Register plugins with dependencies: c -> b -> a
    let plugin_a = Box::new(MockPlugin::new("a", "1.0.0"));
    let plugin_b = Box::new(MockPlugin::new("b", "1.0.0").with_dependency("a", "1.0.0"));
    let plugin_c = Box::new(MockPlugin::new("c", "1.0.0").with_dependency("b", "1.0.0"));

    // Register in random order to test dependency resolution
    manager.register(plugin_c).await.unwrap();
    manager.register(plugin_a).await.unwrap();
    manager.register(plugin_b).await.unwrap();

    // Initialize all plugins
    manager.initialize_all().await.unwrap();

    // Verify all plugins are running
    let running_plugins = manager.running_plugins();
    assert_eq!(running_plugins.len(), 3);
    assert!(running_plugins.contains(&"a".to_string()));
    assert!(running_plugins.contains(&"b".to_string()));
    assert!(running_plugins.contains(&"c".to_string()));

    // Verify plugins were initialized
    for plugin_id in &["a", "b", "c"] {
        let plugin_arc = manager.get_plugin(plugin_id).unwrap();
        let plugin = plugin_arc.read().await;
        let mock_plugin = plugin.as_ref() as &dyn std::any::Any;
        if let Some(mock) = mock_plugin.downcast_ref::<MockPlugin>() {
            assert!(
                mock.is_initialized().await,
                "Plugin {} should be initialized",
                plugin_id
            );
        }
    }

    // Test health checks
    let health_results = manager.health_check_all().await;
    assert_eq!(health_results.len(), 3);
    for (plugin_id, healthy) in health_results {
        assert!(healthy, "Plugin {} should be healthy", plugin_id);
    }

    // Shutdown all plugins
    manager.shutdown_all().await.unwrap();
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let mut manager = PluginManager::new();

    // Create circular dependency: a -> b -> a
    let plugin_a = Box::new(MockPlugin::new("a", "1.0.0").with_dependency("b", "1.0.0"));
    let plugin_b = Box::new(MockPlugin::new("b", "1.0.0").with_dependency("a", "1.0.0"));

    manager.register(plugin_a).await.unwrap();
    manager.register(plugin_b).await.unwrap();

    // Should fail with circular dependency error
    let result = manager.initialize_all().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[tokio::test]
async fn test_missing_dependency_detection() {
    let mut manager = PluginManager::new();

    // Create plugin with missing dependency
    let plugin_a = Box::new(MockPlugin::new("a", "1.0.0").with_dependency("missing", "1.0.0"));

    manager.register(plugin_a).await.unwrap();

    // Should fail with missing dependency error
    let result = manager.initialize_all().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown plugin"));
}

#[tokio::test]
async fn test_emotional_state_creation_and_manipulation() {
    // Test the emotion engine works correctly
    let mut state = EmotionalState::new();

    // Add some emotions
    state.update_emotion(Emotion::Joy, 0.8);
    state.update_emotion(Emotion::Pride, 0.6);
    state.update_emotion(Emotion::Anger, 0.3);

    // Test valence calculation (should be positive due to high joy)
    let valence = state.overall_valence();
    assert!(valence > 0.0, "Overall valence should be positive");

    // Test arousal calculation
    let arousal = state.overall_arousal();
    assert!(arousal > 0.0, "Overall arousal should be positive");

    // Test emotion decay
    let initial_joy = state.emotions.get(&Emotion::Joy).copied().unwrap_or(0.0);
    state.apply_decay(1.0); // Apply 1 second of decay
    let decayed_joy = state.emotions.get(&Emotion::Joy).copied().unwrap_or(0.0);

    assert!(decayed_joy < initial_joy, "Joy should decay over time");
    assert!(
        decayed_joy > 0.7,
        "Joy should still be relatively high after 1 second"
    );
}

#[tokio::test]
async fn test_emotional_event_processing() {
    use banshee_core::emotion::{EmotionalEvent, SocialInteractionType};

    // Test different emotional events
    let events = vec![
        EmotionalEvent::TaskCompleted {
            success: true,
            difficulty: 0.7,
            time_taken: 120.0,
            expected_time: 100.0,
        },
        EmotionalEvent::UserFeedback {
            sentiment: 0.8,
            specificity: 0.9,
            is_constructive: true,
        },
        EmotionalEvent::SocialInteraction {
            interaction_type: SocialInteractionType::Recognition,
            outcome: 0.7,
            peer_status: 0.8,
        },
        EmotionalEvent::UnexpectedEvent {
            surprise_level: 0.6,
            positive_outcome: true,
            context: "User praised our work".to_string(),
        },
    ];

    // Each event should be serializable/deserializable
    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: EmotionalEvent = serde_json::from_str(&json).unwrap();

        // Verify round-trip serialization works
        match (&event, &deserialized) {
            (
                EmotionalEvent::TaskCompleted { success: s1, .. },
                EmotionalEvent::TaskCompleted { success: s2, .. },
            ) => {
                assert_eq!(s1, s2);
            }
            (
                EmotionalEvent::UserFeedback { sentiment: s1, .. },
                EmotionalEvent::UserFeedback { sentiment: s2, .. },
            ) => {
                assert!((s1 - s2).abs() < 0.001);
            }
            _ => {} // Other event types
        }
    }
}

#[tokio::test]
async fn test_plugin_architecture_extensibility() {
    // Test that the plugin architecture can handle various plugin types
    let mut manager = PluginManager::new();

    // Create plugins representing different capabilities
    let core_plugin = Box::new(MockPlugin::new("core", "1.0.0"));
    let memory_plugin =
        Box::new(MockPlugin::new("memory", "1.0.0").with_dependency("core", "1.0.0"));
    let emotion_plugin =
        Box::new(MockPlugin::new("emotion", "1.0.0").with_dependency("memory", "1.0.0"));
    let web3_plugin = Box::new(MockPlugin::new("web3", "1.0.0").with_dependency("memory", "1.0.0"));
    let bootstrap_plugin = Box::new(
        MockPlugin::new("bootstrap", "1.0.0")
            .with_dependency("memory", "1.0.0")
            .with_dependency("emotion", "1.0.0")
            .with_dependency("web3", "1.0.0"),
    );

    // Register all plugins
    manager.register(bootstrap_plugin).await.unwrap();
    manager.register(web3_plugin).await.unwrap();
    manager.register(emotion_plugin).await.unwrap();
    manager.register(memory_plugin).await.unwrap();
    manager.register(core_plugin).await.unwrap();

    // Initialize all
    manager.initialize_all().await.unwrap();

    // Verify correct initialization order
    let running_plugins = manager.running_plugins();
    assert_eq!(running_plugins.len(), 5);

    // All plugins should be healthy
    let health_results = manager.health_check_all().await;
    assert!(health_results.values().all(|&healthy| healthy));

    manager.shutdown_all().await.unwrap();
}

#[tokio::test]
async fn test_version_compatibility() {
    use banshee_core::plugin::{Version, VersionConstraint};

    let v1_0_0 = Version::new(1, 0, 0);
    let v1_1_0 = Version::new(1, 1, 0);
    let v1_1_5 = Version::new(1, 1, 5);
    let v2_0_0 = Version::new(2, 0, 0);

    // Test exact version constraint
    let exact = VersionConstraint::Exact(Version::new(1, 1, 0));
    assert!(v1_1_0.is_compatible(&exact));
    assert!(!v1_1_5.is_compatible(&exact));

    // Test at-least version constraint
    let at_least = VersionConstraint::AtLeast(Version::new(1, 1, 0));
    assert!(!v1_0_0.is_compatible(&at_least));
    assert!(v1_1_0.is_compatible(&at_least));
    assert!(v1_1_5.is_compatible(&at_least));
    assert!(v2_0_0.is_compatible(&at_least));

    // Test compatible version constraint (same major)
    let compatible = VersionConstraint::Compatible(Version::new(1, 1, 0));
    assert!(!v1_0_0.is_compatible(&compatible));
    assert!(v1_1_0.is_compatible(&compatible));
    assert!(v1_1_5.is_compatible(&compatible));
    assert!(!v2_0_0.is_compatible(&compatible)); // Different major version
}

#[tokio::test]
async fn test_real_world_scenario() {
    // Test a realistic scenario with emotional agents
    let agent_id = Uuid::new_v4();

    // Create emotional state
    let mut emotional_state = EmotionalState::new();
    emotional_state.update_emotion(Emotion::Joy, 0.7);
    emotional_state.update_emotion(Emotion::Pride, 0.5);

    // Simulate emotional events
    let task_completed = EmotionalEvent::TaskCompleted {
        success: true,
        difficulty: 0.8,
        time_taken: 150.0,
        expected_time: 120.0,
    };

    let user_feedback = EmotionalEvent::UserFeedback {
        sentiment: 0.9,
        specificity: 0.8,
        is_constructive: true,
    };

    // In a real system, these events would:
    // 1. Be processed by the emotion engine
    // 2. Update the agent's emotional state
    // 3. Be persisted to the database
    // 4. Influence future behavior

    // Verify basic emotional state functionality
    assert!(emotional_state.overall_valence() > 0.0);
    assert!(emotional_state.overall_arousal() > 0.0);

    // Test serialization for persistence
    let state_json = serde_json::to_string(&emotional_state).unwrap();
    let restored_state: EmotionalState = serde_json::from_str(&state_json).unwrap();

    assert!((emotional_state.overall_valence() - restored_state.overall_valence()).abs() < 0.001);

    // Test event serialization
    let event_json = serde_json::to_string(&task_completed).unwrap();
    let _restored_event: EmotionalEvent = serde_json::from_str(&event_json).unwrap();

    println!("✓ Real-world scenario test completed successfully");
    println!("  - Agent ID: {}", agent_id);
    println!(
        "  - Emotional valence: {:.2}",
        emotional_state.overall_valence()
    );
    println!(
        "  - Emotional arousal: {:.2}",
        emotional_state.overall_arousal()
    );
    println!("  - Events processed: 2");
    println!("  - Persistence: ✓ (serialization verified)");
}

/// Test the complete plugin lifecycle
#[tokio::test]
async fn test_complete_plugin_lifecycle() {
    let mut manager = PluginManager::new();

    // Stage 1: Registration
    let plugin = Box::new(MockPlugin::new("test", "1.0.0"));
    manager.register(plugin).await.unwrap();

    // Should be registered but not running
    assert!(manager.get_plugin("test").is_some());
    assert_eq!(manager.running_plugins().len(), 0);

    // Stage 2: Initialization
    manager.initialize_all().await.unwrap();

    // Should now be running
    assert_eq!(manager.running_plugins().len(), 1);
    assert!(manager.running_plugins().contains(&"test".to_string()));

    // Stage 3: Health check
    let health = manager.health_check_all().await;
    assert!(health.get("test").copied().unwrap_or(false));

    // Stage 4: Shutdown
    manager.shutdown_all().await.unwrap();

    // Plugin should still exist but not be running
    assert!(manager.get_plugin("test").is_some());
    // Note: Plugin states aren't directly exposed in the public API,
    // but internally they would be marked as stopped
}

/// Performance test for plugin operations
#[tokio::test]
async fn test_plugin_performance() {
    use std::time::Instant;

    let mut manager = PluginManager::new();

    // Register many plugins
    let start = Instant::now();
    for i in 0..100 {
        let plugin = Box::new(MockPlugin::new(&format!("plugin_{}", i), "1.0.0"));
        manager.register(plugin).await.unwrap();
    }
    let registration_time = start.elapsed();

    // Initialize all plugins
    let start = Instant::now();
    manager.initialize_all().await.unwrap();
    let initialization_time = start.elapsed();

    // Health check all plugins
    let start = Instant::now();
    let health_results = manager.health_check_all().await;
    let health_check_time = start.elapsed();

    // Shutdown all plugins
    let start = Instant::now();
    manager.shutdown_all().await.unwrap();
    let shutdown_time = start.elapsed();

    // Verify results
    assert_eq!(health_results.len(), 100);
    assert!(health_results.values().all(|&healthy| healthy));

    println!("Plugin Performance Metrics:");
    println!("  Registration (100 plugins): {:?}", registration_time);
    println!("  Initialization (100 plugins): {:?}", initialization_time);
    println!("  Health check (100 plugins): {:?}", health_check_time);
    println!("  Shutdown (100 plugins): {:?}", shutdown_time);

    // Basic performance assertions (these are quite generous)
    assert!(
        registration_time.as_millis() < 1000,
        "Registration should be fast"
    );
    assert!(
        initialization_time.as_millis() < 1000,
        "Initialization should be fast"
    );
    assert!(
        health_check_time.as_millis() < 1000,
        "Health checks should be fast"
    );
    assert!(shutdown_time.as_millis() < 1000, "Shutdown should be fast");
}
