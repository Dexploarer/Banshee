//! # Pod-Metaplex-Core - Advanced NFT and Digital Asset Management
//!
//! This pod provides integration with Metaplex Core and MPL-404 for:
//! - Single-account NFT design (85% cost reduction)
//! - MPL-404 hybrid assets (fungible ↔ NFT swaps)
//! - Compressed NFTs with Merkle trees
//! - Emotional NFT collections based on agent state
//! - Dynamic metadata updates
//! - Royalty enforcement

pub mod actions;
pub mod compression;
pub mod config;
pub mod core_asset;
pub mod mpl404;
pub mod pod;
pub mod providers;
pub mod types;

pub use actions::*;
pub use compression::*;
pub use config::*;
pub use core_asset::*;
pub use mpl404::*;
pub use pod::*;
pub use providers::*;
pub use types::*;

/// Metaplex Core program ID (July 2025)
pub const METAPLEX_CORE_PROGRAM_ID: &str = "CoREkBFe8nbudTGfPbLDiRKf1skVK8Uz5misVjJWZi8";

/// MPL-404 program ID
pub const MPL_404_PROGRAM_ID: &str = "MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb";

/// Error types for Metaplex integration
#[derive(Debug, thiserror::Error)]
pub enum MetaplexError {
    #[error("Asset creation failed: {0}")]
    AssetCreationFailed(String),

    #[error("Metadata update failed: {0}")]
    MetadataUpdateFailed(String),

    #[error("MPL-404 swap failed: {0}")]
    Mpl404SwapFailed(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Collection error: {0}")]
    CollectionError(String),

    #[error("Royalty validation failed: expected {expected}%, got {actual}%")]
    RoyaltyValidationFailed { expected: f64, actual: f64 },

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Insufficient balance for operation")]
    InsufficientBalance,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
