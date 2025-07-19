//! Core types for PancakeSwap Infinity integration

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a pool in PancakeSwap Infinity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoolId(pub String);

impl PoolId {
    pub fn new(token0: &Address, token1: &Address, fee: u32, hook: &Address) -> Self {
        Self(format!("{}:{}:{}:{}", token0, token1, fee, hook))
    }
}

impl std::fmt::Display for PoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of AMMs supported by PancakeSwap Infinity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmmType {
    /// Concentrated Liquidity AMM (Uniswap v3 style)
    CLAMM,
    /// Liquidity Book AMM (TraderJoe style)
    LBAMM,
    /// StableSwap AMM (Curve style)
    Stable,
    /// Classic x*y=k AMM
    V2,
}

/// Pool types with specific configurations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    /// Standard AMM pool
    Standard { amm_type: AmmType, fee_tier: u32 },
    /// Stable coin pool with low fees
    Stable { amplification: u32, fee_tier: u32 },
    /// Custom pool with hooks
    Custom {
        amm_type: AmmType,
        hook_address: Address,
        hook_params: HookParams,
        fee_tier: u32,
    },
}

/// Token representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: u64,
}

/// Token pair for trading
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenPair {
    pub token0: Token,
    pub token1: Token,
}

/// Parameters for swap operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub pool_id: PoolId,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub recipient: Address,
    pub deadline: u64,
    pub sqrt_price_limit: Option<U256>,
    pub hook_data: Option<Vec<u8>>,
}

/// Parameters for liquidity operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityParams {
    pub pool_id: PoolId,
    pub token0: Address,
    pub token1: Address,
    pub amount0_desired: U256,
    pub amount1_desired: U256,
    pub amount0_min: U256,
    pub amount1_min: U256,
    pub tick_lower: Option<i32>, // For concentrated liquidity
    pub tick_upper: Option<i32>, // For concentrated liquidity
    pub recipient: Address,
    pub deadline: u64,
    pub hook_data: Option<Vec<u8>>,
}

/// Hook parameters for custom logic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HookParams {
    pub before_initialize: bool,
    pub after_initialize: bool,
    pub before_add_liquidity: bool,
    pub after_add_liquidity: bool,
    pub before_remove_liquidity: bool,
    pub after_remove_liquidity: bool,
    pub before_swap: bool,
    pub after_swap: bool,
    pub before_donate: bool,
    pub after_donate: bool,
    /// Custom data passed to hooks
    pub data: HashMap<String, serde_json::Value>,
}

/// Flash accounting delta tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashAccountingDelta {
    pub token: Address,
    pub delta: i64, // Positive = owed to pool, Negative = owed to user
    pub pool_id: PoolId,
    pub operation_id: String,
}

/// Current state of a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_id: PoolId,
    pub pool_type: PoolType,
    pub token0: Token,
    pub token1: Token,
    pub reserve0: U256,
    pub reserve1: U256,
    pub sqrt_price: U256,
    pub tick: Option<i32>,
    pub liquidity: U256,
    pub fee_tier: u32,
    pub total_volume_24h: U256,
    pub total_fees_24h: U256,
    pub tvl_usd: f64,
    pub apr: f64,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: u64,
    pub pool_id: PoolId,
    pub owner: Address,
    pub tick_lower: Option<i32>,
    pub tick_upper: Option<i32>,
    pub liquidity: U256,
    pub amount0: U256,
    pub amount1: U256,
    pub fees_earned0: U256,
    pub fees_earned1: U256,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Price data for a token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub pool_id: PoolId,
    pub token_pair: TokenPair,
    pub price: f64,
    pub price_24h_ago: f64,
    pub price_change_24h: f64,
    pub price_change_24h_percent: f64,
    pub volume_24h: Volume24h,
    pub timestamp: DateTime<Utc>,
}

/// 24-hour volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume24h {
    pub volume_usd: f64,
    pub volume_token0: U256,
    pub volume_token1: U256,
    pub txn_count: u64,
}

/// Pool metrics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub pool_id: PoolId,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub fees_24h_usd: f64,
    pub apr: f64,
    pub utilization: f64,
    pub impermanent_loss: f64,
    pub price_impact: f64,
    pub last_updated: DateTime<Utc>,
}

/// Gas estimation for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub operation: String,
    pub gas_limit: u64,
    pub gas_price: U256,
    pub estimated_cost_usd: f64,
    pub priority_fee: Option<U256>,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub error: Option<String>,
}

/// Emotional context for trading decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalContext {
    pub sentiment: f64,      // -1.0 to 1.0
    pub confidence: f64,     // 0.0 to 1.0
    pub risk_tolerance: f64, // 0.0 to 1.0
    pub market_fear: f64,    // 0.0 to 1.0
    pub greed_index: f64,    // 0.0 to 1.0
    pub timestamp: DateTime<Utc>,
}

/// Trading strategy based on emotional state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalStrategy {
    pub strategy_type: String,
    pub risk_multiplier: f64,
    pub slippage_tolerance: f64,
    pub max_position_size: f64,
    pub rebalance_threshold: f64,
    pub active: bool,
}

/// Multi-chain pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainPool {
    pub pools: HashMap<u64, PoolState>, // chain_id -> pool_state
    pub total_tvl_usd: f64,
    pub arbitrage_opportunities: Vec<ArbitrageOpportunity>,
    pub last_sync: DateTime<Utc>,
}

/// Arbitrage opportunity across chains or pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub pool_a: PoolId,
    pub pool_b: PoolId,
    pub token_pair: TokenPair,
    pub price_difference: f64,
    pub potential_profit_usd: f64,
    pub gas_cost_estimate: f64,
    pub net_profit_usd: f64,
    pub confidence_score: f64,
    pub expires_at: DateTime<Utc>,
}

/// Liquidity provider (LP) token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpToken {
    pub address: Address,
    pub pool_id: PoolId,
    pub total_supply: U256,
    pub reserve0_per_token: U256,
    pub reserve1_per_token: U256,
    pub apr: f64,
    pub farming_rewards: Option<FarmingRewards>,
}

/// Farming/staking rewards information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmingRewards {
    pub reward_token: Token,
    pub apr: f64,
    pub daily_rewards: U256,
    pub total_staked: U256,
    pub ends_at: Option<DateTime<Utc>>,
}

impl Default for HookParams {
    fn default() -> Self {
        Self {
            before_initialize: false,
            after_initialize: false,
            before_add_liquidity: false,
            after_add_liquidity: false,
            before_remove_liquidity: false,
            after_remove_liquidity: false,
            before_swap: false,
            after_swap: false,
            before_donate: false,
            after_donate: false,
            data: HashMap::new(),
        }
    }
}
