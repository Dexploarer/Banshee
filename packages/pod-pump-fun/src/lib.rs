//! # Pod-Pump-Fun - Direct On-Chain Bonding Curve Integration
//!
//! This pod provides direct integration with Pump.fun's on-chain bonding curve program.
//! Since there is no official SDK, we interact directly with the on-chain program.
//!
//! ## Features
//! - Direct bonding curve token creation and deployment
//! - Buy/sell operations on the bonding curve
//! - Automatic graduation to Raydium when bonding curve completes
//! - Real-time price and volume tracking
//! - Emotional trading strategies based on agent sentiment
//! - Risk management with stop-loss and take-profit

pub mod actions;
pub mod bonding_curve;
pub mod config;
pub mod instructions;
pub mod pod;
pub mod providers;
pub mod types;

pub use actions::*;
pub use bonding_curve::*;
pub use config::*;
pub use pod::*;
pub use providers::*;
pub use types::*;

/// The Pump.fun program ID on Solana mainnet
pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwbj1";

/// Error types for Pump.fun integration
#[derive(Debug, thiserror::Error)]
pub enum PumpFunError {
    #[error("Solana client error: {0}")]
    SolanaClient(String),

    #[error("Invalid bonding curve state: {0}")]
    InvalidBondingCurve(String),

    #[error("Insufficient liquidity: need {need} SOL, have {have} SOL")]
    InsufficientLiquidity { need: f64, have: f64 },

    #[error("Slippage exceeded: expected {expected}, got {actual}")]
    SlippageExceeded { expected: f64, actual: f64 },

    #[error("Token already graduated to Raydium")]
    TokenGraduated,

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Program error: {0}")]
    ProgramError(String),
}
