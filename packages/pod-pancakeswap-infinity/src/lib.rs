//! # PancakeSwap Infinity Pod
//!
//! Integration pod for PancakeSwap v4 (Infinity) with hooks system and flash accounting.
//!
//! ## Features
//! - Multi-AMM support (CLAMM, LBAMM, Stable, v2)
//! - Hook system integration for custom logic
//! - Flash accounting for gas optimization
//! - Singleton contract architecture
//! - Cross-chain liquidity aggregation
//! - MEV-resistant design patterns
//!
//! ## Architecture
//!
//! PancakeSwap Infinity introduces several key innovations over v3:
//!
//! ### Hooks System
//! Hooks allow custom logic to be executed at specific points in the swap lifecycle:
//! - `beforeInitialize` - Before pool initialization
//! - `afterInitialize` - After pool initialization  
//! - `beforeAddLiquidity` - Before liquidity addition
//! - `afterAddLiquidity` - After liquidity addition
//! - `beforeRemoveLiquidity` - Before liquidity removal
//! - `afterRemoveLiquidity` - After liquidity removal
//! - `beforeSwap` - Before swap execution
//! - `afterSwap` - After swap execution
//! - `beforeDonate` - Before donation
//! - `afterDonate` - After donation
//!
//! ### Flash Accounting
//! Instead of transferring tokens at each step, the system uses a net settlement approach:
//! - All operations are recorded as deltas
//! - Net settlement occurs at the end of the transaction
//! - Significant gas savings for complex operations
//! - Enables advanced composability
//!
//! ### Multi-AMM Architecture
//! - CLAMM: Concentrated Liquidity AMM (Uniswap v3 style)
//! - LBAMM: Liquidity Book AMM (TraderJoe style)
//! - Stable: StableSwap AMM (Curve style)
//! - v2: Classic x*y=k AMM
//!
//! ## Integration with Banshee
//!
//! This pod provides emotional agents with sophisticated DeFi capabilities:
//! - Automated liquidity provision based on market sentiment
//! - Hook-based strategies that respond to emotional state
//! - Flash accounting for complex multi-step operations
//! - Cross-AMM arbitrage opportunities
//! - Risk management through emotional intelligence

#![allow(clippy::new_without_default)]
#![allow(clippy::uninlined_format_args)]

pub mod actions;
pub mod config;
pub mod hooks;
pub mod pod;
pub mod providers;
pub mod types;

// Re-export main types
pub use actions::{
    AddLiquidityAction, CreatePoolAction, FlashAccountingAction, HookAction, PancakeSwapAction,
    RemoveLiquidityAction, SwapAction,
};
pub use config::{ChainConfig, HookConfig, PancakeSwapConfig, PoolConfig};
pub use hooks::{
    ArbitrageHook, BaseHook, EmotionalHook, HookManager, HookParams, HookResult, HookType,
    RiskManagementHook,
};
pub use pod::PancakeSwapInfinityPod;
pub use providers::{
    AnalyticsProvider, LiquidityProvider, PancakeSwapProvider, PoolProvider, PriceProvider,
};
pub use types::{
    AmmType, FlashAccountingDelta, GasEstimate, LiquidityParams, PoolId, PoolMetrics, PoolState,
    PoolType, Position, PriceData, SwapParams, Token, TokenPair, Volume24h,
};

/// Result type for PancakeSwap operations
pub type Result<T> = std::result::Result<T, PancakeSwapError>;

/// Error types for PancakeSwap operations
#[derive(Debug, thiserror::Error)]
pub enum PancakeSwapError {
    #[error("Pool not found: {pool_id}")]
    PoolNotFound { pool_id: String },

    #[error("Insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: String, available: String },

    #[error("Slippage tolerance exceeded: expected {expected}, actual {actual}")]
    SlippageExceeded { expected: f64, actual: f64 },

    #[error("Hook execution failed: {hook_type}: {reason}")]
    HookExecutionFailed { hook_type: String, reason: String },

    #[error("Flash accounting error: {reason}")]
    FlashAccountingError { reason: String },

    #[error("Invalid chain configuration: {chain}")]
    InvalidChainConfig { chain: String },

    #[error("Rate limit exceeded: {endpoint}")]
    RateLimitExceeded { endpoint: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Internal error: {0}")]
    InternalError(String),
}
