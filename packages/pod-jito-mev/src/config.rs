//! Configuration for Jito MEV pod

use crate::types::{MevProtection, RiskLevel, TipRouterConfig};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoMevConfig {
    /// Jito block engine endpoint
    pub block_engine_endpoint: String,

    /// Network type
    pub network: NetworkType,

    /// Auth keypair for Jito (base58 encoded private key)
    pub auth_keypair: Option<String>,
    
    /// Key ID from secure key manager (preferred over raw auth_keypair)
    pub auth_keypair_id: Option<String>,

    /// TipRouter configuration
    pub tip_router: TipRouterConfig,

    /// MEV protection settings
    pub protection: MevProtection,

    /// Risk management settings
    pub risk_management: RiskManagement,

    /// Emotional trading parameters
    pub emotional_trading: EmotionalTradingParams,

    /// Enable automatic MEV scanning
    pub auto_scan: bool,

    /// Scan interval in milliseconds
    pub scan_interval_ms: u64,

    /// Minimum profit threshold in SOL
    pub min_profit_sol: Decimal,

    /// Maximum bundle size
    pub max_bundle_size: usize,

    /// Enable bundle simulation before submission
    pub simulate_bundles: bool,

    /// Maximum gas per bundle in SOL
    pub max_gas_per_bundle_sol: Decimal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkType {
    #[serde(rename = "mainnet-beta")]
    MainnetBeta,
    #[serde(rename = "devnet")]
    Devnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagement {
    /// Maximum capital per MEV opportunity
    pub max_capital_per_opportunity_sol: Decimal,

    /// Maximum daily loss limit
    pub max_daily_loss_sol: Decimal,

    /// Risk levels to accept
    pub accepted_risk_levels: Vec<RiskLevel>,

    /// Minimum confidence score (0-1)
    pub min_confidence_score: f64,

    /// Stop trading after consecutive failures
    pub max_consecutive_failures: u32,

    /// Position sizing based on Kelly Criterion
    pub use_kelly_criterion: bool,

    /// Kelly fraction (typically 0.25 for 1/4 Kelly)
    pub kelly_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTradingParams {
    /// Increase aggression when confident
    pub confidence_multiplier: f64,

    /// Reduce position size when fearful
    pub fear_reduction_factor: f64,

    /// Greed threshold to take profits
    pub greed_threshold: f64,

    /// Enable revenge trading protection
    pub revenge_trading_protection: bool,

    /// Cooldown period after losses (seconds)
    pub loss_cooldown_seconds: u64,
}

impl Default for JitoMevConfig {
    fn default() -> Self {
        Self {
            block_engine_endpoint: crate::JITO_BLOCK_ENGINE_MAINNET.to_string(),
            network: NetworkType::MainnetBeta,
            auth_keypair: None,
            auth_keypair_id: None,
            tip_router: TipRouterConfig {
                staker_percentage: 97.0,
                validator_percentage: 3.0,
                min_tip_sol: Decimal::new(1, 3), // 0.001 SOL
                dynamic_tips: true,
                dynamic_tip_percentage: 20.0, // 20% of gross profit
            },
            protection: MevProtection {
                anti_sandwich: true,
                anti_frontrun: true,
                max_slippage_percentage: 1.0,
                use_private_mempool: true,
            },
            risk_management: RiskManagement {
                max_capital_per_opportunity_sol: Decimal::from(10),
                max_daily_loss_sol: Decimal::from(50),
                accepted_risk_levels: vec![RiskLevel::Low, RiskLevel::Medium],
                min_confidence_score: 0.7,
                max_consecutive_failures: 5,
                use_kelly_criterion: true,
                kelly_fraction: 0.25,
            },
            emotional_trading: EmotionalTradingParams {
                confidence_multiplier: 1.5,
                fear_reduction_factor: 0.5,
                greed_threshold: 0.8,
                revenge_trading_protection: true,
                loss_cooldown_seconds: 300,
            },
            auto_scan: false,
            scan_interval_ms: 100,
            min_profit_sol: Decimal::new(1, 2), // 0.01 SOL
            max_bundle_size: 5,
            simulate_bundles: true,
            max_gas_per_bundle_sol: Decimal::new(5, 1), // 0.5 SOL
        }
    }
}

impl JitoMevConfig {
    /// Create config for mainnet MEV extraction
    pub fn mainnet() -> Self {
        Self::default()
    }

    /// Create config for devnet testing
    pub fn devnet() -> Self {
        Self {
            block_engine_endpoint: crate::JITO_BLOCK_ENGINE_DEVNET.to_string(),
            network: NetworkType::Devnet,
            tip_router: TipRouterConfig {
                min_tip_sol: Decimal::new(1, 4), // 0.0001 SOL for devnet
                ..Default::default()
            },
            ..Self::default()
        }
    }

    /// Create conservative MEV config
    pub fn conservative() -> Self {
        Self {
            risk_management: RiskManagement {
                max_capital_per_opportunity_sol: Decimal::from(2),
                max_daily_loss_sol: Decimal::from(10),
                accepted_risk_levels: vec![RiskLevel::Low],
                min_confidence_score: 0.85,
                max_consecutive_failures: 3,
                use_kelly_criterion: true,
                kelly_fraction: 0.1, // Very conservative Kelly
            },
            emotional_trading: EmotionalTradingParams {
                confidence_multiplier: 1.2,
                fear_reduction_factor: 0.3,
                greed_threshold: 0.6,
                revenge_trading_protection: true,
                loss_cooldown_seconds: 600,
            },
            min_profit_sol: Decimal::new(5, 2), // 0.05 SOL minimum
            ..Self::default()
        }
    }

    /// Create aggressive MEV config
    pub fn aggressive() -> Self {
        Self {
            risk_management: RiskManagement {
                max_capital_per_opportunity_sol: Decimal::from(50),
                max_daily_loss_sol: Decimal::from(200),
                accepted_risk_levels: vec![RiskLevel::Low, RiskLevel::Medium, RiskLevel::High],
                min_confidence_score: 0.6,
                max_consecutive_failures: 10,
                use_kelly_criterion: false, // Full position sizing
                kelly_fraction: 1.0,
            },
            emotional_trading: EmotionalTradingParams {
                confidence_multiplier: 2.0,
                fear_reduction_factor: 0.8,
                greed_threshold: 0.9,
                revenge_trading_protection: false,
                loss_cooldown_seconds: 60,
            },
            auto_scan: true,
            scan_interval_ms: 50,               // Very fast scanning
            min_profit_sol: Decimal::new(5, 3), // 0.005 SOL minimum
            max_bundle_size: 10,
            ..Self::default()
        }
    }
}
