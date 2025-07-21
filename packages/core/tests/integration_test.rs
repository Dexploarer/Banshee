//! Integration tests for the pod system

use banshee_core::{
    plugin::{Pod, PodCapability, PodConfig, PodDependency, PodResult, Version, VersionConstraint},
    action::{Action, ActionConfig, ActionRequest, ActionResult, ActionExample},
    provider::{Provider, ProviderConfig, ProviderResult},
    evaluator::{Evaluator, EvaluatorConfig, EvaluationResult},
    Context, Message, MessageRole, MessageContent,
};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// Test pod implementation
struct TestPod {
    config: PodConfig,
}

#[async_trait]
impl Pod for TestPod {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> Vec<PodCapability> {
        vec![
            PodCapability {
                name: "test_actions".to_string(),
                version: Version::new(1, 0, 0),
                description: "Test actions capability".to_string(),
            },
            PodCapability {
                name: "test_providers".to_string(),
                version: Version::new(1, 0, 0),
                description: "Test providers capability".to_string(),
            },
        ]
    }

    fn dependencies(&self) -> Vec<PodDependency> {
        self.config.dependencies.clone()
    }

    async fn initialize(&mut self) -> PodResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![Box::new(TestAction::new())]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![Box::new(TestProvider::new())]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        vec![Box::new(TestEvaluator::new())]
    }
}

/// Test action implementation
struct TestAction {
    config: ActionConfig,
}

impl TestAction {
    fn new() -> Self {
        Self {
            config: ActionConfig {
                name: "test_action".to_string(),
                description: "A test action".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    }
                }),
                output_schema: None,
                has_side_effects: false,
                emotional_impact: None,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Action for TestAction {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.config
    }

    async fn execute(&self, request: ActionRequest) -> banshee_core::Result<ActionResult> {
        let message = request.parameters
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        Ok(ActionResult {
            success: true,
            data: json!({ "echo": message }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, parameters: &HashMap<String, serde_json::Value>) -> banshee_core::Result<()> {
        if parameters.contains_key("message") {
            Ok(())
        } else {
            Err("Missing message parameter".into())
        }
    }

    async fn is_available(&self, _context: &Context) -> banshee_core::Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Echo a message".to_string(),
            parameters: [("message".to_string(), json!("Hello"))].into(),
            expected_output: json!({ "echo": "Hello" }),
        }]
    }
}

/// Test provider implementation
struct TestProvider {
    config: ProviderConfig,
}

impl TestProvider {
    fn new() -> Self {
        Self {
            config: ProviderConfig {
                name: "test_provider".to_string(),
                description: "A test provider".to_string(),
                priority: 100,
                enabled: true,
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Provider for TestProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn provide(&self, _context: &Context) -> banshee_core::Result<Vec<ProviderResult>> {
        Ok(vec![ProviderResult {
            provider: self.name().to_string(),
            data: json!({ "test": "data" }),
            relevance: 0.8,
            confidence: 1.0,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, _context: &Context) -> banshee_core::Result<bool> {
        Ok(true)
    }

    async fn initialize(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }
}

/// Test evaluator implementation
struct TestEvaluator {
    config: EvaluatorConfig,
}

impl TestEvaluator {
    fn new() -> Self {
        Self {
            config: EvaluatorConfig {
                name: "test_evaluator".to_string(),
                description: "A test evaluator".to_string(),
                frequency: banshee_core::evaluator::EvaluationFrequency::OnDemand,
                enabled: true,
                thresholds: HashMap::new(),
                settings: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Evaluator for TestEvaluator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &EvaluatorConfig {
        &self.config
    }

    async fn evaluate(&self, _context: &Context, _conversation: &[Message]) -> banshee_core::Result<EvaluationResult> {
        Ok(EvaluationResult {
            evaluator: self.name().to_string(),
            score: 0.8,
            insights: vec![],
            recommendations: vec![],
            alerts: vec![],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn should_evaluate(&self, _context: &Context) -> banshee_core::Result<bool> {
        Ok(true)
    }

    async fn initialize(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use banshee_core::EmotionalAgentsRuntime;

    #[tokio::test]
    async fn test_pod_registration() {
        let mut runtime = EmotionalAgentsRuntime::new();
        
        let pod = Box::new(TestPod {
            config: PodConfig {
                id: "test_pod".to_string(),
                name: "test_pod".to_string(),
                description: "A test pod".to_string(),
                version: Version::new(1, 0, 0),
                dependencies: vec![],
                provides: vec![
                    PodCapability {
                        name: "test_actions".to_string(),
                        version: Version::new(1, 0, 0),
                        description: "Test actions capability".to_string(),
                    },
                ],
                settings: HashMap::new(),
            },
        });

        let result = runtime.register_pod(pod).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_action_execution() {
        let action = TestAction::new();
        
        let request = ActionRequest {
            action_name: "test_action".to_string(),
            parameters: [("message".to_string(), json!("Hello, World!"))].into(),
            trigger_message: Message {
                id: uuid::Uuid::new_v4(),
                role: MessageRole::User,
                content: vec![MessageContent::Text { text: "test".to_string() }],
                name: None,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            context: Context::new(uuid::Uuid::new_v4(), "test_session".to_string()),
            metadata: HashMap::new(),
        };

        let result = action.execute(request).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["echo"], "Hello, World!");
    }

    #[tokio::test]
    async fn test_provider() {
        let provider = TestProvider::new();
        let context = Context::new(uuid::Uuid::new_v4(), "test_session".to_string());
        
        let results = provider.provide(&context).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "test_provider");
    }

    #[tokio::test]
    async fn test_evaluator() {
        let evaluator = TestEvaluator::new();
        let context = Context::new(uuid::Uuid::new_v4(), "test_session".to_string());
        let conversation = vec![];
        
        let result = evaluator.evaluate(&context, &conversation).await.unwrap();
        assert_eq!(result.evaluator, "test_evaluator");
        assert_eq!(result.score, 0.8);
    }

    #[tokio::test]
    async fn test_pod_dependencies() {
        let pod = TestPod {
            config: PodConfig {
                id: "dependent_pod".to_string(),
                name: "dependent_pod".to_string(),
                description: "A pod with dependencies".to_string(),
                version: Version::new(1, 0, 0),
                dependencies: vec![
                    PodDependency {
                        pod_id: "core".to_string(),
                        version: VersionConstraint::AtLeast(Version::new(0, 1, 0)),
                        optional: false,
                    },
                ],
                provides: vec![],
                settings: HashMap::new(),
            },
        };

        assert_eq!(pod.dependencies().len(), 1);
        assert_eq!(pod.dependencies()[0].pod_id, "core");
    }

    #[tokio::test]
    async fn test_pod_initialization_timeout() {
        use banshee_core::plugin::{PodManager, PodTimeoutConfig};
        use std::time::Duration;
        
        // Create a pod that takes too long to initialize
        struct SlowInitPod;
        
        #[async_trait]
        impl Pod for SlowInitPod {
            fn name(&self) -> &str {
                "slow_init_pod"
            }
            
            fn version(&self) -> &str {
                "1.0.0"
            }
            
            fn capabilities(&self) -> Vec<PodCapability> {
                vec![]
            }
            
            fn dependencies(&self) -> Vec<PodDependency> {
                vec![]
            }
            
            async fn initialize(&mut self) -> PodResult<()> {
                // Sleep longer than the timeout
                tokio::time::sleep(Duration::from_secs(5)).await;
                Ok(())
            }
            
            async fn shutdown(&mut self) -> PodResult<()> {
                Ok(())
            }
        }
        
        let timeout_config = PodTimeoutConfig {
            init_timeout_secs: 1, // 1 second timeout
            shutdown_timeout_secs: 1,
        };
        
        let mut manager = PodManager::with_timeout_config(timeout_config);
        let pod = Box::new(SlowInitPod);
        
        // Register should succeed
        manager.register(pod).await.unwrap();
        
        // Initialize should timeout
        let result = manager.initialize_all().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[tokio::test]
    async fn test_pod_shutdown_timeout() {
        use banshee_core::plugin::{PodManager, PodTimeoutConfig};
        use std::time::Duration;
        
        // Create a pod that takes too long to shutdown
        struct SlowShutdownPod {
            initialized: bool,
        }
        
        #[async_trait]
        impl Pod for SlowShutdownPod {
            fn name(&self) -> &str {
                "slow_shutdown_pod"
            }
            
            fn version(&self) -> &str {
                "1.0.0"
            }
            
            fn capabilities(&self) -> Vec<PodCapability> {
                vec![]
            }
            
            fn dependencies(&self) -> Vec<PodDependency> {
                vec![]
            }
            
            async fn initialize(&mut self) -> PodResult<()> {
                self.initialized = true;
                Ok(())
            }
            
            async fn shutdown(&mut self) -> PodResult<()> {
                // Sleep longer than the timeout
                tokio::time::sleep(Duration::from_secs(5)).await;
                Ok(())
            }
        }
        
        let timeout_config = PodTimeoutConfig {
            init_timeout_secs: 10,
            shutdown_timeout_secs: 1, // 1 second timeout
        };
        
        let mut manager = PodManager::with_timeout_config(timeout_config);
        let pod = Box::new(SlowShutdownPod { initialized: false });
        
        manager.register(pod).await.unwrap();
        manager.initialize_all().await.unwrap();
        
        // Shutdown should complete even with timeout (graceful degradation)
        let result = manager.shutdown_all().await;
        assert!(result.is_ok());
    }
}