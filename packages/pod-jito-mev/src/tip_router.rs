//! TipRouter integration for decentralized tip distribution

use crate::{JitoError, TIP_ROUTER_PROGRAM_ID};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// TipRouter handles the July 2025 upgrade for decentralized tip distribution
pub struct TipRouter;

impl TipRouter {
    /// Calculate tip distribution between stakers and validators
    pub fn calculate_distribution(
        total_tip_sol: Decimal,
        staker_percentage: f64,
        validator_percentage: f64,
    ) -> Result<(Decimal, Decimal), JitoError> {
        if (staker_percentage + validator_percentage - 100.0).abs() > 0.01 {
            return Err(JitoError::TipRouterError(
                "Percentages must sum to 100%".to_string(),
            ));
        }

        let staker_amount = total_tip_sol
            * Decimal::from_f64(staker_percentage / 100.0)
                .ok_or_else(|| JitoError::TipRouterError("Invalid percentage".to_string()))?;
        let validator_amount = total_tip_sol
            * Decimal::from_f64(validator_percentage / 100.0)
                .ok_or_else(|| JitoError::TipRouterError("Invalid percentage".to_string()))?;

        Ok((staker_amount, validator_amount))
    }

    /// Create tip instruction data for TipRouter (to be used with TypeScript bridge)
    pub fn create_tip_instruction_data(
        tipper_address: &str,
        tip_amount_lamports: u64,
        target_validator: Option<&str>,
    ) -> Result<serde_json::Value, JitoError> {
        // Return JSON data that can be passed to TypeScript bridge
        Ok(serde_json::json!({
            "program_id": TIP_ROUTER_PROGRAM_ID,
            "tipper": tipper_address,
            "tip_amount_lamports": tip_amount_lamports,
            "target_validator": target_validator,
        }))
    }

    /// Calculate dynamic tip based on MEV profit
    pub fn calculate_dynamic_tip(
        gross_profit_sol: Decimal,
        dynamic_tip_percentage: f64,
        min_tip_sol: Decimal,
    ) -> Decimal {
        let calculated_tip = gross_profit_sol
            * Decimal::from_f64(dynamic_tip_percentage / 100.0).unwrap_or(Decimal::ZERO);

        // Ensure minimum tip is met
        calculated_tip.max(min_tip_sol)
    }

    /// Estimate APY boost from MEV tips
    pub fn estimate_apy_boost(
        stake_amount_sol: Decimal,
        daily_tips_sol: Decimal,
        base_apy: f64,
    ) -> f64 {
        if stake_amount_sol.is_zero() {
            return base_apy;
        }

        // Calculate annualized tip return
        let annual_tips = daily_tips_sol * Decimal::from(365);
        let tip_apy = (annual_tips / stake_amount_sol * Decimal::from(100))
            .to_f64()
            .unwrap_or(0.0);

        // Add to base APY (typically adds ~7% on mainnet)
        base_apy + tip_apy
    }

    /// Get optimal validators for MEV submission based on performance
    pub fn get_optimal_validators(
        validator_metrics: &[(String, f64)], // (validator_address, reliability_score)
        max_validators: usize,
    ) -> Vec<String> {
        let mut sorted_validators = validator_metrics.to_vec();
        sorted_validators.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        sorted_validators
            .into_iter()
            .take(max_validators)
            .map(|(validator, _)| validator)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tip_distribution() {
        let total_tip = dec!(1.0); // 1 SOL
        let (staker_amount, validator_amount) =
            TipRouter::calculate_distribution(total_tip, 97.0, 3.0).unwrap();

        assert_eq!(staker_amount, dec!(0.97));
        assert_eq!(validator_amount, dec!(0.03));
    }

    #[test]
    fn test_dynamic_tip_calculation() {
        let gross_profit = dec!(10.0); // 10 SOL profit
        let tip = TipRouter::calculate_dynamic_tip(gross_profit, 20.0, dec!(0.001));

        assert_eq!(tip, dec!(2.0)); // 20% of 10 SOL

        // Test minimum tip
        let small_profit = dec!(0.001);
        let small_tip = TipRouter::calculate_dynamic_tip(small_profit, 20.0, dec!(0.001));
        assert_eq!(small_tip, dec!(0.001)); // Minimum tip enforced
    }

    #[test]
    fn test_apy_boost_calculation() {
        let stake_amount = dec!(1000.0); // 1000 SOL staked
        let daily_tips = dec!(0.2); // 0.2 SOL daily tips
        let base_apy = 5.0; // 5% base APY

        let boosted_apy = TipRouter::estimate_apy_boost(stake_amount, daily_tips, base_apy);

        // 0.2 SOL * 365 days = 73 SOL annually
        // 73 / 1000 * 100 = 7.3% boost
        // Total APY = 5% + 7.3% = 12.3%
        assert!((boosted_apy - 12.3).abs() < 0.1);
    }
}
