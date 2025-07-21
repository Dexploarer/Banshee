//! Pump.fun program instruction builders

use crate::PUMP_FUN_PROGRAM_ID;
use borsh::BorshSerialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Pump.fun instruction types
#[derive(Debug, BorshSerialize)]
pub enum PumpFunInstruction {
    /// Create a new token with bonding curve
    CreateToken {
        name: String,
        symbol: String,
        uri: String,
    },

    /// Buy tokens from the bonding curve
    Buy {
        amount: u64,     // SOL amount in lamports
        min_tokens: u64, // Minimum tokens to receive
    },

    /// Sell tokens to the bonding curve
    Sell {
        amount: u64,  // Token amount
        min_sol: u64, // Minimum SOL to receive in lamports
    },
}

/// Derive the bonding curve PDA for a token
pub fn derive_bonding_curve_pda(token_mint: &Pubkey) -> (Pubkey, u8) {
    let program_id = PUMP_FUN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    Pubkey::find_program_address(&[b"bonding_curve", token_mint.as_ref()], &program_id)
}

/// Derive the fee account PDA
pub fn derive_fee_account_pda() -> (Pubkey, u8) {
    let program_id = PUMP_FUN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    Pubkey::find_program_address(&[b"fee_account"], &program_id)
}

/// Build create token instruction
pub fn create_token_instruction(
    creator: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Result<Instruction, crate::PumpFunError> {
    let program_id = PUMP_FUN_PROGRAM_ID
        .parse::<Pubkey>()
        .map_err(|_| crate::PumpFunError::ProgramError("Invalid program ID".to_string()))?;

    // Generate token mint keypair deterministically from creator and name
    let (token_mint, _) =
        Pubkey::find_program_address(&[b"token", creator.as_ref(), name.as_bytes()], &program_id);

    let (bonding_curve, _) = derive_bonding_curve_pda(&token_mint);
    let (fee_account, _) = derive_fee_account_pda();

    let data = PumpFunInstruction::CreateToken { name, symbol, uri }
        .try_to_vec()
        .map_err(|e| crate::PumpFunError::SerializationError(e.to_string()))?;

    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(*creator, true), // Creator (signer, pays for creation)
            AccountMeta::new(token_mint, false), // Token mint PDA
            AccountMeta::new(bonding_curve, false), // Bonding curve PDA
            AccountMeta::new_readonly(fee_account, false), // Fee account
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    })
}

/// Build buy instruction
pub fn buy_instruction(
    buyer: &Pubkey,
    token_mint: &Pubkey,
    amount_sol: u64,
    min_tokens: u64,
    referrer: Option<&Pubkey>,
) -> Result<Instruction, crate::PumpFunError> {
    let program_id = PUMP_FUN_PROGRAM_ID
        .parse::<Pubkey>()
        .map_err(|_| crate::PumpFunError::ProgramError("Invalid program ID".to_string()))?;

    let (bonding_curve, _) = derive_bonding_curve_pda(token_mint);
    let (fee_account, _) = derive_fee_account_pda();

    // Derive buyer's token account
    let buyer_token_account =
        spl_associated_token_account::get_associated_token_address(buyer, token_mint);

    let data = PumpFunInstruction::Buy {
        amount: amount_sol,
        min_tokens,
    }
    .try_to_vec()
    .map_err(|e| crate::PumpFunError::SerializationError(e.to_string()))?;

    let mut accounts = vec![
        AccountMeta::new(*buyer, true),               // Buyer (signer)
        AccountMeta::new(buyer_token_account, false), // Buyer's token account
        AccountMeta::new(*token_mint, false),         // Token mint
        AccountMeta::new(bonding_curve, false),       // Bonding curve PDA
        AccountMeta::new(fee_account, false),         // Fee account
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    // Add referrer if provided
    if let Some(ref_pubkey) = referrer {
        accounts.push(AccountMeta::new(*ref_pubkey, false));
    }

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

/// Build sell instruction
pub fn sell_instruction(
    seller: &Pubkey,
    token_mint: &Pubkey,
    amount_tokens: u64,
    min_sol: u64,
    referrer: Option<&Pubkey>,
) -> Result<Instruction, crate::PumpFunError> {
    let program_id = PUMP_FUN_PROGRAM_ID
        .parse::<Pubkey>()
        .map_err(|_| crate::PumpFunError::ProgramError("Invalid program ID".to_string()))?;

    let (bonding_curve, _) = derive_bonding_curve_pda(token_mint);
    let (fee_account, _) = derive_fee_account_pda();

    // Derive seller's token account
    let seller_token_account =
        spl_associated_token_account::get_associated_token_address(seller, token_mint);

    let data = PumpFunInstruction::Sell {
        amount: amount_tokens,
        min_sol,
    }
    .try_to_vec()
    .map_err(|e| crate::PumpFunError::SerializationError(e.to_string()))?;

    let mut accounts = vec![
        AccountMeta::new(*seller, true),               // Seller (signer)
        AccountMeta::new(seller_token_account, false), // Seller's token account
        AccountMeta::new(*token_mint, false),          // Token mint
        AccountMeta::new(bonding_curve, false),        // Bonding curve PDA
        AccountMeta::new(fee_account, false),          // Fee account
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    // Add referrer if provided
    if let Some(ref_pubkey) = referrer {
        accounts.push(AccountMeta::new(*ref_pubkey, false));
    }

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}
