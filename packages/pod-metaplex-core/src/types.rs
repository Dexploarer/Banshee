//! Type definitions for Metaplex Core and MPL-404

use borsh::{BorshDeserialize, BorshSerialize};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Core asset type with single-account design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreAsset {
    /// Asset address (single account)
    pub address: Pubkey,

    /// Owner of the asset
    pub owner: Pubkey,

    /// Update authority
    pub update_authority: Option<Pubkey>,

    /// Asset name
    pub name: String,

    /// Asset URI (metadata)
    pub uri: String,

    /// Collection the asset belongs to
    pub collection: Option<Pubkey>,

    /// Royalty basis points (0-10000)
    pub royalty_basis_points: u16,

    /// Creators and their shares
    pub creators: Vec<Creator>,

    /// Whether the asset is frozen
    pub frozen: bool,

    /// Compression proof if compressed
    pub compression_proof: Option<CompressionProof>,
}

/// Creator information
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Creator {
    pub address: Pubkey,
    pub share: u8, // Percentage (0-100)
    pub verified: bool,
}

/// MPL-404 hybrid asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mpl404Asset {
    /// Token mint address
    pub mint: Pubkey,

    /// Current state
    pub state: Mpl404State,

    /// Total supply
    pub total_supply: u64,

    /// Fungible decimals
    pub decimals: u8,

    /// NFT threshold (tokens needed to convert to NFT)
    pub nft_threshold: u64,

    /// Base URI for NFT metadata
    pub base_uri: String,

    /// Current NFT ID counter
    pub nft_counter: u64,
}

/// MPL-404 asset state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mpl404State {
    /// Fully fungible state
    Fungible,
    /// NFT state
    NonFungible { nft_id: u64 },
    /// Partial state (some fungible, some NFT)
    Hybrid,
}

/// Compression proof for compressed NFTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionProof {
    /// Merkle tree address
    pub tree: Pubkey,

    /// Leaf index in the tree
    pub leaf_index: u32,

    /// Merkle proof path
    pub proof: Vec<[u8; 32]>,

    /// Root hash
    pub root: [u8; 32],
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub address: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub description: String,
    pub size: u32,
    pub verified: bool,
}

/// Asset creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetParams {
    pub name: String,
    pub uri: String,
    pub collection: Option<Pubkey>,
    pub royalty_percentage: f64,
    pub creators: Vec<Creator>,
    pub compress: bool,
    pub emotional_theme: Option<EmotionalTheme>,
}

/// MPL-404 creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMpl404Params {
    pub name: String,
    pub symbol: String,
    pub total_supply: u64,
    pub decimals: u8,
    pub nft_threshold: u64,
    pub base_uri: String,
    pub royalty_percentage: f64,
}

/// Emotional themes for NFT collections
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmotionalTheme {
    Joy,
    Melancholy,
    Excitement,
    Serenity,
    Chaos,
    Harmony,
}

/// Asset analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAnalytics {
    pub asset: Pubkey,
    pub floor_price_sol: Decimal,
    pub volume_24h_sol: Decimal,
    pub sales_24h: u32,
    pub holders: u32,
    pub listed_count: u32,
    pub average_price_7d: Decimal,
    pub price_change_24h_percentage: f64,
}

/// Collection analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionAnalytics {
    pub collection: Pubkey,
    pub floor_price_sol: Decimal,
    pub volume_all_time_sol: Decimal,
    pub volume_24h_sol: Decimal,
    pub sales_24h: u32,
    pub unique_holders: u32,
    pub total_supply: u32,
    pub listed_percentage: f64,
    pub royalties_earned_sol: Decimal,
}

/// Compressed NFT batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBatch {
    pub tree: Pubkey,
    pub assets: Vec<CompressedAssetInfo>,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
}

/// Compressed asset info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedAssetInfo {
    pub index: u32,
    pub owner: Pubkey,
    pub metadata_hash: [u8; 32],
    pub data_hash: [u8; 32],
    pub creator_hash: [u8; 32],
}

/// Transfer parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferParams {
    pub asset: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: Option<u64>, // For MPL-404 fungible transfers
    pub compression_proof: Option<CompressionProof>,
}

/// Update metadata parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadataParams {
    pub asset: Pubkey,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub royalty_basis_points: Option<u16>,
    pub creators: Option<Vec<Creator>>,
}

/// Burn parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnParams {
    pub asset: Pubkey,
    pub owner: Pubkey,
    pub amount: Option<u64>, // For MPL-404
    pub compression_proof: Option<CompressionProof>,
}
