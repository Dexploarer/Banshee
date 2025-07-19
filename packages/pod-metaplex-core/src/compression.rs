//! Compressed NFT implementation with Merkle trees

use crate::{types::*, MetaplexError};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    keccak,
    pubkey::Pubkey,
};

/// Compressed NFT manager using Merkle trees
pub struct CompressionManager;

impl CompressionManager {
    /// Create a new Merkle tree for compressed NFTs
    pub fn create_tree_instruction(
        payer: &Pubkey,
        tree: &Pubkey,
        max_depth: u32,
        max_buffer_size: u32,
        canopy_depth: u32,
    ) -> Result<Instruction, MetaplexError> {
        // Use SPL Account Compression program
        let program_id = spl_account_compression::id();

        let data = spl_account_compression::instruction::CreateTreeConfigArgs {
            max_depth,
            max_buffer_size,
            public: Some(false),
        };

        let accounts = vec![
            AccountMeta::new(*tree, false), // Merkle tree account
            AccountMeta::new(*payer, true), // Payer
            AccountMeta::new_readonly(spl_noop::id(), false), // Noop program
        ];

        Ok(
            spl_account_compression::instruction::create_initialize_merkle_tree_instruction(
                accounts,
                max_depth,
                max_buffer_size,
            ),
        )
    }

    /// Mint compressed NFT
    pub fn mint_compressed_nft_instruction(
        tree: &Pubkey,
        leaf_owner: &Pubkey,
        metadata: &CoreAsset,
        proof_path: Vec<AccountMeta>,
    ) -> Result<Instruction, MetaplexError> {
        // Use Bubblegum program for compressed NFTs
        let program_id = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
            .parse::<Pubkey>()
            .map_err(|_| {
                MetaplexError::SerializationError("Invalid Bubblegum program ID".to_string())
            })?;

        // Hash metadata for leaf
        let metadata_hash = hash_metadata(metadata);

        let mut accounts = vec![
            AccountMeta::new(*tree, false),                // Merkle tree
            AccountMeta::new_readonly(*leaf_owner, false), // Leaf owner
            AccountMeta::new_readonly(spl_account_compression::id(), false),
            AccountMeta::new_readonly(spl_noop::id(), false),
        ];

        // Add proof accounts
        accounts.extend(proof_path);

        // Simplified instruction data (would be more complex in production)
        let data = vec![0u8]; // Mint instruction discriminator

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Transfer compressed NFT
    pub fn transfer_compressed_nft_instruction(
        tree: &Pubkey,
        from: &Pubkey,
        to: &Pubkey,
        leaf_index: u32,
        proof: &CompressionProof,
    ) -> Result<Instruction, MetaplexError> {
        let program_id = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
            .parse::<Pubkey>()
            .map_err(|_| {
                MetaplexError::SerializationError("Invalid Bubblegum program ID".to_string())
            })?;

        let mut accounts = vec![
            AccountMeta::new(*tree, false),        // Merkle tree
            AccountMeta::new(*from, true),         // Current owner
            AccountMeta::new_readonly(*to, false), // New owner
            AccountMeta::new_readonly(spl_account_compression::id(), false),
            AccountMeta::new_readonly(spl_noop::id(), false),
        ];

        // Add proof accounts
        for proof_node in &proof.proof {
            accounts.push(AccountMeta::new_readonly(Pubkey::from(*proof_node), false));
        }

        // Instruction data would include leaf index and root
        let mut data = vec![1u8]; // Transfer instruction discriminator
        data.extend_from_slice(&leaf_index.to_le_bytes());
        data.extend_from_slice(&proof.root);

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Calculate storage cost for compressed NFTs
    pub fn calculate_tree_cost(max_depth: u32, canopy_depth: u32) -> rust_decimal::Decimal {
        // Base tree account size
        let tree_size = 1_000; // Base size
        let canopy_size = 2u64.pow(canopy_depth) * 32; // 32 bytes per node
        let total_size = tree_size + canopy_size;

        // Cost calculation (rent exempt)
        let lamports_per_byte = 6_960; // Approximate
        let total_lamports = total_size * lamports_per_byte;

        rust_decimal::Decimal::from(total_lamports) / rust_decimal::Decimal::from(1_000_000_000)
    }

    /// Calculate cost per compressed NFT
    pub fn calculate_cost_per_nft(
        tree_cost_sol: rust_decimal::Decimal,
        max_capacity: u32,
    ) -> rust_decimal::Decimal {
        tree_cost_sol / rust_decimal::Decimal::from(max_capacity)
    }

    /// Get maximum tree capacity
    pub fn get_max_capacity(max_depth: u32) -> u32 {
        2u32.pow(max_depth)
    }
}

/// Hash metadata for Merkle tree leaf
fn hash_metadata(metadata: &CoreAsset) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(metadata.owner.as_ref());
    data.extend_from_slice(metadata.name.as_bytes());
    data.extend_from_slice(metadata.uri.as_bytes());

    keccak::hash(&data).to_bytes()
}

/// Verify Merkle proof
pub fn verify_proof(leaf: [u8; 32], proof: &[[u8; 32]], root: [u8; 32], index: u32) -> bool {
    let mut current = leaf;
    let mut current_index = index;

    for proof_element in proof {
        let (left, right) = if current_index % 2 == 0 {
            (current, *proof_element)
        } else {
            (*proof_element, current)
        };

        current = hash_pair(left, right);
        current_index /= 2;
    }

    current == root
}

/// Hash two nodes together
fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&left);
    data.extend_from_slice(&right);
    keccak::hash(&data).to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_max_capacity() {
        assert_eq!(CompressionManager::get_max_capacity(10), 1024);
        assert_eq!(CompressionManager::get_max_capacity(14), 16384);
        assert_eq!(CompressionManager::get_max_capacity(20), 1048576);
    }

    #[test]
    fn test_cost_calculations() {
        let tree_cost = CompressionManager::calculate_tree_cost(14, 10);
        assert!(tree_cost > dec!(0));
        assert!(tree_cost < dec!(10)); // Should be less than 10 SOL

        let per_nft_cost = CompressionManager::calculate_cost_per_nft(tree_cost, 16384);
        assert!(per_nft_cost < dec!(0.001)); // Should be less than 0.001 SOL per NFT
    }

    #[test]
    fn test_merkle_proof_verification() {
        let leaf = [1u8; 32];
        let sibling = [2u8; 32];
        let root = hash_pair(leaf, sibling);

        let proof = vec![sibling];
        assert!(verify_proof(leaf, &proof, root, 0));
        assert!(!verify_proof(leaf, &proof, root, 1));
    }
}
