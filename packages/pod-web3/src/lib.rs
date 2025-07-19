//! # Banshee Web3 Plugin - July 2025 Solana SDK Implementation
//!
//! Real Solana blockchain integration using the latest July 2025 SDK versions.
//! Features:
//! - Wallet creation and management with BIP39 mnemonic support
//! - SOL transfers with transaction signing and confirmation
//! - SPL token deployment, minting, and transfers
//! - NFT creation and management
//! - Real-time balance checking and transaction history
//! - Devnet airdrop functionality
//! - Full error handling and validation

use async_trait::async_trait;
use banshee_core::{
    action::{ActionConfig, ActionExample, EmotionalImpact},
    plugin::{self, Pod, PodConfig},
    provider::ProviderConfig,
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// July 2025 Solana SDK imports
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};

// Wallet and crypto imports
use bip39::{Language, Mnemonic};
use bs58;

// Internal modules
pub mod actions;
pub mod providers;
pub mod tokens;
pub mod wallet;

// Re-exports
pub use actions::*;
pub use providers::*;
pub use tokens::*;
pub use wallet::*;

/// Configuration for the Web3 plugin using July 2025 Solana SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PluginConfig {
    /// Solana RPC endpoint
    pub rpc_endpoint: String,
    /// Network type (mainnet-beta, testnet, devnet)
    pub network: String,
    /// Commitment level for transactions
    pub commitment: String,
    /// Maximum retry attempts for failed transactions
    pub max_retries: u32,
    /// Optional wallet mnemonic phrase for persistent wallet
    pub wallet_mnemonic: Option<String>,
    /// Whether to auto-generate wallet if none provided
    pub auto_generate_wallet: bool,
    /// Enable devnet airdrops for low balance wallets
    pub enable_airdrops: bool,
}

impl Default for Web3PluginConfig {
    fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network: "devnet".to_string(),
            commitment: "confirmed".to_string(),
            max_retries: 3,
            wallet_mnemonic: None,
            auto_generate_wallet: true,
            enable_airdrops: true,
        }
    }
}

/// Main Web3 plugin with July 2025 Solana SDK integration
pub struct Web3Plugin {
    config: PodConfig,
    web3_config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl Web3Plugin {
    pub fn new(web3_config: Web3PluginConfig) -> Self {
        Self {
            config: PodConfig {
                id: "web3".to_string(),
                name: "Web3 Plugin".to_string(),
                version: plugin::Version::new(1, 0, 0),
                description: "Real Solana blockchain integration with July 2025 SDK".to_string(),
                dependencies: vec![],
                provides: vec![],
                settings: HashMap::new(),
            },
            web3_config,
            wallet_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Get or create wallet manager instance
    async fn get_wallet_manager(&self) -> anyhow::Result<WalletManager> {
        let manager_guard = self.wallet_manager.read().await;

        if let Some(manager) = manager_guard.as_ref() {
            return Ok(manager.clone());
        }

        drop(manager_guard);

        // Create new wallet manager
        let manager = WalletManager::new(&self.web3_config)?;

        let mut manager_guard = self.wallet_manager.write().await;
        *manager_guard = Some(manager.clone());

        Ok(manager)
    }
}

impl Default for Web3Plugin {
    fn default() -> Self {
        Self::new(Web3PluginConfig::default())
    }
}

#[async_trait]
impl Pod for Web3Plugin {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn initialize(&mut self) -> plugin::PodResult<()> {
        tracing::info!(
            "Initializing Web3 plugin with July 2025 Solana SDK on network: {} at {}",
            self.web3_config.network,
            self.web3_config.rpc_endpoint
        );

        // Initialize wallet manager
        match self.get_wallet_manager().await {
            Ok(mut manager) => {
                let wallet = manager
                    .get_primary_wallet()
                    .await
                    .map_err(|e| e.to_string())?;
                let address = wallet.public_key().to_string();
                tracing::info!("Web3 plugin initialized with wallet: {}", address);

                // Check balance and request airdrop if needed
                if self.web3_config.enable_airdrops && self.web3_config.network == "devnet" {
                    let balance = wallet.get_balance().await?;
                    if balance < 1.0 {
                        tracing::info!("Low balance detected, requesting devnet airdrop...");
                        match wallet.request_airdrop(2.0).await {
                            Ok(sig) => tracing::info!("Airdrop successful: {}", sig),
                            Err(e) => tracing::warn!("Airdrop failed: {}", e),
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to initialize wallet manager: {}", e);
                return Err(e.into());
            }
        }

        tracing::info!("Web3 plugin initialization complete");
        Ok(())
    }

    async fn shutdown(&mut self) -> plugin::PodResult<()> {
        tracing::info!("Web3 plugin shutting down");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![
            Box::new(TransferSolAction::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
            Box::new(GetBalanceAction::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
            Box::new(CreateWalletAction::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
            Box::new(DeployTokenAction::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
            Box::new(TransferTokenAction::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(WalletInfoProvider::new(
                self.web3_config.clone(),
                self.wallet_manager.clone(),
            )),
            Box::new(NetworkInfoProvider::new(self.web3_config.clone())),
        ]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        vec![]
    }

    async fn health_check(&self) -> plugin::PodResult<bool> {
        match self.get_wallet_manager().await {
            Ok(mut manager) => {
                let wallet = manager
                    .get_primary_wallet()
                    .await
                    .map_err(|e| e.to_string())?;
                wallet.health_check().await.map_err(|e| e.to_string())
            }
            Err(e) => {
                tracing::error!("Health check failed - wallet manager unavailable: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = Web3Plugin::default();

        assert!(plugin.initialize().await.is_ok());
        assert!(plugin.health_check().await.unwrap());
        assert!(plugin.shutdown().await.is_ok());
    }

    #[test]
    fn test_plugin_components() {
        let plugin = Web3Plugin::default();

        assert!(!plugin.actions().is_empty());
        assert!(!plugin.providers().is_empty());
        assert_eq!(plugin.config().id, "web3");
        assert_eq!(plugin.config().name, "Web3 Plugin");
    }

    #[test]
    fn test_config_defaults() {
        let config = Web3PluginConfig::default();

        assert_eq!(config.network, "devnet");
        assert_eq!(config.commitment, "confirmed");
        assert!(config.auto_generate_wallet);
        assert!(config.enable_airdrops);
    }
}
