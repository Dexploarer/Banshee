//! Configuration for Pump.fun pod

use crate::types::{EmotionalTradingParams, RiskParams};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunConfig {
    /// Solana RPC endpoint
    pub rpc_endpoint: String,

    /// Network (mainnet-beta or devnet)
    pub network: NetworkType,

    /// Trading wallet private key (base58 encoded)
    pub wallet_private_key: Option<String>,

    /// Auto-generate wallet if none provided
    pub auto_generate_wallet: bool,

    /// Risk management parameters
    pub risk_params: RiskParams,

    /// Emotional trading parameters
    pub emotional_trading: EmotionalTradingParams,

    /// Enable automatic token discovery
    pub auto_discovery: bool,

    /// Minimum liquidity for auto-discovery (in SOL)
    pub min_liquidity_sol: Decimal,

    /// Maximum age for auto-discovered tokens (seconds)
    pub max_token_age_seconds: u64,

    /// Referrer address for fee sharing
    pub referrer_address: Option<String>,

    /// Enable graduation monitoring
    pub monitor_graduation: bool,

    /// Priority fee in microlamports
    pub priority_fee: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkType {
    #[serde(rename = "mainnet-beta")]
    MainnetBeta,
    #[serde(rename = "devnet")]
    Devnet,
}

impl Default for PumpFunConfig {
    fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            network: NetworkType::MainnetBeta,
            wallet_private_key: None,
            auto_generate_wallet: true,
            risk_params: RiskParams {
                max_position_size_sol: Decimal::from(5),
                max_slippage_percentage: 5.0,
                stop_loss_percentage: Some(20.0),
                take_profit_percentage: Some(50.0),
                max_gas_sol: Decimal::from(1),
            },
            emotional_trading: EmotionalTradingParams {
                excitement_multiplier: 1.5,
                fear_threshold: 0.7,
                fomo_protection: true,
                greed_control_percentage: 40.0,
            },
            auto_discovery: false,
            min_liquidity_sol: Decimal::from(10),
            max_token_age_seconds: 3600, // 1 hour
            referrer_address: None,
            monitor_graduation: true,
            priority_fee: 10_000, // 0.00001 SOL
        }
    }
}

impl PumpFunConfig {
    /// Create config for mainnet trading
    pub fn mainnet() -> Self {
        Self::default()
    }

    /// Create config for devnet testing
    pub fn devnet() -> Self {
        Self {
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network: NetworkType::Devnet,
            ..Self::default()
        }
    }

    /// Create config with conservative risk parameters
    pub fn conservative() -> Self {
        Self {
            risk_params: RiskParams {
                max_position_size_sol: Decimal::from(1),
                max_slippage_percentage: 2.0,
                stop_loss_percentage: Some(10.0),
                take_profit_percentage: Some(25.0),
                max_gas_sol: Decimal::new(5, 1), // 0.5 SOL
            },
            emotional_trading: EmotionalTradingParams {
                excitement_multiplier: 1.2,
                fear_threshold: 0.8,
                fomo_protection: true,
                greed_control_percentage: 20.0,
            },
            ..Self::default()
        }
    }

    /// Create config for aggressive trading
    pub fn aggressive() -> Self {
        Self {
            risk_params: RiskParams {
                max_position_size_sol: Decimal::from(10),
                max_slippage_percentage: 10.0,
                stop_loss_percentage: Some(30.0),
                take_profit_percentage: Some(100.0),
                max_gas_sol: Decimal::from(2),
            },
            emotional_trading: EmotionalTradingParams {
                excitement_multiplier: 2.0,
                fear_threshold: 0.5,
                fomo_protection: false,
                greed_control_percentage: 60.0,
            },
            auto_discovery: true,
            ..Self::default()
        }
    }
}
