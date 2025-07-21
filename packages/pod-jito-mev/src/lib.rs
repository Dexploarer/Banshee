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
pub mod error;
pub mod ffi;
pub mod pod;
pub mod providers;
pub mod tip_router;
pub mod types;

pub use actions::*;
pub use bundle_builder::*;
pub use config::*;
pub use error::{JitoError, Result};
pub use pod::*;
pub use providers::*;
pub use tip_router::*;
pub use types::*;

/// Jito block engine endpoints for July 2025
pub const JITO_BLOCK_ENGINE_MAINNET: &str = "mainnet.block-engine.jito.wtf";
pub const JITO_BLOCK_ENGINE_DEVNET: &str = "devnet.block-engine.jito.wtf";

/// TipRouter program ID (July 2025 upgrade)
pub const TIP_ROUTER_PROGRAM_ID: &str = "TiPR8JitoYbYj5zpRV8kPnWNkL7PrsvqSndVFSBYYj";
