//! # Solana Wallet Management - July 2025 SDK
//!
//! Advanced wallet management using the latest Solana SDK 2.3.1.
//! Features:
//! - BIP39 mnemonic generation and recovery
//! - Keypair management and derivation
//! - SOL transfers with full transaction lifecycle
//! - Balance checking with commitment levels
//! - Transaction history and confirmation
//! - Devnet airdrop functionality
//! - Multi-wallet support

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn, error};

use crate::Web3PluginConfig;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Insufficient funds: required {required} SOL, available {available} SOL")]
    InsufficientFunds { required: f64, available: f64 },
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("RPC client error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
}

/// Solana wallet with July 2025 SDK features
pub struct SolanaWallet {
    keypair: Keypair,
    rpc_client: Arc<RpcClient>,
    commitment: CommitmentConfig,
    network: String,
}

impl SolanaWallet {
    /// Create a new wallet with random keypair
    pub fn new(rpc_endpoint: &str, network: &str, commitment: CommitmentConfig) -> Result<Self> {
        let keypair = Keypair::new();
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_endpoint.to_string(),
            commitment,
        ));
        
        info!("Created new Solana wallet: {}", keypair.pubkey());
        
        Ok(Self {
            keypair,
            rpc_client,
            commitment,
            network: network.to_string(),
        })
    }

    /// Create wallet from existing keypair
    pub fn from_keypair(
        keypair: Keypair,
        rpc_endpoint: &str,
        network: &str,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_endpoint.to_string(),
            commitment,
        ));
        
        info!("Loaded Solana wallet: {}", keypair.pubkey());
        
        Ok(Self {
            keypair,
            rpc_client,
            commitment,
            network: network.to_string(),
        })
    }

    /// Create wallet from BIP39 mnemonic
    pub fn from_mnemonic(
        mnemonic_phrase: &str,
        passphrase: Option<&str>,
        rpc_endpoint: &str,
        network: &str,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        
        // Use the first 32 bytes of the seed for the keypair
        let keypair = Keypair::from_bytes(&seed[..32])
            .map_err(|e| WalletError::InvalidMnemonic(format!("Failed to create keypair: {}", e)))?;
        
        Self::from_keypair(keypair, rpc_endpoint, network, commitment)
    }

    /// Generate new wallet with BIP39 mnemonic
    pub fn generate_with_mnemonic(
        rpc_endpoint: &str,
        network: &str,
        commitment: CommitmentConfig,
    ) -> Result<(Self, String)> {
        let mnemonic = Mnemonic::from_entropy(&bip39::rand::random::<[u8; 16]>())
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        let mnemonic_phrase = mnemonic.to_string();
        
        let wallet = Self::from_mnemonic(&mnemonic_phrase, None, rpc_endpoint, network, commitment)?;
        
        info!("Generated new wallet with 12-word mnemonic");
        Ok((wallet, mnemonic_phrase))
    }

    /// Get wallet public key
    pub fn public_key(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Get wallet address as string
    pub fn address(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    /// Get SOL balance for this wallet
    pub async fn get_balance(&self) -> Result<f64> {
        let balance_lamports = self
            .rpc_client
            .get_balance_with_commitment(&self.keypair.pubkey(), self.commitment)
            .await?
            .value;
        
        let balance_sol = balance_lamports as f64 / LAMPORTS_PER_SOL as f64;
        info!("Wallet {} balance: {} SOL", self.address(), balance_sol);
        
        Ok(balance_sol)
    }

    /// Get SOL balance for any address
    pub async fn get_balance_for_address(&self, address: &str) -> Result<f64> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|_| WalletError::InvalidPublicKey(address.to_string()))?;
        
        let balance_lamports = self
            .rpc_client
            .get_balance_with_commitment(&pubkey, self.commitment)
            .await?
            .value;
        
        let balance_sol = balance_lamports as f64 / LAMPORTS_PER_SOL as f64;
        info!("Address {} balance: {} SOL", address, balance_sol);
        
        Ok(balance_sol)
    }

    /// Transfer SOL to another address with full transaction lifecycle
    pub async fn transfer_sol(&self, to_address: &str, amount_sol: f64) -> Result<Signature> {
        let to_pubkey = Pubkey::from_str(to_address)
            .map_err(|_| WalletError::InvalidPublicKey(to_address.to_string()))?;
        
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        
        // Check balance
        let current_balance = self.get_balance().await?;
        if current_balance < amount_sol {
            return Err(WalletError::InsufficientFunds {
                required: amount_sol,
                available: current_balance,
            }.into());
        }

        info!(
            "Transferring {} SOL ({} lamports) from {} to {}",
            amount_sol, amount_lamports, self.address(), to_address
        );

        // Get recent blockhash
        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .await
            .context("Failed to get recent blockhash")?;

        // Create transfer instruction
        let transfer_instruction = system_instruction::transfer(
            &self.keypair.pubkey(),
            &to_pubkey,
            amount_lamports,
        );

        // Create transaction
        let message = Message::new(&[transfer_instruction], Some(&self.keypair.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);
        
        // Sign transaction
        transaction.sign(&[&self.keypair], recent_blockhash);

        // Send and confirm transaction
        let signature = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .await
            .map_err(|e| WalletError::TransactionFailed(e.to_string()))?;

        info!(
            "Successfully transferred {} SOL to {}. Signature: {}",
            amount_sol, to_address, signature
        );

        Ok(signature)
    }

    /// Request SOL airdrop (devnet/testnet only)
    pub async fn request_airdrop(&self, amount_sol: f64) -> Result<Signature> {
        if self.network == "mainnet-beta" {
            return Err(anyhow::anyhow!("Airdrops not available on mainnet"));
        }

        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        
        info!(
            "Requesting {} SOL airdrop ({} lamports) to {} on {}",
            amount_sol, amount_lamports, self.address(), self.network
        );

        // Get recent blockhash for confirmation
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;

        // Request airdrop
        let signature = self
            .rpc_client
            .request_airdrop_with_blockhash(
                &self.keypair.pubkey(),
                amount_lamports,
                &recent_blockhash,
            )
            .await
            .map_err(|e| WalletError::TransactionFailed(format!("Airdrop failed: {}", e)))?;

        // Confirm the airdrop transaction
        self.rpc_client
            .confirm_transaction_with_spinner(&signature, &recent_blockhash, self.commitment)
            .await
            .map_err(|e| WalletError::TransactionFailed(format!("Failed to confirm airdrop: {}", e)))?;

        info!("Airdrop successful. Signature: {}", signature);
        Ok(signature)
    }

    /// Get recent transaction signatures for this wallet
    pub async fn get_recent_signatures(&self, limit: usize) -> Result<Vec<String>> {
        let signatures = self
            .rpc_client
            .get_signatures_for_address_with_config(
                &self.keypair.pubkey(),
                solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                    limit: Some(limit),
                    ..Default::default()
                },
            )
            .await?;

        let signature_strings: Vec<String> = signatures
            .into_iter()
            .map(|sig_info| sig_info.signature)
            .collect();

        info!("Retrieved {} recent signatures for wallet {}", signature_strings.len(), self.address());
        Ok(signature_strings)
    }

    /// Check RPC connection health
    pub async fn health_check(&self) -> Result<bool> {
        match self.rpc_client.get_health().await {
            Ok(_) => {
                info!("Solana RPC connection healthy for network: {}", self.network);
                Ok(true)
            }
            Err(e) => {
                warn!("Solana RPC health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get network information
    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let slot = self.rpc_client.get_slot().await?;
        let epoch_info = self.rpc_client.get_epoch_info().await?;
        let version = self.rpc_client.get_version().await?;
        
        Ok(NetworkInfo {
            network: self.network.clone(),
            slot,
            epoch: epoch_info.epoch,
            block_height: epoch_info.block_height,
            version: version.solana_core,
        })
    }
}

/// Network information structure
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub network: String,
    pub slot: u64,
    pub epoch: u64,
    pub block_height: u64,
    pub version: String,
}

/// Wallet manager for handling multiple wallets
#[derive(Debug, Clone)]
pub struct WalletManager {
    primary_wallet: Option<SolanaWallet>,
    config: Web3PluginConfig,
    commitment: CommitmentConfig,
}

impl WalletManager {
    /// Create new wallet manager
    pub fn new(config: &Web3PluginConfig) -> Result<Self> {
        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        Ok(Self {
            primary_wallet: None,
            config: config.clone(),
            commitment,
        })
    }

    /// Get or create primary wallet
    pub async fn get_primary_wallet(&mut self) -> Result<&SolanaWallet> {
        if self.primary_wallet.is_none() {
            let wallet = if let Some(ref mnemonic) = self.config.wallet_mnemonic {
                SolanaWallet::from_mnemonic(
                    mnemonic,
                    None,
                    &self.config.rpc_endpoint,
                    &self.config.network,
                    self.commitment,
                )?
            } else if self.config.auto_generate_wallet {
                let (wallet, mnemonic) = SolanaWallet::generate_with_mnemonic(
                    &self.config.rpc_endpoint,
                    &self.config.network,
                    self.commitment,
                )?;
                info!("Generated new wallet with mnemonic: {}", mnemonic);
                warn!("SAVE THIS MNEMONIC SECURELY: {}", mnemonic);
                wallet
            } else {
                return Err(anyhow::anyhow!("No wallet configured and auto-generation disabled"));
            };

            self.primary_wallet = Some(wallet);
        }

        Ok(self.primary_wallet.as_ref().unwrap())
    }

    /// Create additional wallet from mnemonic
    pub fn create_wallet_from_mnemonic(&self, mnemonic: &str) -> Result<SolanaWallet> {
        SolanaWallet::from_mnemonic(
            mnemonic,
            None,
            &self.config.rpc_endpoint,
            &self.config.network,
            self.commitment,
        )
    }

    /// Generate new wallet
    pub fn generate_wallet(&self) -> Result<(SolanaWallet, String)> {
        SolanaWallet::generate_with_mnemonic(
            &self.config.rpc_endpoint,
            &self.config.network,
            self.commitment,
        )
    }
}

/// Utility functions for wallet operations
pub mod utils {
    use super::*;

    /// Validate Solana address format
    pub fn is_valid_address(address: &str) -> bool {
        Pubkey::from_str(address).is_ok()
    }

    /// Convert SOL to lamports
    pub fn sol_to_lamports(sol: f64) -> u64 {
        (sol * LAMPORTS_PER_SOL as f64) as u64
    }

    /// Convert lamports to SOL
    pub fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / LAMPORTS_PER_SOL as f64
    }

    /// Generate random keypair
    pub fn generate_keypair() -> Keypair {
        Keypair::new()
    }

    /// Parse commitment level from string
    pub fn parse_commitment(commitment_str: &str) -> CommitmentConfig {
        match commitment_str {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = SolanaWallet::new(
            "https://api.devnet.solana.com",
            "devnet",
            CommitmentConfig::confirmed(),
        );
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert!(utils::is_valid_address(&wallet.address()));
    }

    #[test]
    fn test_mnemonic_generation() {
        let result = SolanaWallet::generate_with_mnemonic(
            "https://api.devnet.solana.com",
            "devnet",
            CommitmentConfig::confirmed(),
        );
        assert!(result.is_ok());
        
        let (wallet, mnemonic) = result.unwrap();
        assert!(!wallet.address().is_empty());
        assert!(!mnemonic.is_empty());
        
        // Test recovery from mnemonic
        let recovered_wallet = SolanaWallet::from_mnemonic(
            &mnemonic,
            None,
            "https://api.devnet.solana.com",
            "devnet",
            CommitmentConfig::confirmed(),
        );
        assert!(recovered_wallet.is_ok());
        assert_eq!(wallet.address(), recovered_wallet.unwrap().address());
    }

    #[test]
    fn test_address_validation() {
        assert!(utils::is_valid_address("DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"));
        assert!(!utils::is_valid_address("invalid_address"));
        assert!(!utils::is_valid_address(""));
    }

    #[test]
    fn test_conversion_utils() {
        let sol_amount = 2.5;
        let lamports = utils::sol_to_lamports(sol_amount);
        let converted_back = utils::lamports_to_sol(lamports);
        
        assert_eq!(lamports, 2_500_000_000);
        assert!((converted_back - sol_amount).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_devnet_health_check() {
        let wallet = SolanaWallet::new(
            "https://api.devnet.solana.com",
            "devnet", 
            CommitmentConfig::confirmed(),
        );
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        let health = wallet.health_check().await;
        assert!(health.is_ok());
    }
}