//! MPL-404 hybrid asset implementation

use crate::{types::*, MetaplexError, MPL_404_PROGRAM_ID};
use rust_decimal::Decimal;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

/// MPL-404 manager for fungible ↔ NFT swaps
pub struct Mpl404Manager;

impl Mpl404Manager {
    /// Create a new MPL-404 hybrid asset
    pub fn create_mpl404_instruction(
        payer: &Pubkey,
        mint: &Pubkey,
        params: &CreateMpl404Params,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = MPL_404_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let royalty_basis_points = (params.royalty_percentage * 100.0) as u16;

        let data = Mpl404Instruction::Create {
            name: params.name.clone(),
            symbol: params.symbol.clone(),
            total_supply: params.total_supply,
            decimals: params.decimals,
            nft_threshold: params.nft_threshold,
            base_uri: params.base_uri.clone(),
            royalty_basis_points,
        };

        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*payer, true), // Payer
                AccountMeta::new(*mint, false), // Token mint
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: data_bytes,
        })
    }

    /// Swap fungible tokens to NFT
    pub fn fungible_to_nft_instruction(
        owner: &Pubkey,
        mint: &Pubkey,
        token_account: &Pubkey,
        nft_mint: &Pubkey,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = MPL_404_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let data = Mpl404Instruction::FungibleToNft;
        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*owner, true),          // Owner
                AccountMeta::new(*mint, false),          // Fungible token mint
                AccountMeta::new(*token_account, false), // Token account
                AccountMeta::new(*nft_mint, false),      // NFT mint to create
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: data_bytes,
        })
    }

    /// Swap NFT to fungible tokens
    pub fn nft_to_fungible_instruction(
        owner: &Pubkey,
        nft_mint: &Pubkey,
        fungible_mint: &Pubkey,
        token_account: &Pubkey,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = MPL_404_PROGRAM_ID
            .parse::<Pubkey>()
            .map_err(|_| MetaplexError::SerializationError("Invalid program ID".to_string()))?;

        let data = Mpl404Instruction::NftToFungible;
        let data_bytes =
            borsh::to_vec(&data).map_err(|e| MetaplexError::SerializationError(e.to_string()))?;

        Ok(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(*owner, true),          // Owner
                AccountMeta::new(*nft_mint, false),      // NFT mint to burn
                AccountMeta::new(*fungible_mint, false), // Fungible token mint
                AccountMeta::new(*token_account, false), // Token account to receive
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: data_bytes,
        })
    }

    /// Check if balance meets NFT threshold
    pub fn should_convert_to_nft(balance: u64, threshold: u64, decimals: u8) -> bool {
        balance >= threshold
    }

    /// Calculate NFT ID from token amount
    pub fn calculate_nft_id(token_amount: u64, threshold: u64, base_counter: u64) -> u64 {
        let nft_count = token_amount / threshold;
        base_counter + nft_count
    }

    /// Calculate swap fee
    pub fn calculate_swap_fee(amount: u64, fee_percentage: f64, decimals: u8) -> u64 {
        let amount_decimal = Decimal::from(amount) / Decimal::from(10u64.pow(decimals as u32));
        let fee = amount_decimal * Decimal::from_f64(fee_percentage / 100.0).unwrap_or_default();
        (fee * Decimal::from(10u64.pow(decimals as u32)))
            .to_u64()
            .unwrap_or(0)
    }

    /// Get NFT metadata URI from base URI and ID
    pub fn get_nft_uri(base_uri: &str, nft_id: u64) -> String {
        format!("{}/{}.json", base_uri.trim_end_matches('/'), nft_id)
    }
}

/// MPL-404 instruction types
#[derive(Debug, borsh::BorshSerialize)]
enum Mpl404Instruction {
    Create {
        name: String,
        symbol: String,
        total_supply: u64,
        decimals: u8,
        nft_threshold: u64,
        base_uri: String,
        royalty_basis_points: u16,
    },
    FungibleToNft,
    NftToFungible,
    UpdateThreshold {
        new_threshold: u64,
    },
    UpdateBaseUri {
        new_base_uri: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_threshold_check() {
        assert!(Mpl404Manager::should_convert_to_nft(
            1_000_000, 1_000_000, 6
        ));
        assert!(Mpl404Manager::should_convert_to_nft(
            2_000_000, 1_000_000, 6
        ));
        assert!(!Mpl404Manager::should_convert_to_nft(999_999, 1_000_000, 6));
    }

    #[test]
    fn test_nft_id_calculation() {
        let id = Mpl404Manager::calculate_nft_id(3_000_000, 1_000_000, 100);
        assert_eq!(id, 103); // Base 100 + 3 NFTs
    }

    #[test]
    fn test_swap_fee_calculation() {
        let fee = Mpl404Manager::calculate_swap_fee(1_000_000, 1.0, 6);
        assert_eq!(fee, 10_000); // 1% of 1 token (with 6 decimals)
    }

    #[test]
    fn test_nft_uri_generation() {
        let uri = Mpl404Manager::get_nft_uri("https://api.example.com/metadata", 42);
        assert_eq!(uri, "https://api.example.com/metadata/42.json");

        let uri_trailing = Mpl404Manager::get_nft_uri("https://api.example.com/metadata/", 42);
        assert_eq!(uri_trailing, "https://api.example.com/metadata/42.json");
    }
}
