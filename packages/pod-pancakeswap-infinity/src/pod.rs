//! Main pod implementation for PancakeSwap Infinity

use crate::{
    actions::{
        AddLiquidityAction, CreatePoolAction, FlashAccountingAction, HookAction,
        RemoveLiquidityAction, SwapAction,
    },
    config::PancakeSwapConfig,
    hooks::HookManager,
    providers::PancakeSwapProvider,
    types::EmotionalContext,
};
use async_trait::async_trait;
use banshee_core::{
    plugin::PodResult,
    plugin::{pod_capability, pod_dependency, PodCapability, PodDependency},
    Action, Evaluator, Pod, Provider,
};
use std::collections::HashMap;
use std::sync::Arc;

/// PancakeSwap Infinity Pod - Integrates the full PancakeSwap v4 ecosystem
/// with emotional intelligence and advanced DeFi features
pub struct PancakeSwapInfinityPod {
    config: PancakeSwapConfig,
    hook_manager: HookManager,
    provider: Option<Arc<PancakeSwapProvider>>,
    emotional_context: Option<EmotionalContext>,
    initialized: bool,
}

impl PancakeSwapInfinityPod {
    /// Create a new PancakeSwap Infinity pod with default configuration
    pub fn new() -> Self {
        Self::with_config(PancakeSwapConfig::default())
    }

    /// Create a new pod with custom configuration
    pub fn with_config(config: PancakeSwapConfig) -> Self {
        let hook_manager = HookManager::new();

        Self {
            config,
            hook_manager,
            provider: None,
            emotional_context: None,
            initialized: false,
        }
    }

    /// Update the emotional context for trading decisions
    pub fn set_emotional_context(&mut self, context: EmotionalContext) {
        self.emotional_context = Some(context);
    }

    /// Get the current emotional context
    pub fn get_emotional_context(&self) -> Option<&EmotionalContext> {
        self.emotional_context.as_ref()
    }

    /// Get reference to the hook manager
    pub fn hook_manager(&self) -> &HookManager {
        &self.hook_manager
    }

    /// Get mutable reference to the hook manager
    pub fn hook_manager_mut(&mut self) -> &mut HookManager {
        &mut self.hook_manager
    }

    /// Get the provider instance
    pub fn provider(&self) -> Option<&Arc<PancakeSwapProvider>> {
        self.provider.as_ref()
    }
}

#[async_trait]
impl Pod for PancakeSwapInfinityPod {
    fn name(&self) -> &str {
        "pancakeswap-infinity"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dependencies(&self) -> Vec<PodDependency> {
        vec![
            pod_dependency!("emotion", "1.0.0"),
            pod_dependency!("memory", "1.0.0"),
            pod_dependency!("web3", "1.0.0", optional),
        ]
    }

    fn capabilities(&self) -> Vec<PodCapability> {
        vec![
            pod_capability!(
                "defi_trading",
                "1.0.0",
                "Advanced DeFi trading with emotional intelligence"
            ),
            pod_capability!(
                "liquidity_provision",
                "1.0.0",
                "Optimal liquidity provision across AMM types"
            ),
            pod_capability!(
                "arbitrage_detection",
                "1.0.0",
                "Cross-pool arbitrage opportunity detection"
            ),
            pod_capability!(
                "mev_protection",
                "1.0.0",
                "MEV-resistant trading strategies"
            ),
            pod_capability!(
                "flash_accounting",
                "1.0.0",
                "Gas-optimized flash accounting operations"
            ),
            pod_capability!(
                "hook_system",
                "1.0.0",
                "Custom hook execution for advanced strategies"
            ),
            pod_capability!(
                "emotional_trading",
                "1.0.0",
                "Sentiment-based trading adjustments"
            ),
            pod_capability!(
                "risk_management",
                "1.0.0",
                "Automated risk management and position sizing"
            ),
        ]
    }

    async fn initialize(&mut self) -> PodResult<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("Initializing PancakeSwap Infinity pod...");

        // Initialize the provider
        let provider = Arc::new(PancakeSwapProvider::new(self.config.clone()));
        self.provider = Some(provider);

        self.initialized = true;

        tracing::info!(
            "PancakeSwap Infinity pod initialized successfully with {} chains configured",
            self.config.chains.len()
        );

        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        if !self.initialized {
            return Ok(());
        }

        tracing::info!("Shutting down PancakeSwap Infinity pod...");

        // Clean up resources
        self.provider = None;
        self.emotional_context = None;
        self.initialized = false;

        tracing::info!("PancakeSwap Infinity pod shutdown complete");

        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        if !self.initialized {
            return Vec::new();
        }

        vec![
            Box::new(SwapAction::new(None)),
            Box::new(AddLiquidityAction::new(None)),
            Box::new(RemoveLiquidityAction::new()),
            Box::new(CreatePoolAction::new()),
            Box::new(HookAction::new(HookManager::new())),
            Box::new(FlashAccountingAction::new()),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        if !self.initialized || self.provider.is_none() {
            return Vec::new();
        }

        vec![Box::new(self.provider.as_ref().unwrap().clone())]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        if !self.initialized {
            return Vec::new();
        }

        vec![Box::new(PancakeSwapEvaluator::new(
            self.config.clone(),
            self.provider.clone(),
        ))]
    }

    async fn health_check(&self) -> PodResult<bool> {
        if !self.initialized {
            return Ok(false);
        }

        // Simple health check - just verify we're initialized
        tracing::debug!("PancakeSwap pod health check passed");
        Ok(true)
    }

    async fn on_dependency_available(
        &mut self,
        dependency_id: &str,
        _dependency: Arc<dyn Pod>,
    ) -> PodResult<()> {
        match dependency_id {
            "emotion" => {
                tracing::info!("Emotion pod is now available, enabling emotional trading features");
            }
            "memory" => {
                tracing::info!(
                    "Memory pod is now available, enabling trading history and analytics"
                );
            }
            "web3" => {
                tracing::info!("Web3 pod is now available, enabling on-chain operations");
            }
            _ => {}
        }
        Ok(())
    }

    async fn on_dependency_unavailable(&mut self, dependency_id: &str) -> PodResult<()> {
        match dependency_id {
            "emotion" => {
                tracing::warn!(
                    "Emotion pod is no longer available, disabling emotional trading features"
                );
                self.emotional_context = None;
            }
            "memory" => {
                tracing::warn!(
                    "Memory pod is no longer available, trading history will not be persisted"
                );
            }
            "web3" => {
                tracing::warn!("Web3 pod is no longer available, on-chain operations disabled");
            }
            _ => {}
        }
        Ok(())
    }
}

/// Evaluator for PancakeSwap operations and strategies
pub struct PancakeSwapEvaluator {
    config: banshee_core::evaluator::EvaluatorConfig,
    _provider: Option<Arc<PancakeSwapProvider>>,
}

impl PancakeSwapEvaluator {
    pub fn new(_config: PancakeSwapConfig, provider: Option<Arc<PancakeSwapProvider>>) -> Self {
        let config = banshee_core::evaluator::EvaluatorConfig {
            name: "pancakeswap_strategy_evaluator".to_string(),
            description: "Evaluates trading strategies and opportunities on PancakeSwap Infinity"
                .to_string(),
            frequency: banshee_core::evaluator::EvaluationFrequency::OnDemand,
            enabled: true,
            thresholds: HashMap::new(),
            settings: HashMap::new(),
        };

        Self {
            config,
            _provider: provider,
        }
    }
}

#[async_trait]
impl Evaluator for PancakeSwapEvaluator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn config(&self) -> &banshee_core::evaluator::EvaluatorConfig {
        &self.config
    }

    async fn evaluate(
        &self,
        _context: &banshee_core::Context,
        _conversation: &[banshee_core::Message],
    ) -> banshee_core::Result<banshee_core::evaluator::EvaluationResult> {
        let result = banshee_core::evaluator::EvaluationResult {
            evaluator: self.name().to_string(),
            score: 0.8,
            insights: vec![],
            recommendations: vec![],
            alerts: vec![],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        Ok(result)
    }

    async fn should_evaluate(
        &self,
        _context: &banshee_core::Context,
    ) -> banshee_core::Result<bool> {
        Ok(true)
    }

    async fn initialize(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> banshee_core::Result<()> {
        Ok(())
    }
}

impl Default for PancakeSwapInfinityPod {
    fn default() -> Self {
        Self::new()
    }
}
