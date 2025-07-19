//! Bundle builder for MEV extraction

use crate::{tip_router::TipRouter, types::*, JitoError};
use rust_decimal::Decimal;
use solana_sdk::{
    instruction::Instruction, message::Message, pubkey::Pubkey, signature::Keypair,
    transaction::Transaction,
};

/// Bundle builder for creating MEV bundles
pub struct BundleBuilder {
    transactions: Vec<Transaction>,
    tip_amount_lamports: u64,
    target_slot: Option<u64>,
}

impl BundleBuilder {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            tip_amount_lamports: 0,
            target_slot: None,
        }
    }

    /// Add a transaction to the bundle
    pub fn add_transaction(mut self, transaction: Transaction) -> Self {
        self.transactions.push(transaction);
        self
    }

    /// Set tip amount for the bundle
    pub fn with_tip(mut self, tip_sol: Decimal) -> Result<Self, JitoError> {
        let tip_lamports = (tip_sol * Decimal::from(1_000_000_000))
            .round()
            .to_u64()
            .ok_or_else(|| JitoError::SerializationError("Invalid tip amount".to_string()))?;

        self.tip_amount_lamports = tip_lamports;
        Ok(self)
    }

    /// Set target slot for bundle landing
    pub fn with_target_slot(mut self, slot: u64) -> Self {
        self.target_slot = Some(slot);
        self
    }

    /// Build the final bundle with tip transaction
    pub fn build(
        self,
        payer: &Keypair,
        tip_receiver: Option<&Pubkey>,
    ) -> Result<JitoBundle, JitoError> {
        if self.transactions.is_empty() {
            return Err(JitoError::BundleSubmissionFailed(
                "Bundle is empty".to_string(),
            ));
        }

        // Create tip transaction
        let tip_instruction = TipRouter::create_tip_instruction(
            &payer.pubkey(),
            self.tip_amount_lamports,
            tip_receiver,
        )?;

        let recent_blockhash = solana_sdk::hash::Hash::default(); // Would get from RPC
        let tip_message = Message::new(&[tip_instruction], Some(&payer.pubkey()));
        let mut tip_transaction = Transaction::new_unsigned(tip_message);
        tip_transaction.sign(&[payer], recent_blockhash);

        // Serialize all transactions
        let mut serialized_transactions = Vec::new();

        // Add user transactions first
        for tx in self.transactions {
            let serialized = bs58::encode(
                bincode::serialize(&tx)
                    .map_err(|e| JitoError::SerializationError(e.to_string()))?,
            )
            .into_string();
            serialized_transactions.push(serialized);
        }

        // Add tip transaction last
        serialized_transactions.push(
            bs58::encode(
                bincode::serialize(&tip_transaction)
                    .map_err(|e| JitoError::SerializationError(e.to_string()))?,
            )
            .into_string(),
        );

        Ok(JitoBundle {
            transactions: serialized_transactions,
            bundle_id: uuid::Uuid::new_v4().to_string(),
            tip_amount_lamports: self.tip_amount_lamports,
            target_slot: self.target_slot,
            max_retries: 3,
        })
    }

    /// Create arbitrage bundle
    pub fn create_arbitrage_bundle(
        dex_a_swap: Transaction,
        dex_b_swap: Transaction,
        tip_sol: Decimal,
        payer: &Keypair,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(dex_a_swap)
            .add_transaction(dex_b_swap)
            .with_tip(tip_sol)?
            .build(payer, None)
    }

    /// Create sandwich bundle
    pub fn create_sandwich_bundle(
        front_transaction: Transaction,
        victim_transaction: Transaction,
        back_transaction: Transaction,
        tip_sol: Decimal,
        payer: &Keypair,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(front_transaction)
            .add_transaction(victim_transaction)
            .add_transaction(back_transaction)
            .with_tip(tip_sol)?
            .build(payer, None)
    }

    /// Create backrun bundle
    pub fn create_backrun_bundle(
        target_transaction: Transaction,
        backrun_transaction: Transaction,
        tip_sol: Decimal,
        payer: &Keypair,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(target_transaction)
            .add_transaction(backrun_transaction)
            .with_tip(tip_sol)?
            .build(payer, None)
    }

    /// Simulate bundle for profit calculation
    pub async fn simulate_bundle(
        &self,
        initial_balances: &[(Pubkey, u64)],
    ) -> Result<Decimal, JitoError> {
        // In a real implementation, this would:
        // 1. Connect to Jito's simulation endpoint
        // 2. Submit the bundle for simulation
        // 3. Calculate profit from balance changes

        // Mock simulation
        let mock_profit = Decimal::new(15, 1); // 1.5 SOL profit

        // Subtract tip to get net profit
        let tip_sol = Decimal::from(self.tip_amount_lamports) / Decimal::from(1_000_000_000);
        let net_profit = mock_profit - tip_sol;

        if net_profit < Decimal::ZERO {
            return Err(JitoError::InsufficientProfit {
                expected: 0.0,
                actual: net_profit.to_f64().unwrap_or(0.0),
            });
        }

        Ok(net_profit)
    }
}
