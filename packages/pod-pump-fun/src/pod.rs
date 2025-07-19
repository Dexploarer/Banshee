//! Main Pump.fun pod implementation

use async_trait::async_trait;
use banshee_core::{
    plugin::{Pod, PodConfig, PodResult, Version, PodDependency, PodCapability, VersionConstraint},
    Action, Evaluator, Provider,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{actions::*, config::PumpFunConfig, providers::*};

/// Pump.fun pod for direct bonding curve integration
pub struct PumpFunPod {
    config: PodConfig,
    pump_config: PumpFunConfig,
    version_str: String,
}

impl PumpFunPod {
    pub fn new(pump_config: PumpFunConfig) -> Self {
        let version = Version::new(0, 1, 0);
        let version_str = version.to_string();
        Self {
            config: PodConfig {
                id: "pump-fun".to_string(),
                name: "Pump.fun Integration".to_string(),
                version,
                description: "Direct on-chain bonding curve integration with Pump.fun".to_string(),
                dependencies: vec![
                    banshee_core::pod_dependency!("web3", "1.0.0"), // Depends on Web3 pod for wallet
                    banshee_core::pod_dependency!("emotion", "0.1.0"), // Emotional trading
                ],
                provides: vec![
                    banshee_core::pod_capability!(
                        "bonding_curve_trading",
                        "0.1.0",
                        "Trade tokens on Pump.fun bonding curves"
                    ),
                    banshee_core::pod_capability!(
                        "token_creation",
                        "0.1.0",
                        "Create new tokens with bonding curves"
                    ),
                ],
                settings: HashMap::new(),
            },
            pump_config,
            version_str,
        }
    }

    pub fn with_config(mut self, key: &str, value: serde_json::Value) -> Self {
        self.config.settings.insert(key.to_string(), value);
        self
    }
}

impl Default for PumpFunPod {
    fn default() -> Self {
        Self::new(PumpFunConfig::default())
    }
}

#[async_trait]
impl Pod for PumpFunPod {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        &self.version_str
    }

    fn dependencies(&self) -> Vec<banshee_core::plugin::PodDependency> {
        self.config.dependencies.clone()
    }

    fn capabilities(&self) -> Vec<banshee_core::plugin::PodCapability> {
        self.config.provides.clone()
    }

    async fn initialize(&mut self) -> PodResult<()> {
        info!(
            "Initializing Pump.fun pod on {} network",
            match self.pump_config.network {
                crate::config::NetworkType::MainnetBeta => "mainnet",
                crate::config::NetworkType::Devnet => "devnet",
            }
        );

        // Validate configuration
        if self.pump_config.risk_params.max_position_size_sol.is_zero() {
            return Err("Invalid risk parameters: max position size cannot be zero".to_string());
        }

        if self.pump_config.emotional_trading.fear_threshold > 1.0
            || self.pump_config.emotional_trading.fear_threshold < 0.0
        {
            return Err(
                "Invalid emotional parameters: fear threshold must be between 0 and 1".to_string(),
            );
        }

        // Log configuration
        info!(
            "Risk limits - Max position: {} SOL, Max slippage: {}%",
            self.pump_config.risk_params.max_position_size_sol,
            self.pump_config.risk_params.max_slippage_percentage
        );

        info!(
            "Emotional trading - Fear threshold: {}, Excitement multiplier: {}x",
            self.pump_config.emotional_trading.fear_threshold,
            self.pump_config.emotional_trading.excitement_multiplier
        );

        if self.pump_config.auto_discovery {
            info!(
                "Auto-discovery enabled - Min liquidity: {} SOL, Max age: {} seconds",
                self.pump_config.min_liquidity_sol, self.pump_config.max_token_age_seconds
            );
        }

        info!("Pump.fun pod initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        info!("Shutting down Pump.fun pod");

        // Clean up any active connections or resources
        // In a real implementation, this would close RPC connections, cancel subscriptions, etc.

        info!("Pump.fun pod shutdown complete");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![
            Box::new(CreateTokenAction::new(self.pump_config.clone())),
            Box::new(BuyTokenAction::new(self.pump_config.clone())),
            Box::new(SellTokenAction::new(self.pump_config.clone())),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(TokenPriceProvider::new(self.pump_config.clone())),
            Box::new(TokenAnalyticsProvider::new(self.pump_config.clone())),
            Box::new(TokenDiscoveryProvider::new(self.pump_config.clone())),
        ]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        // No evaluators for now
        vec![]
    }

    async fn health_check(&self) -> PodResult<bool> {
        // In a real implementation, this would check:
        // 1. RPC connection health
        // 2. Wallet balance
        // 3. Recent transaction success rate

        Ok(true)
    }

    async fn on_dependency_available(
        &mut self,
        dependency_id: &str,
        _dependency: std::sync::Arc<dyn Pod>,
    ) -> PodResult<()> {
        match dependency_id {
            "web3" => {
                info!("Web3 pod available - wallet functionality enabled");
            }
            "emotion" => {
                info!("Emotion pod available - emotional trading strategies enabled");
            }
            _ => {}
        }
        Ok(())
    }

    async fn on_dependency_unavailable(&mut self, dependency_id: &str) -> PodResult<()> {
        match dependency_id {
            "web3" => {
                warn!("Web3 pod unavailable - trading disabled");
            }
            "emotion" => {
                warn!("Emotion pod unavailable - using default trading parameters");
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pod_initialization() {
        let mut pod = PumpFunPod::default();

        assert!(pod.initialize().await.is_ok());
        assert_eq!(pod.name(), "Pump.fun Integration");
        assert!(!pod.actions().is_empty());
        assert!(!pod.providers().is_empty());
        assert!(pod.health_check().await.unwrap());
        assert!(pod.shutdown().await.is_ok());
    }

    #[test]
    fn test_pod_configuration() {
        let config = PumpFunConfig::conservative();
        let pod = PumpFunPod::new(config.clone());

        assert_eq!(pod.config.id, "pump-fun");
        assert_eq!(
            pod.pump_config.risk_params.max_position_size_sol,
            config.risk_params.max_position_size_sol
        );
    }
}
