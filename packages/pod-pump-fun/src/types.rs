//! Type definitions for Pump.fun bonding curve integration

use borsh::{BorshDeserialize, BorshSerialize};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Bonding curve state as stored on-chain
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BondingCurveState {
    /// The token mint address
    pub token_mint: Pubkey,

    /// Creator of the bonding curve
    pub creator: Pubkey,

    /// Total supply of tokens
    pub total_supply: u64,

    /// Current reserve of SOL in the curve
    pub sol_reserve: u64,

    /// Current reserve of tokens in the curve
    pub token_reserve: u64,

    /// Whether the curve has graduated to Raydium
    pub graduated: bool,

    /// Timestamp when created
    pub created_at: i64,

    /// Virtual SOL reserves for price calculation
    pub virtual_sol_reserves: u64,

    /// Virtual token reserves for price calculation
    pub virtual_token_reserves: u64,

    /// Initial virtual SOL (usually 30 SOL)
    pub initial_virtual_sol: u64,

    /// Initial virtual tokens (usually 1B tokens)
    pub initial_virtual_tokens: u64,
}

/// Token metadata for Pump.fun tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_url: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
}

/// Parameters for creating a new token on Pump.fun
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenParams {
    pub metadata: TokenMetadata,
    pub initial_buy_amount: Option<Decimal>, // Optional initial buy in SOL
}

/// Buy operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyParams {
    pub token_mint: Pubkey,
    pub sol_amount: Decimal,
    pub min_tokens_out: Option<u64>, // Slippage protection
    pub referrer: Option<Pubkey>,
}

/// Sell operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellParams {
    pub token_mint: Pubkey,
    pub token_amount: u64,
    pub min_sol_out: Option<u64>, // Slippage protection
    pub referrer: Option<Pubkey>,
}

/// Trade type for tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeType {
    Buy,
    Sell,
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub trade_type: TradeType,
    pub token_mint: Pubkey,
    pub sol_amount: Decimal,
    pub token_amount: u64,
    pub price_per_token: Decimal,
    pub transaction_signature: String,
    pub timestamp: i64,
}

/// Price information from bonding curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub token_mint: Pubkey,
    pub price_per_token_sol: Decimal,
    pub market_cap_sol: Decimal,
    pub sol_reserve: Decimal,
    pub token_reserve: u64,
    pub virtual_sol_reserves: Decimal,
    pub virtual_token_reserves: u64,
    pub progress_percentage: f64, // Progress to graduation (0-100)
}

/// Volume statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeStats {
    pub token_mint: Pubkey,
    pub volume_24h_sol: Decimal,
    pub volume_24h_usd: Decimal,
    pub trades_24h: u32,
    pub unique_traders_24h: u32,
    pub buy_volume_24h: Decimal,
    pub sell_volume_24h: Decimal,
}

/// Holder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderInfo {
    pub address: Pubkey,
    pub balance: u64,
    pub percentage: f64,
}

/// Token analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalytics {
    pub token_mint: Pubkey,
    pub price_info: PriceInfo,
    pub volume_stats: VolumeStats,
    pub holder_count: u32,
    pub top_holders: Vec<HolderInfo>,
    pub graduation_eta: Option<i64>, // Estimated time to graduation
}

/// Risk parameters for trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParams {
    pub max_position_size_sol: Decimal,
    pub max_slippage_percentage: f64,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
    pub max_gas_sol: Decimal,
}

/// Emotional trading parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTradingParams {
    /// Buy more when excited/happy
    pub excitement_multiplier: f64,

    /// Sell when fearful
    pub fear_threshold: f64,

    /// FOMO protection - don't chase pumps
    pub fomo_protection: bool,

    /// Greed control - take profits
    pub greed_control_percentage: f64,
}
