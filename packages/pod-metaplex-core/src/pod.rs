//! Main Metaplex Core pod implementation

use async_trait::async_trait;
use banshee_core::{
    plugin::{Pod, PodConfig, PodResult, Version},
    Action, Evaluator, Provider,
};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{actions::*, config::MetaplexCoreConfig, providers::*};

/// Metaplex Core pod for advanced NFT management
pub struct MetaplexCorePod {
    config: PodConfig,
    metaplex_config: MetaplexCoreConfig,
}

impl MetaplexCorePod {
    pub fn new(metaplex_config: MetaplexCoreConfig) -> Self {
        Self {
            config: PodConfig {
                id: "metaplex-core".to_string(),
                name: "Metaplex Core Integration".to_string(),
                version: Version::new(0, 1, 0),
                description: "Advanced NFT and digital asset management with Core and MPL-404"
                    .to_string(),
                dependencies: vec![
                    banshee_core::pod_dependency!("web3", "1.0.0"), // For wallet
                    banshee_core::pod_dependency!("emotion", "0.1.0", optional), // For emotional NFTs
                ],
                provides: vec![
                    banshee_core::pod_capability!(
                        "core_assets",
                        "0.1.0",
                        "Create and manage Metaplex Core single-account NFTs"
                    ),
                    banshee_core::pod_capability!(
                        "mpl_404",
                        "0.1.0",
                        "Hybrid fungible/NFT assets with MPL-404"
                    ),
                    banshee_core::pod_capability!(
                        "compressed_nfts",
                        "0.1.0",
                        "Compressed NFTs with Merkle tree proofs"
                    ),
                ],
                settings: HashMap::new(),
            },
            metaplex_config,
        }
    }
}

impl Default for MetaplexCorePod {
    fn default() -> Self {
        Self::new(MetaplexCoreConfig::default())
    }
}

#[async_trait]
impl Pod for MetaplexCorePod {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        &self.config.version.to_string()
    }

    fn dependencies(&self) -> Vec<banshee_core::plugin::PodDependency> {
        self.config.dependencies.clone()
    }

    fn capabilities(&self) -> Vec<banshee_core::plugin::PodCapability> {
        self.config.provides.clone()
    }

    async fn initialize(&mut self) -> PodResult<()> {
        info!(
            "Initializing Metaplex Core pod on {} network",
            match self.metaplex_config.network {
                crate::config::NetworkType::MainnetBeta => "mainnet",
                crate::config::NetworkType::Devnet => "devnet",
            }
        );

        // Log configuration
        info!(
            "NFT settings - Royalty: {}%, Compression: {}, Emotional: {}",
            self.metaplex_config.default_royalty_percentage,
            self.metaplex_config.prefer_compression,
            self.metaplex_config.emotional_assets.enabled
        );

        if self.metaplex_config.prefer_compression {
            info!(
                "Compression config - Depth: {}, Buffer: {}, Canopy: {}",
                self.metaplex_config.compression.max_depth,
                self.metaplex_config.compression.max_buffer_size,
                self.metaplex_config.compression.canopy_depth
            );

            let max_capacity = 2u32.pow(self.metaplex_config.compression.max_depth);
            info!(
                "Max NFTs per tree: {}, Cost per NFT: {} SOL",
                max_capacity, self.metaplex_config.compression.cost_per_asset_sol
            );
        }

        if self.metaplex_config.mpl404.auto_swap {
            info!(
                "MPL-404 auto-swap enabled at {} token threshold",
                self.metaplex_config.mpl404.default_nft_threshold
            );
        }

        // Validate wallet
        if self.metaplex_config.creator_keypair.is_none()
            && !self.metaplex_config.auto_generate_wallet
        {
            warn!("No creator wallet configured - NFT creation will require wallet input");
        }

        info!("Metaplex Core pod initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PodResult<()> {
        info!("Shutting down Metaplex Core pod");

        // Clean up any resources

        info!("Metaplex Core pod shutdown complete");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![
            Box::new(CreateCoreAssetAction::new(self.metaplex_config.clone())),
            Box::new(CreateMpl404Action::new(self.metaplex_config.clone())),
            Box::new(SwapMpl404Action::new(self.metaplex_config.clone())),
            Box::new(CreateCompressedCollectionAction::new(
                self.metaplex_config.clone(),
            )),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(AssetAnalyticsProvider::new(self.metaplex_config.clone())),
            Box::new(Mpl404StateProvider::new(self.metaplex_config.clone())),
            Box::new(CompressionProofProvider::new(self.metaplex_config.clone())),
            Box::new(EmotionalAssetProvider::new(self.metaplex_config.clone())),
        ]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        // No evaluators for now
        vec![]
    }

    async fn health_check(&self) -> PodResult<bool> {
        // In real implementation, would check:
        // 1. RPC connection
        // 2. Program availability
        // 3. Wallet balance for fees

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
                info!("Emotion pod available - emotional NFT generation enabled");
                // Enable emotional features
                self.metaplex_config.emotional_assets.enabled = true;
            }
            _ => {}
        }
        Ok(())
    }

    async fn on_dependency_unavailable(&mut self, dependency_id: &str) -> PodResult<()> {
        match dependency_id {
            "web3" => {
                warn!("Web3 pod unavailable - NFT operations disabled");
            }
            "emotion" => {
                info!("Emotion pod unavailable - disabling emotional NFT features");
                self.metaplex_config.emotional_assets.enabled = false;
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
        let mut pod = MetaplexCorePod::default();

        assert!(pod.initialize().await.is_ok());
        assert_eq!(pod.name(), "Metaplex Core Integration");
        assert!(!pod.actions().is_empty());
        assert!(!pod.providers().is_empty());
        assert!(pod.health_check().await.unwrap());
        assert!(pod.shutdown().await.is_ok());
    }

    #[test]
    fn test_pod_configuration() {
        let config = MetaplexCoreConfig::emotional();
        let pod = MetaplexCorePod::new(config.clone());

        assert_eq!(pod.config.id, "metaplex-core");
        assert!(pod.metaplex_config.emotional_assets.enabled);
    }

    #[tokio::test]
    async fn test_dependency_handling() {
        let mut pod = MetaplexCorePod::default();
        pod.metaplex_config.emotional_assets.enabled = false;

        // Simulate emotion pod becoming available
        let mock_pod: Box<dyn Pod> = Box::new(MetaplexCorePod::default());
        pod.on_dependency_available(
            "emotion",
            std::sync::Arc::new(tokio::sync::RwLock::new(mock_pod)),
        )
        .await
        .unwrap();

        assert!(pod.metaplex_config.emotional_assets.enabled);
    }
}
