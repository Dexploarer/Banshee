//! Bundle builder for MEV extraction

use crate::{tip_router::TipRouter, types::*, JitoError};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Bundle builder for creating MEV bundles
pub struct BundleBuilder {
    transactions: Vec<String>, // Base58 encoded transactions
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

    /// Add a transaction to the bundle (base58 encoded)
    pub fn add_transaction(mut self, transaction: String) -> Self {
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

    /// Build the final bundle with tip transaction data
    pub fn build(
        self,
        payer_address: &str,
        tip_receiver: Option<&str>,
    ) -> Result<JitoBundle, JitoError> {
        if self.transactions.is_empty() {
            return Err(JitoError::BundleSubmissionFailed(
                "Bundle is empty".to_string(),
            ));
        }

        // Create tip instruction data
        let _tip_instruction_data = TipRouter::create_tip_instruction_data(
            payer_address,
            self.tip_amount_lamports,
            tip_receiver,
        )?;

        // Note: In production, the actual tip transaction would be created
        // and signed by the TypeScript bridge using Solana Agent Kit

        Ok(JitoBundle {
            transactions: self.transactions,
            bundle_id: uuid::Uuid::new_v4().to_string(),
            tip_amount_lamports: self.tip_amount_lamports,
            target_slot: self.target_slot,
            max_retries: 3,
        })
    }

    /// Create arbitrage bundle
    pub fn create_arbitrage_bundle(
        dex_a_swap: String,
        dex_b_swap: String,
        tip_sol: Decimal,
        payer_address: &str,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(dex_a_swap)
            .add_transaction(dex_b_swap)
            .with_tip(tip_sol)?
            .build(payer_address, None)
    }

    /// Create sandwich bundle
    pub fn create_sandwich_bundle(
        front_transaction: String,
        victim_transaction: String,
        back_transaction: String,
        tip_sol: Decimal,
        payer_address: &str,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(front_transaction)
            .add_transaction(victim_transaction)
            .add_transaction(back_transaction)
            .with_tip(tip_sol)?
            .build(payer_address, None)
    }

    /// Create backrun bundle
    pub fn create_backrun_bundle(
        target_transaction: String,
        backrun_transaction: String,
        tip_sol: Decimal,
        payer_address: &str,
    ) -> Result<JitoBundle, JitoError> {
        Self::new()
            .add_transaction(target_transaction)
            .add_transaction(backrun_transaction)
            .with_tip(tip_sol)?
            .build(payer_address, None)
    }

    /// Simulate bundle for profit calculation
    pub async fn simulate_bundle(
        &self,
        _initial_balances: &[(String, u64)], // (address, lamports)
    ) -> Result<Decimal, JitoError> {
        // In a real implementation, this would:
        // 1. Connect to Jito's simulation endpoint via TypeScript bridge
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

impl Default for BundleBuilder {
    fn default() -> Self {
        Self::new()
    }
}
