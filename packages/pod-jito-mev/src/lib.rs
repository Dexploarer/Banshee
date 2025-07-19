//! # Pod-Jito-MEV - Maximum Extractable Value with Jito TipRouter
//!
//! This pod provides integration with Jito's MEV infrastructure, including:
//! - TipRouter for decentralized tip distribution to stakers
//! - Bundle submission for MEV extraction
//! - Backrun and sandwich protection/execution
//! - Staking rewards optimization (7% APY boost)
//! - Emotional intelligence for MEV opportunities

pub mod actions;
pub mod bundle_builder;
pub mod config;
pub mod pod;
pub mod providers;
pub mod tip_router;
pub mod types;

pub use actions::*;
pub use bundle_builder::*;
pub use config::*;
pub use pod::*;
pub use providers::*;
pub use tip_router::*;
pub use types::*;

/// Jito block engine endpoints for July 2025
pub const JITO_BLOCK_ENGINE_MAINNET: &str = "mainnet.block-engine.jito.wtf";
pub const JITO_BLOCK_ENGINE_DEVNET: &str = "devnet.block-engine.jito.wtf";

/// TipRouter program ID (July 2025 upgrade)
pub const TIP_ROUTER_PROGRAM_ID: &str = "TiPR8JitoYbYj5zpRV8kPnWNkL7PrsvqSndVFSBYYj";

/// Error types for Jito MEV integration
#[derive(Debug, thiserror::Error)]
pub enum JitoError {
    #[error("Bundle submission failed: {0}")]
    BundleSubmissionFailed(String),

    #[error("Tip amount too low: minimum {min} SOL, provided {provided} SOL")]
    TipTooLow { min: f64, provided: f64 },

    #[error("MEV opportunity expired")]
    OpportunityExpired,

    #[error("Insufficient profit margin: expected {expected}%, got {actual}%")]
    InsufficientProfit { expected: f64, actual: f64 },

    #[error("Bundle simulation failed: {0}")]
    SimulationFailed(String),

    #[error("TipRouter error: {0}")]
    TipRouterError(String),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
