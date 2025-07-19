//! Configuration for Metaplex Core pod

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaplexCoreConfig {
    /// Solana RPC endpoint
    pub rpc_endpoint: String,

    /// Network type
    pub network: NetworkType,

    /// Creator wallet private key (base58)
    pub creator_keypair: Option<String>,

    /// Auto-generate creator wallet if none provided
    pub auto_generate_wallet: bool,

    /// Default royalty percentage for new assets
    pub default_royalty_percentage: f64,

    /// Enable compressed NFTs by default
    pub prefer_compression: bool,

    /// Compression settings
    pub compression: CompressionConfig,

    /// MPL-404 settings
    pub mpl404: Mpl404Config,

    /// Collection settings
    pub collection: CollectionConfig,

    /// Emotional asset generation
    pub emotional_assets: EmotionalAssetConfig,

    /// Priority fee in microlamports
    pub priority_fee: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkType {
    #[serde(rename = "mainnet-beta")]
    MainnetBeta,
    #[serde(rename = "devnet")]
    Devnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Maximum tree depth (3-30)
    pub max_depth: u32,

    /// Maximum buffer size (8-2048)
    pub max_buffer_size: u32,

    /// Canopy depth for caching (0-24)
    pub canopy_depth: u32,

    /// Cost per asset in SOL
    pub cost_per_asset_sol: Decimal,

    /// Batch size for minting
    pub mint_batch_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mpl404Config {
    /// Default NFT threshold
    pub default_nft_threshold: u64,

    /// Enable automatic swaps at threshold
    pub auto_swap: bool,

    /// Swap fee percentage
    pub swap_fee_percentage: f64,

    /// Maximum supply for new MPL-404 tokens
    pub max_supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Verify creators by default
    pub auto_verify_creators: bool,

    /// Collection size limit
    pub max_collection_size: u32,

    /// Enable collection royalties
    pub enable_royalties: bool,

    /// Collection update freeze after mint
    pub freeze_after_mint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalAssetConfig {
    /// Enable emotional theme generation
    pub enabled: bool,

    /// Emotion intensity threshold (0-1)
    pub intensity_threshold: f64,

    /// Dynamic metadata based on emotions
    pub dynamic_metadata: bool,

    /// Emotion decay rate for metadata updates
    pub emotion_decay_hours: u32,
}

impl Default for MetaplexCoreConfig {
    fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            network: NetworkType::MainnetBeta,
            creator_keypair: None,
            auto_generate_wallet: true,
            default_royalty_percentage: 5.0,
            prefer_compression: true,
            compression: CompressionConfig {
                max_depth: 14,
                max_buffer_size: 64,
                canopy_depth: 10,
                cost_per_asset_sol: Decimal::new(1, 5), // 0.00001 SOL
                mint_batch_size: 100,
            },
            mpl404: Mpl404Config {
                default_nft_threshold: 1_000_000, // 1M tokens
                auto_swap: true,
                swap_fee_percentage: 1.0,
                max_supply: 1_000_000_000, // 1B max
            },
            collection: CollectionConfig {
                auto_verify_creators: true,
                max_collection_size: 10_000,
                enable_royalties: true,
                freeze_after_mint: false,
            },
            emotional_assets: EmotionalAssetConfig {
                enabled: false,
                intensity_threshold: 0.7,
                dynamic_metadata: false,
                emotion_decay_hours: 24,
            },
            priority_fee: 10_000, // 0.00001 SOL
        }
    }
}

impl MetaplexCoreConfig {
    /// Create config for mainnet
    pub fn mainnet() -> Self {
        Self::default()
    }

    /// Create config for devnet
    pub fn devnet() -> Self {
        Self {
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network: NetworkType::Devnet,
            compression: CompressionConfig {
                cost_per_asset_sol: Decimal::new(1, 6), // 0.000001 SOL on devnet
                ..Default::default()
            },
            ..Self::default()
        }
    }

    /// Create config for emotional NFT collections
    pub fn emotional() -> Self {
        Self {
            emotional_assets: EmotionalAssetConfig {
                enabled: true,
                intensity_threshold: 0.6,
                dynamic_metadata: true,
                emotion_decay_hours: 12,
            },
            ..Self::default()
        }
    }

    /// Create config optimized for large collections
    pub fn large_collection() -> Self {
        Self {
            prefer_compression: true,
            compression: CompressionConfig {
                max_depth: 20,
                max_buffer_size: 256,
                canopy_depth: 14,
                cost_per_asset_sol: Decimal::new(1, 5),
                mint_batch_size: 1000,
            },
            collection: CollectionConfig {
                max_collection_size: 100_000,
                freeze_after_mint: true,
                ..Default::default()
            },
            ..Self::default()
        }
    }
}
