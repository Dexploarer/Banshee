//! # Web3 Providers - July 2025 Solana SDK
//!
//! Information providers for Solana blockchain data

use async_trait::async_trait;
use banshee_core::{provider::ProviderConfig, *};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Web3PluginConfig, WalletManager};

/// Provides wallet and account information
pub struct WalletInfoProvider {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl WalletInfoProvider {
    pub fn new(config: Web3PluginConfig, wallet_manager: Arc<RwLock<Option<WalletManager>>>) -> Self {
        Self { config, wallet_manager }
    }

    async fn get_wallet_manager(&self) -> anyhow::Result<WalletManager> {
        let manager_guard = self.wallet_manager.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            return Ok(manager.clone());
        }
        drop(manager_guard);
        
        let manager = WalletManager::new(&self.config)?;
        let mut manager_guard = self.wallet_manager.write().await;
        *manager_guard = Some(manager.clone());
        Ok(manager)
    }
}

#[async_trait]
impl Provider for WalletInfoProvider {
    fn name(&self) -> &str {
        "wallet_info"
    }

    fn description(&self) -> &str {
        "Provides Solana wallet and account information"
    }

    fn config(&self) -> &ProviderConfig {
        static CONFIG: std::sync::OnceLock<ProviderConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ProviderConfig {
            name: "wallet_info".to_string(),
            description: "Provides real-time Solana wallet and account information".to_string(),
            priority: 1,
            enabled: true,
            settings: HashMap::new(),
        })
    }

    async fn provide(&self, _context: &Context) -> Result<Vec<ProviderResult>> {
        let mut manager = match self.get_wallet_manager().await {
            Ok(mut m) => m,
            Err(e) => {
                tracing::error!("Failed to get wallet manager: {}", e);
                return Ok(vec![]);
            }
        };

        let wallet = match manager.get_primary_wallet().await {
            Ok(w) => w.clone(),
            Err(e) => {
                tracing::error!("Failed to get wallet: {}", e);
                return Ok(vec![]);
            }
        };

        let balance = wallet.get_balance().await.unwrap_or(0.0);
        let recent_signatures = wallet.get_recent_signatures(5).await.unwrap_or_default();
        
        let wallet_data = serde_json::json!({
            "address": wallet.address(),
            "network": self.config.network,
            "balance": balance,
            "balance_lamports": (balance * 1_000_000_000.0) as u64,
            "recent_transactions": recent_signatures.len(),
            "commitment": self.config.commitment,
            "rpc_endpoint": self.config.rpc_endpoint,
            "sdk_version": "2.3.1"
        });

        Ok(vec![ProviderResult {
            provider: "wallet_info".to_string(),
            data: wallet_data,
            relevance: 0.9,
            confidence: 0.95,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Provides network and blockchain information
pub struct NetworkInfoProvider {
    config: Web3PluginConfig,
}

impl NetworkInfoProvider {
    pub fn new(config: Web3PluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Provider for NetworkInfoProvider {
    fn name(&self) -> &str {
        "network_info"
    }

    fn description(&self) -> &str {
        "Provides Solana network and blockchain information"
    }

    fn config(&self) -> &ProviderConfig {
        static CONFIG: std::sync::OnceLock<ProviderConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ProviderConfig {
            name: "network_info".to_string(),
            description: "Provides real-time Solana network and blockchain information".to_string(),
            priority: 2,
            enabled: true,
            settings: HashMap::new(),
        })
    }

    async fn provide(&self, _context: &Context) -> Result<Vec<ProviderResult>> {
        // Create a temporary wallet for RPC access
        let manager = WalletManager::new(&self.config)?;
        let mut manager_clone = manager.clone();
        let wallet = match manager_clone.get_primary_wallet().await {
            Ok(w) => w.clone(),
            Err(e) => {
                tracing::error!("Failed to get wallet for network info: {}", e);
                return Ok(vec![]);
            }
        };

        let network_info = match wallet.get_network_info().await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Failed to get network info: {}", e);
                return Ok(vec![]);
            }
        };

        let network_data = serde_json::json!({
            "network": network_info.network,
            "slot": network_info.slot,
            "epoch": network_info.epoch,
            "block_height": network_info.block_height,
            "version": network_info.version,
            "commitment": self.config.commitment,
            "rpc_endpoint": self.config.rpc_endpoint,
            "sdk_version": "2.3.1"
        });

        Ok(vec![ProviderResult {
            provider: "network_info".to_string(),
            data: network_data,
            relevance: 0.7,
            confidence: 0.9,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}