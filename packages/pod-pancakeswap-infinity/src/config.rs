//! Configuration types for PancakeSwap Infinity integration

use crate::types::{AmmType, HookParams};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for PancakeSwap Infinity pod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PancakeSwapConfig {
    /// Chain-specific configurations
    pub chains: HashMap<u64, ChainConfig>,
    /// Default chain to use
    pub default_chain: u64,
    /// API configuration
    pub api: ApiConfig,
    /// Trading configuration
    pub trading: TradingConfig,
    /// Hook configurations
    pub hooks: HashMap<String, HookConfig>,
    /// Risk management settings
    pub risk_management: RiskConfig,
    /// Emotional trading settings
    pub emotional_trading: EmotionalConfig,
}

/// Configuration for a specific blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub wss_url: Option<String>,
    pub explorer_url: String,
    pub contracts: ContractAddresses,
    pub native_token: Address,
    pub wrapped_native: Address,
    pub gas_price_oracle: Option<String>,
    pub max_gas_price: Option<u64>,
    pub block_time_ms: u64,
}

/// Contract addresses for PancakeSwap Infinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    /// Main PancakeSwap v4 Manager contract
    pub pool_manager: Address,
    /// Router for swaps
    pub swap_router: Address,
    /// Position manager for liquidity
    pub position_manager: Address,
    /// Factory contract
    pub factory: Address,
    /// Quoter for price quotes
    pub quoter: Address,
    /// Multicall contract
    pub multicall: Address,
    /// Hook registry
    pub hook_registry: Option<Address>,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base URL for PancakeSwap API
    pub base_url: String,
    /// API key for rate limiting
    pub api_key: Option<String>,
    /// Rate limit per second
    pub rate_limit: u32,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Retry configuration
    pub retry: RetryConfig,
}

/// Retry configuration for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

/// Trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Default slippage tolerance (0.0 to 1.0)
    pub default_slippage: f64,
    /// Maximum slippage tolerance
    pub max_slippage: f64,
    /// Default transaction deadline in seconds
    pub default_deadline: u64,
    /// Preferred AMM types in order of preference
    pub preferred_amms: Vec<AmmType>,
    /// Pool selection strategy
    pub pool_selection: PoolSelectionConfig,
    /// Gas optimization settings
    pub gas_optimization: GasConfig,
}

/// Pool selection strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSelectionConfig {
    /// Minimum liquidity required (in USD)
    pub min_liquidity_usd: f64,
    /// Maximum price impact allowed
    pub max_price_impact: f64,
    /// Prefer pools with higher volume
    pub prefer_high_volume: bool,
    /// Minimum volume 24h (in USD)
    pub min_volume_24h_usd: f64,
    /// Pool scoring weights
    pub scoring: PoolScoringWeights,
}

/// Weights for pool scoring algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolScoringWeights {
    pub liquidity: f64,
    pub volume: f64,
    pub fees: f64,
    pub price_stability: f64,
    pub hook_complexity: f64,
}

/// Gas optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Use flash accounting when beneficial
    pub use_flash_accounting: bool,
    /// Batch operations when possible
    pub batch_operations: bool,
    /// Maximum gas price multiplier
    pub max_gas_price_multiplier: f64,
    /// Priority fee strategy
    pub priority_fee_strategy: PriorityFeeStrategy,
}

/// Strategy for setting priority fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityFeeStrategy {
    /// Fixed priority fee in gwei
    Fixed(u64),
    /// Percentage of base fee
    Percentage(f64),
    /// Dynamic based on network conditions
    Dynamic,
}

/// Hook-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub name: String,
    pub address: Address,
    pub params: HookParams,
    pub enabled: bool,
    /// Gas limit for hook execution
    pub gas_limit: u64,
    /// Custom configuration for the hook
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum position size per pool (in USD)
    pub max_position_size_usd: f64,
    /// Maximum total exposure (in USD)
    pub max_total_exposure_usd: f64,
    /// Stop loss percentage
    pub stop_loss_percent: f64,
    /// Take profit percentage
    pub take_profit_percent: f64,
    /// Maximum drawdown before position closure
    pub max_drawdown_percent: f64,
    /// Blacklisted tokens
    pub blacklisted_tokens: Vec<Address>,
    /// Whitelisted tokens (if empty, all tokens allowed)
    pub whitelisted_tokens: Vec<Address>,
    /// Maximum slippage for risk management
    pub max_slippage_risk: f64,
}

/// Emotional trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalConfig {
    /// Enable emotional trading features
    pub enabled: bool,
    /// Sentiment analysis integration
    pub sentiment_analysis: SentimentConfig,
    /// Fear and greed adjustments
    pub fear_greed_adjustments: FearGreedConfig,
    /// Confidence-based position sizing
    pub confidence_scaling: ConfidenceConfig,
}

/// Sentiment analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    /// Weight of sentiment in trading decisions (0.0 to 1.0)
    pub weight: f64,
    /// Sources for sentiment data
    pub sources: Vec<String>,
    /// Update frequency in seconds
    pub update_frequency: u64,
    /// Minimum confidence required for sentiment-based trades
    pub min_confidence: f64,
}

/// Fear and greed index adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedConfig {
    /// Reduce position sizes during extreme fear
    pub fear_reduction_factor: f64,
    /// Increase position sizes during extreme greed (with caution)
    pub greed_increase_factor: f64,
    /// Fear threshold (0.0 to 1.0)
    pub fear_threshold: f64,
    /// Greed threshold (0.0 to 1.0)
    pub greed_threshold: f64,
}

/// Confidence-based scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceConfig {
    /// Base position size multiplier
    pub base_multiplier: f64,
    /// Maximum multiplier for high confidence
    pub max_multiplier: f64,
    /// Minimum multiplier for low confidence
    pub min_multiplier: f64,
    /// Confidence curve exponent
    pub curve_exponent: f64,
}

/// Pool-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub pool_id: String,
    pub max_position_size_usd: Option<f64>,
    pub custom_slippage: Option<f64>,
    pub hook_overrides: Option<HookConfig>,
    pub disabled: bool,
    pub notes: Option<String>,
}

impl Default for PancakeSwapConfig {
    fn default() -> Self {
        let mut chains = HashMap::new();

        // BSC Mainnet configuration
        chains.insert(
            56,
            ChainConfig {
                chain_id: 56,
                name: "BSC Mainnet".to_string(),
                rpc_url: "https://bsc-dataseed.binance.org/".to_string(),
                wss_url: Some("wss://bsc-ws-node.nariox.org:443".to_string()),
                explorer_url: "https://bscscan.com".to_string(),
                contracts: ContractAddresses {
                    pool_manager: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(), // Placeholder
                    swap_router: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    position_manager: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    factory: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    quoter: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    multicall: "0x0000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    hook_registry: None,
                },
                native_token: "0x0000000000000000000000000000000000000000"
                    .parse()
                    .unwrap(),
                wrapped_native: "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"
                    .parse()
                    .unwrap(), // WBNB
                gas_price_oracle: Some("https://bscgas.info/gas".to_string()),
                max_gas_price: Some(20_000_000_000), // 20 gwei
                block_time_ms: 3000,
            },
        );

        Self {
            chains,
            default_chain: 56,
            api: ApiConfig {
                base_url: "https://api.pancakeswap.finance/v4".to_string(),
                api_key: None,
                rate_limit: 100,
                timeout_seconds: 30,
                retry: RetryConfig {
                    max_attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 10000,
                    exponential_backoff: true,
                },
            },
            trading: TradingConfig {
                default_slippage: 0.005, // 0.5%
                max_slippage: 0.05,      // 5%
                default_deadline: 300,   // 5 minutes
                preferred_amms: vec![AmmType::CLAMM, AmmType::V2, AmmType::Stable],
                pool_selection: PoolSelectionConfig {
                    min_liquidity_usd: 10000.0,
                    max_price_impact: 0.03,
                    prefer_high_volume: true,
                    min_volume_24h_usd: 1000.0,
                    scoring: PoolScoringWeights {
                        liquidity: 0.4,
                        volume: 0.3,
                        fees: 0.2,
                        price_stability: 0.08,
                        hook_complexity: 0.02,
                    },
                },
                gas_optimization: GasConfig {
                    use_flash_accounting: true,
                    batch_operations: true,
                    max_gas_price_multiplier: 2.0,
                    priority_fee_strategy: PriorityFeeStrategy::Dynamic,
                },
            },
            hooks: HashMap::new(),
            risk_management: RiskConfig {
                max_position_size_usd: 100000.0,
                max_total_exposure_usd: 1000000.0,
                stop_loss_percent: 0.1,     // 10%
                take_profit_percent: 0.2,   // 20%
                max_drawdown_percent: 0.15, // 15%
                blacklisted_tokens: Vec::new(),
                whitelisted_tokens: Vec::new(),
                max_slippage_risk: 0.02, // 2%
            },
            emotional_trading: EmotionalConfig {
                enabled: true,
                sentiment_analysis: SentimentConfig {
                    weight: 0.3,
                    sources: vec![
                        "social_media".to_string(),
                        "news".to_string(),
                        "on_chain".to_string(),
                    ],
                    update_frequency: 300, // 5 minutes
                    min_confidence: 0.6,
                },
                fear_greed_adjustments: FearGreedConfig {
                    fear_reduction_factor: 0.5,
                    greed_increase_factor: 1.2,
                    fear_threshold: 0.25,
                    greed_threshold: 0.75,
                },
                confidence_scaling: ConfidenceConfig {
                    base_multiplier: 1.0,
                    max_multiplier: 2.0,
                    min_multiplier: 0.1,
                    curve_exponent: 2.0,
                },
            },
        }
    }
}
