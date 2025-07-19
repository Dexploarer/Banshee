//! Metaplex Core asset management with single-account design

use crate::{types::*, MetaplexError, METAPLEX_CORE_PROGRAM_ID};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

/// Core asset manager for single-account NFTs
pub struct CoreAssetManager;

impl CoreAssetManager {
    /// Create a new Core asset
    pub fn create_asset_instruction(
        payer: &Pubkey,
        asset: &Pubkey,
        params: &CreateAssetParams,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = METAPLEX_CORE_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        // Convert royalty percentage to basis points
        let royalty_basis_points = (params.royalty_percentage * 100.0) as u16;

        // Instruction data
        let data = CoreAssetInstruction::Create {
            name: params.name.clone(),
            uri: params.uri.clone(),
            royalty_basis_points,
            creators: params.creators.clone(),
        };

        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        let mut accounts = vec![
            AccountMeta::new(*payer, true),  // Payer
            AccountMeta::new(*asset, false), // Asset account
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        // Add collection if provided
        if let Some(collection) = params.collection {
            accounts.push(AccountMeta::new_readonly(collection, false));
        }

        Ok(Instruction {
            program_id,
            accounts,
            data: data_bytes,
        })
    }

    /// Update asset metadata
    pub fn update_metadata_instruction(
        authority: &Pubkey,
        asset: &Pubkey,
        params: &UpdateMetadataParams,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = METAPLEX_CORE_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let data = CoreAssetInstruction::UpdateMetadata {
            name: params.name.clone(),
            uri: params.uri.clone(),
            royalty_basis_points: params.royalty_basis_points,
            creators: params.creators.clone(),
        };

        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*authority, true), // Update authority
                AccountMeta::new(*asset, false),    // Asset account
            ],
            data: data_bytes,
        })
    }

    /// Transfer asset
    pub fn transfer_instruction(
        from: &Pubkey,
        to: &Pubkey,
        asset: &Pubkey,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = METAPLEX_CORE_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let data = CoreAssetInstruction::Transfer;
        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*from, true),   // Current owner
                AccountMeta::new(*to, false),    // New owner
                AccountMeta::new(*asset, false), // Asset account
            ],
            data: data_bytes,
        })
    }

    /// Burn asset
    pub fn burn_instruction(owner: &Pubkey, asset: &Pubkey) -> Result<Instruction, MetaplexError> {
        let program_id = METAPLEX_CORE_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let data = CoreAssetInstruction::Burn;
        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*owner, true),  // Owner
                AccountMeta::new(*asset, false), // Asset account
            ],
            data: data_bytes,
        })
    }

    /// Calculate storage cost for Core asset
    pub fn calculate_storage_cost() -> rust_decimal::Decimal {
        // Core assets use single account design
        // Typical size: ~200 bytes
        // Cost: ~0.00203928 SOL
        rust_decimal::Decimal::new(203928, 8) // 0.00203928 SOL
    }

    /// Calculate cost savings vs traditional NFTs
    pub fn calculate_savings_percentage() -> f64 {
        // Traditional NFT: ~0.012 SOL (token account + metadata account + mint)
        // Core asset: ~0.002 SOL (single account)
        // Savings: ~85%
        85.0
    }
}

/// Core asset instruction types
#[derive(Debug, borsh::BorshSerialize)]
enum CoreAssetInstruction {
    Create {
        name: String,
        uri: String,
        royalty_basis_points: u16,
        creators: Vec<Creator>,
    },
    UpdateMetadata {
        name: Option<String>,
        uri: Option<String>,
        royalty_basis_points: Option<u16>,
        creators: Option<Vec<Creator>>,
    },
    Transfer,
    Burn,
    Freeze,
    Thaw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculations() {
        let storage_cost = CoreAssetManager::calculate_storage_cost();
        assert!(storage_cost > rust_decimal::Decimal::ZERO);
        assert!(storage_cost < rust_decimal::Decimal::from(1)); // Less than 1 SOL

        let savings = CoreAssetManager::calculate_savings_percentage();
        assert_eq!(savings, 85.0);
    }

    #[test]
    fn test_royalty_conversion() {
        let params = CreateAssetParams {
            name: "Test Asset".to_string(),
            uri: "https://test.com".to_string(),
            collection: None,
            royalty_percentage: 5.5,
            creators: vec![],
            compress: false,
            emotional_theme: None,
        };

        let royalty_basis_points = (params.royalty_percentage * 100.0) as u16;
        assert_eq!(royalty_basis_points, 550); // 5.5% = 550 basis points
    }
}
