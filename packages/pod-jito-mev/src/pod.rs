//! Main Jito MEV pod implementation

use async_trait::async_trait;
use banshee_core::{
    plugin::{Pod, PodCapability, PodConfig, PodDependency, PodResult, Version, VersionConstraint},
    Action, Evaluator, Provider,
};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::{actions::*, config::JitoMevConfig, providers::*};

/// Jito MEV pod for maximum extractable value
pub struct JitoMevPod {
    config: PodConfig,
    jito_config: JitoMevConfig,
    consecutive_failures: u32,
    last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl JitoMevPod {
    pub fn new(jito_config: JitoMevConfig) -> Self {
        Self {
            config: PodConfig {
                id: "jito-mev".to_string(),
                name: "Jito MEV Integration".to_string(),
                version: Version::new(0, 1, 0),
                description: "MEV extraction with Jito TipRouter and staking rewards".to_string(),
                dependencies: vec![
                    banshee_core::pod_dependency!("web3", "1.0.0"), // For wallet
                    banshee_core::pod_dependency!("emotion", "0.1.0"), // For emotional trading
                ],
                provides: vec![
                    banshee_core::pod_capability!(
                        "mev_extraction",
                        "0.1.0",
                        "Extract MEV through Jito block engine"
                    ),
                    banshee_core::pod_capability!(
                        "tip_router",
                        "0.1.0",
                        "Decentralized tip distribution to stakers"
                    ),
                    banshee_core::pod_capability!(
                        "staking_optimization",
                        "0.1.0",
                        "Optimize staking for MEV rewards"
                    ),
                ],
                settings: HashMap::new(),
            },
            jito_config,
            consecutive_failures: 0,
            last_failure_time: None,
        }
    }

    /// Check if we're in cooldown period after losses
    fn is_in_cooldown(&self) -> bool {
        if !self
            .jito_config
            .emotional_trading
            .revenge_trading_protection
        {
            return false;
        }

        if let Some(last_failure) = self.last_failure_time {
            let cooldown_duration = chrono::Duration::seconds(
                self.jito_config.emotional_trading.loss_cooldown_seconds as i64,
            );
            let now = chrono::Utc::now();

            now - last_failure < cooldown_duration
        } else {
            false
        }
    }

    /// Update failure tracking
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure_time = Some(chrono::Utc::now());

        if self.consecutive_failures >= self.jito_config.risk_management.max_consecutive_failures {
            error!(
                "Max consecutive failures reached ({}). MEV extraction paused.",
                self.consecutive_failures
            );
        }
    }

    /// Reset failure tracking on success
    fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_failure_time = None;
    }
}

impl Default for JitoMevPod {
    fn default() -> Self {
        Self::new(JitoMevConfig::default())
    }
}

#[async_trait]
impl Pod for JitoMevPod {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<banshee_core::plugin::PodDependency> {
        self.config.dependencies.clone()
    }

    fn capabilities(&self) -> Vec<banshee_core::plugin::PodCapability> {
        self.config.provides.clone()
    }

    async fn initialize(&mut self) -> PodResult<()> {
        info!(
            "Initializing Jito MEV pod on {} network",
            match self.jito_config.network {
                crate::config::NetworkType::MainnetBeta => "mainnet",
                crate::config::NetworkType::Devnet => "devnet",
            }
        );

        // Log configuration
        info!(
            "TipRouter config - Stakers: {}%, Validators: {}%, Min tip: {} SOL",
            self.jito_config.tip_router.staker_percentage,
            self.jito_config.tip_router.validator_percentage,
            self.jito_config.tip_router.min_tip_sol
        );

        info!(
            "Risk limits - Max capital: {} SOL, Daily loss: {} SOL, Min confidence: {}",
            self.jito_config
                .risk_management
                .max_capital_per_opportunity_sol,
            self.jito_config.risk_management.max_daily_loss_sol,
            self.jito_config.risk_management.min_confidence_score
        );

        if self.jito_config.protection.use_private_mempool {
            info!("Private mempool submission enabled for MEV protection");
        }

        if self.jito_config.auto_scan {
            info!(
                "Auto-scanning enabled with {} ms interval",
                self.jito_config.scan_interval_ms
            );
        }

        // Validate auth keypair if provided
        if self.jito_config.auth_keypair.is_none() {
            warn!("No auth keypair provided - some Jito features may be limited");
        }

        info!("Jito MEV pod initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        info!("Shutting down Jito MEV pod");

        // Log final statistics
        if self.consecutive_failures > 0 {
            warn!(
                "Shutting down with {} consecutive failures",
                self.consecutive_failures
            );
        }

        info!("Jito MEV pod shutdown complete");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![
            Box::new(SubmitBundleAction::new(self.jito_config.clone())),
            Box::new(ScanMevAction::new(self.jito_config.clone())),
            Box::new(OptimizeStakingAction::new(self.jito_config.clone())),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(MevAnalyticsProvider::new(self.jito_config.clone())),
            Box::new(ValidatorMetricsProvider::new(self.jito_config.clone())),
            Box::new(StakingRewardsProvider::new(self.jito_config.clone())),
            Box::new(MevOpportunityProvider::new(self.jito_config.clone())),
        ]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        // No evaluators for now
        vec![]
    }

    async fn health_check(&self) -> PodResult<bool> {
        // Check if we're in cooldown
        if self.is_in_cooldown() {
            info!("MEV pod in cooldown period");
            return Ok(true); // Still healthy, just cooling down
        }

        // Check if we've hit max failures
        if self.consecutive_failures >= self.jito_config.risk_management.max_consecutive_failures {
            error!("MEV pod suspended due to excessive failures");
            return Ok(false);
        }

        // In real implementation, would check:
        // 1. Block engine connection
        // 2. Auth keypair validity
        // 3. Recent bundle success rate

        Ok(true)
    }

    async fn on_dependency_available(
        &mut self,
        dependency_id: &str,
        _dependency: std::sync::Arc<dyn Pod>,
    ) -> PodResult<()> {
        match dependency_id {
            "web3" => {
                info!("Web3 pod available - wallet functionality enabled for MEV");
            }
            "emotion" => {
                info!("Emotion pod available - emotional MEV strategies enabled");
            }
            _ => {}
        }
        Ok(())
    }

    async fn on_dependency_unavailable(&mut self, dependency_id: &str) -> PodResult<()> {
        match dependency_id {
            "web3" => {
                error!("Web3 pod unavailable - MEV extraction disabled");
            }
            "emotion" => {
                warn!("Emotion pod unavailable - using default MEV parameters");
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
        let mut pod = JitoMevPod::default();

        assert!(pod.initialize().await.is_ok());
        assert_eq!(pod.name(), "Jito MEV Integration");
        assert!(!pod.actions().is_empty());
        assert!(!pod.providers().is_empty());
        assert!(pod.health_check().await.unwrap());
        assert!(pod.shutdown().await.is_ok());
    }

    #[test]
    fn test_failure_tracking() {
        let mut pod = JitoMevPod::default();

        assert_eq!(pod.consecutive_failures, 0);

        pod.record_failure();
        assert_eq!(pod.consecutive_failures, 1);
        assert!(pod.last_failure_time.is_some());

        pod.record_success();
        assert_eq!(pod.consecutive_failures, 0);
        assert!(pod.last_failure_time.is_none());
    }

    #[test]
    fn test_cooldown_period() {
        let mut pod = JitoMevPod::default();
        pod.jito_config.emotional_trading.revenge_trading_protection = true;
        pod.jito_config.emotional_trading.loss_cooldown_seconds = 300;

        assert!(!pod.is_in_cooldown());

        pod.record_failure();
        assert!(pod.is_in_cooldown());
    }
}
