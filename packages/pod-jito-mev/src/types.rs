//! Type definitions for Jito MEV integration

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// MEV opportunity types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MevType {
    /// Arbitrage between DEXs
    Arbitrage,
    /// Liquidation opportunity
    Liquidation,
    /// Sandwich trading
    Sandwich,
    /// Backrun a transaction
    Backrun,
    /// Front-run protection
    FrontrunProtection,
    /// NFT sniping
    NftSnipe,
    /// Staking optimization
    StakingOptimization,
}

/// MEV opportunity details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevOpportunity {
    pub id: String,
    pub mev_type: MevType,
    pub target_transaction: Option<String>,
    pub estimated_profit_sol: Decimal,
    pub required_capital_sol: Decimal,
    pub confidence_score: f64,
    pub expiry_slot: u64,
    pub risk_level: RiskLevel,
    pub emotional_signal: EmotionalSignal,
}

/// Risk levels for MEV opportunities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Emotional signals for MEV trading
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionalSignal {
    /// Greed level (0-1) - higher means more aggressive
    pub greed: f64,
    /// Fear level (0-1) - higher means more cautious
    pub fear: f64,
    /// Confidence level (0-1) - higher means more certain
    pub confidence: f64,
}

/// Jito bundle for MEV extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoBundle {
    pub transactions: Vec<String>, // Base58 encoded transactions
    pub bundle_id: String,
    pub tip_amount_lamports: u64,
    pub target_slot: Option<u64>,
    pub max_retries: u32,
}

/// Bundle submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResult {
    pub bundle_id: String,
    pub accepted: bool,
    pub landed_slot: Option<u64>,
    pub profit_sol: Decimal,
    pub tip_paid_sol: Decimal,
    pub net_profit_sol: Decimal,
    pub transaction_signatures: Vec<String>,
}

/// TipRouter configuration for July 2025
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TipRouterConfig {
    /// Tip distribution percentage to stakers (default: 97%)
    pub staker_percentage: f64,
    /// Tip distribution percentage to validators (default: 3%)
    pub validator_percentage: f64,
    /// Minimum tip amount in SOL
    pub min_tip_sol: Decimal,
    /// Use dynamic tip calculation based on MEV profit
    pub dynamic_tips: bool,
    /// Tip percentage of gross profit (if dynamic)
    pub dynamic_tip_percentage: f64,
}

/// Staking rewards information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRewards {
    pub validator: String, // Base58 string instead of Pubkey
    pub base_apy: f64,
    pub mev_boost_apy: f64,
    pub total_apy: f64,
    pub stake_amount_sol: Decimal,
    pub estimated_yearly_rewards_sol: Decimal,
    pub mev_tips_received_24h: Decimal,
}

/// MEV protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtection {
    /// Enable sandwich attack protection
    pub anti_sandwich: bool,
    /// Enable frontrun protection
    pub anti_frontrun: bool,
    /// Maximum slippage tolerance
    pub max_slippage_percentage: f64,
    /// Use private mempool submission
    pub use_private_mempool: bool,
}

/// Bundle builder parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleParams {
    pub mev_type: MevType,
    pub transactions: Vec<TransactionInfo>,
    pub tip_amount_sol: Decimal,
    pub priority_fee_lamports: u64,
    pub max_retries: u32,
    pub simulation_required: bool,
}

/// Transaction information for bundle building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub instruction_data: Vec<u8>,
    pub accounts: Vec<AccountInfo>,
    pub signer: Option<String>, // Base58 string instead of Pubkey
}

/// Account information for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub pubkey: String, // Base58 string instead of Pubkey
    pub is_signer: bool,
    pub is_writable: bool,
}

/// MEV analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevAnalytics {
    pub total_opportunities_24h: u32,
    pub successful_bundles_24h: u32,
    pub total_profit_24h_sol: Decimal,
    pub total_tips_paid_24h_sol: Decimal,
    pub average_profit_per_bundle_sol: Decimal,
    pub success_rate: f64,
    pub top_mev_types: Vec<(MevType, u32)>,
}

/// Validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    pub validator: String, // Base58 string instead of Pubkey
    pub slots_processed_24h: u32,
    pub bundles_landed_24h: u32,
    pub mev_tips_earned_24h_sol: Decimal,
    pub average_slot_time_ms: f64,
    pub reliability_score: f64,
}
