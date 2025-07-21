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
pub mod error;
pub mod ffi;
pub mod instructions;
pub mod pod;
pub mod providers;
pub mod types;

pub use actions::*;
pub use bonding_curve::*;
pub use config::*;
pub use error::{PumpFunError, Result};
pub use pod::*;
pub use providers::*;
pub use types::*;

/// The Pump.fun program ID on Solana mainnet
pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwbj1";
