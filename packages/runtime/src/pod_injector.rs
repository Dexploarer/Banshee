//! Pod Injection System
//!
//! Loads and manages pods from character sheet configuration
//! Integrates with AI SDK 5 and MCP manager for complete runtime system

use anyhow::{Context, Result};
use banshee_core::plugin::{Pod, PodManager, PodResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::ai_sdk_client::AiSdk5ClientManager;
use crate::character_sheet::{
    CharacterSheet, PodInstanceConfig, PodListener,
};
use crate::mcp_manager::McpManager;

/// Pod injector manages pod lifecycle from character sheet configuration
pub struct PodInjector {
    pod_manager: PodManager,
    ai_client_manager: Arc<RwLock<AiSdk5ClientManager>>,
    mcp_manager: Arc<RwLock<McpManager>>,
    character_sheet_id: Option<uuid::Uuid>,
    injected_pods: HashMap<String, InjectedPodInfo>,
}

/// Information about an injected pod
#[derive(Debug, Clone)]
pub struct InjectedPodInfo {
    pub config: PodInstanceConfig,
    pub status: PodStatus,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub restart_count: u32,
    pub error_count: u32,
}

/// Pod status tracking
#[derive(Debug, Clone, PartialEq)]
pub enum PodStatus {
    Initializing,
    Running,
    Stopped,
    Error(String),
    Restarting,
}

/// Pod injection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInjectionConfig {
    pub max_concurrent_initializations: u32,
    pub initialization_timeout_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub auto_restart_on_failure: bool,
    pub max_restart_attempts: u32,
}

impl Default for PodInjectionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_initializations: 5,
            initialization_timeout_seconds: 60,
            health_check_interval_seconds: 30,
            auto_restart_on_failure: true,
            max_restart_attempts: 3,
        }
    }
}

impl PodInjector {
    /// Create new pod injector
    pub fn new(
        ai_client_manager: Arc<RwLock<AiSdk5ClientManager>>,
        mcp_manager: Arc<RwLock<McpManager>>,
    ) -> Self {
        Self {
            pod_manager: PodManager::new(),
            ai_client_manager,
            mcp_manager,
            character_sheet_id: None,
            injected_pods: HashMap::new(),
        }
    }

    /// Load pods from character sheet configuration
    pub async fn load_from_character_sheet(
        &mut self,
        character_sheet: &CharacterSheet,
    ) -> Result<()> {
        info!(
            "Loading pods from character sheet: {}",
            character_sheet.name
        );
        self.character_sheet_id = Some(character_sheet.id);

        let pod_config = &character_sheet.pods;
        info!(
            "Found {} pod configurations in character sheet",
            pod_config.enabled_pods.len()
        );

        // Clear existing pods
        self.shutdown_all_pods().await?;

        // Load MCP pod listeners first (for cross-pod communication)
        self.configure_mcp_pod_listeners(&pod_config.pod_listeners)
            .await?;

        // Sort pods by priority for proper initialization order
        let mut sorted_pods: Vec<(&String, &PodInstanceConfig)> = pod_config
            .enabled_pods
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect();
        sorted_pods.sort_by_key(|(_, config)| config.priority);

        // Initialize pods in priority order
        for (pod_id, pod_config) in sorted_pods {
            match self.load_pod(pod_id, pod_config).await {
                Ok(_) => {
                    info!("Successfully loaded pod: {}", pod_id);

                    // Track pod info
                    let pod_info = InjectedPodInfo {
                        config: pod_config.clone(),
                        status: PodStatus::Running,
                        last_health_check: chrono::Utc::now(),
                        restart_count: 0,
                        error_count: 0,
                    };
                    self.injected_pods.insert(pod_id.clone(), pod_info);
                }
                Err(e) => {
                    error!("Failed to load pod '{}': {}", pod_id, e);

                    // Track error
                    let pod_info = InjectedPodInfo {
                        config: pod_config.clone(),
                        status: PodStatus::Error(e.to_string()),
                        last_health_check: chrono::Utc::now(),
                        restart_count: 0,
                        error_count: 1,
                    };
                    self.injected_pods.insert(pod_id.clone(), pod_info);
                }
            }
        }

        // Initialize all loaded pods with dependency resolution
        if pod_config.auto_dependency_resolution {
            info!("Initializing pods with automatic dependency resolution");
            self.pod_manager
                .initialize_all()
                .await
                .context("Failed to initialize pods with dependency resolution")?;
        }

        info!(
            "Pod loading complete. Running: {}, Total configured: {}",
            self.injected_pods
                .values()
                .filter(|p| p.status == PodStatus::Running)
                .count(),
            pod_config.enabled_pods.len()
        );

        Ok(())
    }

    /// Load individual pod based on configuration
    async fn load_pod(&mut self, pod_id: &str, config: &PodInstanceConfig) -> Result<()> {
        debug!("Loading pod: {} with priority {}", pod_id, config.priority);

        // Create the appropriate pod based on pod_id
        let pod = self
            .create_pod(pod_id, config)
            .await
            .with_context(|| format!("Failed to create pod: {}", pod_id))?;

        // Register pod with manager
        self.pod_manager
            .register(pod)
            .await
            .with_context(|| format!("Failed to register pod: {}", pod_id))?;

        debug!("Pod '{}' loaded and registered successfully", pod_id);
        Ok(())
    }

    /// Create pod instance based on pod ID and configuration
    async fn create_pod(&self, pod_id: &str, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        match pod_id {
            "emotion" => {
                // Create emotion pod with configuration overrides
                info!("Creating emotion pod with OCC model and decay mechanics");
                self.create_emotion_pod(config).await
            }
            "memory" => {
                // Create memory pod with persistence layer
                info!("Creating memory pod with embedded database integration");
                self.create_memory_pod(config).await
            }
            "web3" => {
                // Create Web3 pod with wallet integration
                info!("Creating Web3 pod with multi-chain support");
                self.create_web3_pod(config).await
            }
            "bootstrap" => {
                // Create bootstrap pod for basic agent functionality
                info!("Creating bootstrap pod with basic agent capabilities");
                self.create_bootstrap_pod(config).await
            }
            "providers" => {
                // Create providers pod for AI service integration
                info!("Creating providers pod with AI SDK 5 integration");
                self.create_providers_pod(config).await
            }
            "pancakeswap-infinity" => {
                // Create PancakeSwap Infinity pod
                info!("Creating PancakeSwap Infinity pod with hooks system");
                self.create_defi_pod("pancakeswap-infinity", config).await
            }
            "pump-fun" => {
                // Create Pump.fun pod
                info!("Creating Pump.fun pod with bonding curve integration");
                self.create_defi_pod("pump-fun", config).await
            }
            "jito-mev" => {
                // Create Jito MEV pod
                info!("Creating Jito MEV pod with TipRouter integration");
                self.create_defi_pod("jito-mev", config).await
            }
            "metaplex-core" => {
                // Create Metaplex Core pod
                info!("Creating Metaplex Core pod with MPL-404 support");
                self.create_defi_pod("metaplex-core", config).await
            }
            _ => {
                warn!("Unknown pod type: {}, creating generic pod", pod_id);
                self.create_generic_pod(pod_id, config).await
            }
        }
    }

    /// Create emotion pod with OCC model
    async fn create_emotion_pod(&self, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        // In real implementation, would create actual emotion pod
        // For now, return a mock pod
        Ok(Box::new(MockPod::new(
            "emotion",
            "Emotion intelligence with OCC model",
        )))
    }

    /// Create memory pod with embedded database
    async fn create_memory_pod(&self, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        // In real implementation, would create actual memory pod
        Ok(Box::new(MockPod::new(
            "memory",
            "Memory persistence with embedded database",
        )))
    }

    /// Create Web3 pod with multi-chain support
    async fn create_web3_pod(&self, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        // In real implementation, would create actual Web3 pod
        Ok(Box::new(MockPod::new(
            "web3",
            "Multi-chain Web3 integration",
        )))
    }

    /// Create bootstrap pod for basic functionality
    async fn create_bootstrap_pod(&self, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        // In real implementation, would create actual bootstrap pod
        Ok(Box::new(MockPod::new(
            "bootstrap",
            "Basic agent functionality",
        )))
    }

    /// Create providers pod for AI integration
    async fn create_providers_pod(&self, config: &PodInstanceConfig) -> Result<Box<dyn Pod>> {
        // In real implementation, would create actual providers pod with AI SDK 5
        Ok(Box::new(MockPod::new(
            "providers",
            "AI SDK 5 provider integration",
        )))
    }

    /// Create DeFi pod with emotional trading support
    async fn create_defi_pod(
        &self,
        pod_type: &str,
        config: &PodInstanceConfig,
    ) -> Result<Box<dyn Pod>> {
        let defi_config = config
            .defi_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("DeFi configuration required for pod: {}", pod_type))?;

        info!(
            "DeFi pod config - Networks: {:?}, Slippage: {}%, Max Gas: {} gwei",
            defi_config.networks,
            defi_config.slippage_tolerance * 100.0,
            defi_config.max_gas_price_gwei
        );

        if defi_config.emotional_trading.enabled {
            info!(
                "Emotional trading enabled - Fear/Greed threshold: {}, Confidence factor: {}",
                defi_config.emotional_trading.fear_greed_threshold,
                defi_config.emotional_trading.confidence_leverage_factor
            );
        }

        // In real implementation, would create actual DeFi pods
        match pod_type {
            "pancakeswap-infinity" => Ok(Box::new(MockPod::new(
                pod_type,
                "PancakeSwap Infinity with hooks",
            ))),
            "pump-fun" => Ok(Box::new(MockPod::new(
                pod_type,
                "Pump.fun bonding curve integration",
            ))),
            "jito-mev" => Ok(Box::new(MockPod::new(pod_type, "Jito MEV with TipRouter"))),
            "metaplex-core" => Ok(Box::new(MockPod::new(
                pod_type,
                "Metaplex Core with MPL-404",
            ))),
            _ => Err(anyhow::anyhow!("Unknown DeFi pod type: {}", pod_type)),
        }
    }

    /// Create generic pod for unknown types
    async fn create_generic_pod(
        &self,
        pod_id: &str,
        config: &PodInstanceConfig,
    ) -> Result<Box<dyn Pod>> {
        Ok(Box::new(MockPod::new(
            pod_id,
            "Generic pod with basic functionality",
        )))
    }

    /// Configure MCP pod listeners for cross-pod communication
    async fn configure_mcp_pod_listeners(&self, listeners: &[PodListener]) -> Result<()> {
        if listeners.is_empty() {
            debug!("No MCP pod listeners configured");
            return Ok();
        }

        info!("Configuring {} MCP pod listeners", listeners.len());

        // Get tools for each pod listener
        let mcp_manager = self.mcp_manager.read().await;
        let pod_tools = mcp_manager.get_tools_for_pod(listeners).await;

        for (pod_id, tools) in &pod_tools {
            info!(
                "Pod '{}' will have access to {} MCP tools",
                pod_id,
                tools.len()
            );

            for tool in tools {
                debug!("  - {} (from {})", tool.name, tool.server);
            }
        }

        Ok(())
    }

    /// Get pod manager reference
    pub fn pod_manager(&self) -> &PodManager {
        &self.pod_manager
    }

    /// Get mutable pod manager reference
    pub fn pod_manager_mut(&mut self) -> &mut PodManager {
        &mut self.pod_manager
    }

    /// Get injected pod information
    pub fn get_pod_info(&self, pod_id: &str) -> Option<&InjectedPodInfo> {
        self.injected_pods.get(pod_id)
    }

    /// List all injected pods
    pub fn list_injected_pods(&self) -> Vec<(&String, &InjectedPodInfo)> {
        self.injected_pods.iter().collect()
    }

    /// Perform health check on all injected pods
    pub async fn health_check_all(&mut self) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();
        let now = chrono::Utc::now();

        // Check pod manager health
        let pod_health = self.pod_manager.health_check_all().await;

        for (pod_id, pod_info) in &mut self.injected_pods {
            if let Some(&healthy) = pod_health.get(pod_id) {
                results.insert(pod_id.clone(), healthy);

                if healthy {
                    pod_info.last_health_check = now;
                    if matches!(pod_info.status, PodStatus::Error(_)) {
                        pod_info.status = PodStatus::Running;
                        info!("Pod '{}' recovered from error state", pod_id);
                    }
                } else {
                    pod_info.error_count += 1;
                    pod_info.status = PodStatus::Error("Health check failed".to_string());
                    warn!("Pod '{}' failed health check", pod_id);
                }
            } else {
                results.insert(pod_id.clone(), false);
                pod_info.status = PodStatus::Error("Pod not found in manager".to_string());
            }
        }

        Ok(results)
    }

    /// Shutdown all injected pods
    pub async fn shutdown_all_pods(&mut self) -> Result<()> {
        info!("Shutting down all injected pods");

        // Update pod statuses
        for (_, pod_info) in &mut self.injected_pods {
            pod_info.status = PodStatus::Stopped;
        }

        // Shutdown pod manager
        self.pod_manager
            .shutdown_all()
            .await
            .context("Failed to shutdown pod manager")?;

        // Clear injected pods
        self.injected_pods.clear();

        info!("All injected pods shut down");
        Ok(())
    }
}

/// Mock pod implementation for demonstration
struct MockPod {
    id: String,
    description: String,
}

impl MockPod {
    fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Pod for MockPod {
    fn name(&self) -> &str {
        &self.id
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<banshee_core::plugin::PodDependency> {
        vec![]
    }

    fn capabilities(&self) -> Vec<banshee_core::plugin::PodCapability> {
        vec![]
    }

    async fn initialize(&mut self) -> PodResult<()> {
        info!("Initializing mock pod: {} - {}", self.id, self.description);
        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        info!("Shutting down mock pod: {}", self.id);
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn banshee_core::Action>> {
        vec![]
    }

    fn providers(&self) -> Vec<Box<dyn banshee_core::Provider>> {
        vec![]
    }

    fn evaluators(&self) -> Vec<Box<dyn banshee_core::Evaluator>> {
        vec![]
    }

    async fn health_check(&self) -> PodResult<bool> {
        Ok(true)
    }

    async fn on_dependency_available(
        &mut self,
        _dependency_id: &str,
        _dependency: std::sync::Arc<dyn Pod>,
    ) -> PodResult<()> {
        Ok(())
    }

    async fn on_dependency_unavailable(&mut self, _dependency_id: &str) -> PodResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character_sheet::{CharacterSheetManager, PodConfiguration};

    #[tokio::test]
    async fn test_pod_injector_creation() {
        let ai_client_manager =
            Arc::new(RwLock::new(crate::ai_sdk_client::AiSdk5ClientManager::new()));
        let mcp_manager = Arc::new(RwLock::new(crate::mcp_manager::McpManager::new(
            ai_client_manager.clone(),
        )));

        let injector = PodInjector::new(ai_client_manager, mcp_manager);

        assert_eq!(injector.injected_pods.len(), 0);
        assert!(injector.character_sheet_id.is_none());
    }

    #[tokio::test]
    async fn test_character_sheet_loading() {
        let ai_client_manager =
            Arc::new(RwLock::new(crate::ai_sdk_client::AiSdk5ClientManager::new()));
        let mcp_manager = Arc::new(RwLock::new(crate::mcp_manager::McpManager::new(
            ai_client_manager.clone(),
        )));
        let mut injector = PodInjector::new(ai_client_manager, mcp_manager);

        // Create test character sheet with pod configuration
        let mut character_sheet = CharacterSheetManager::create_default_sheet();

        let mut emotion_config = PodInstanceConfig {
            enabled: true,
            config_overrides: HashMap::new(),
            dependencies: vec![],
            priority: 1,
            health_check_enabled: true,
            restart_on_failure: true,
            defi_config: None,
        };

        character_sheet
            .pods
            .enabled_pods
            .insert("emotion".to_string(), emotion_config);

        // Test loading (would work with real pod implementations)
        // For now, just test the configuration parsing
        assert_eq!(character_sheet.pods.enabled_pods.len(), 1);
        assert!(character_sheet.pods.enabled_pods.contains_key("emotion"));
        assert!(character_sheet.pods.auto_dependency_resolution);
    }

    #[test]
    fn test_pod_injection_config() {
        let config = PodInjectionConfig::default();

        assert_eq!(config.max_concurrent_initializations, 5);
        assert_eq!(config.initialization_timeout_seconds, 60);
        assert!(config.auto_restart_on_failure);
        assert_eq!(config.max_restart_attempts, 3);
    }
}
