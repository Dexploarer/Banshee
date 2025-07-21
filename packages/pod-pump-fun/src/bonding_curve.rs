//! Bonding curve mathematics and calculations

use crate::types::{BondingCurveState, PriceInfo};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The Pump.fun bonding curve uses a constant product AMM (x * y = k)
/// with virtual reserves to ensure smooth price discovery
pub struct BondingCurveMath;

impl BondingCurveMath {
    /// Calculate the current price per token in SOL
    pub fn calculate_price(state: &BondingCurveState) -> Decimal {
        let sol_reserves = Decimal::from(state.virtual_sol_reserves) / dec!(1_000_000_000); // lamports to SOL
        let token_reserves = Decimal::from(state.virtual_token_reserves);

        if token_reserves.is_zero() {
            return Decimal::ZERO;
        }

        sol_reserves / token_reserves
    }

    /// Calculate how many tokens you get for a given SOL amount
    pub fn calculate_tokens_out(
        state: &BondingCurveState,
        sol_in: Decimal,
    ) -> Result<u64, crate::PumpFunError> {
        let sol_in_lamports = (sol_in * dec!(1_000_000_000))
            .round()
            .to_u64()
            .ok_or_else(|| {
                crate::PumpFunError::InvalidBondingCurve("Invalid SOL amount".to_string())
            })?;

        // Apply constant product formula: (x + dx) * (y - dy) = x * y
        let k = state.virtual_sol_reserves as u128 * state.virtual_token_reserves as u128;
        let new_sol_reserves = state.virtual_sol_reserves as u128 + sol_in_lamports as u128;

        if new_sol_reserves == 0 {
            return Err(crate::PumpFunError::InvalidBondingCurve(
                "Invalid reserves".to_string(),
            ));
        }

        let new_token_reserves = k / new_sol_reserves;
        let tokens_out = (state.virtual_token_reserves as u128).saturating_sub(new_token_reserves);

        Ok(tokens_out as u64)
    }

    /// Calculate how much SOL you get for selling tokens
    pub fn calculate_sol_out(
        state: &BondingCurveState,
        tokens_in: u64,
    ) -> Result<Decimal, crate::PumpFunError> {
        // Apply constant product formula
        let k = state.virtual_sol_reserves as u128 * state.virtual_token_reserves as u128;
        let new_token_reserves = state.virtual_token_reserves as u128 + tokens_in as u128;

        if new_token_reserves == 0 {
            return Err(crate::PumpFunError::InvalidBondingCurve(
                "Invalid reserves".to_string(),
            ));
        }

        let new_sol_reserves = k / new_token_reserves;
        let sol_out_lamports =
            (state.virtual_sol_reserves as u128).saturating_sub(new_sol_reserves);

        Ok(Decimal::from(sol_out_lamports) / dec!(1_000_000_000))
    }

    /// Calculate the progress towards graduation (0-100%)
    pub fn calculate_progress(state: &BondingCurveState) -> f64 {
        // Pump.fun graduates to Raydium when bonding curve reaches ~85 SOL
        const GRADUATION_SOL_TARGET: u64 = 85_000_000_000; // 85 SOL in lamports

        let progress = (state.sol_reserve as f64 / GRADUATION_SOL_TARGET as f64) * 100.0;
        progress.min(100.0)
    }

    /// Calculate market cap in SOL
    pub fn calculate_market_cap(state: &BondingCurveState) -> Decimal {
        let price_per_token = Self::calculate_price(state);
        let total_supply = Decimal::from(state.total_supply);
        price_per_token * total_supply
    }

    /// Calculate price impact of a trade
    pub fn calculate_price_impact(
        state: &BondingCurveState,
        sol_amount: Decimal,
        is_buy: bool,
    ) -> Result<f64, crate::PumpFunError> {
        let current_price = Self::calculate_price(state);

        // Clone state to simulate the trade
        let mut simulated_state = state.clone();

        if is_buy {
            let tokens_out = Self::calculate_tokens_out(state, sol_amount)?;
            let sol_in_lamports = (sol_amount * dec!(1_000_000_000))
                .round()
                .to_u64()
                .ok_or_else(|| {
                    crate::PumpFunError::InvalidBondingCurve("Invalid SOL amount".to_string())
                })?;

            simulated_state.virtual_sol_reserves += sol_in_lamports;
            simulated_state.virtual_token_reserves -= tokens_out;
        } else {
            // For sells, convert SOL amount to tokens first
            let tokens_equivalent =
                (sol_amount / current_price)
                    .round()
                    .to_u64()
                    .ok_or_else(|| {
                        crate::PumpFunError::InvalidBondingCurve("Invalid token amount".to_string())
                    })?;

            let sol_out = Self::calculate_sol_out(state, tokens_equivalent)?;
            let sol_out_lamports = (sol_out * dec!(1_000_000_000))
                .round()
                .to_u64()
                .ok_or_else(|| {
                    crate::PumpFunError::InvalidBondingCurve("Invalid SOL amount".to_string())
                })?;

            simulated_state.virtual_sol_reserves -= sol_out_lamports;
            simulated_state.virtual_token_reserves += tokens_equivalent;
        }

        let new_price = Self::calculate_price(&simulated_state);
        let price_change = ((new_price - current_price) / current_price * dec!(100))
            .to_f64()
            .ok_or_else(|| {
                crate::PumpFunError::InvalidBondingCurve("Invalid price calculation".to_string())
            })?;

        Ok(price_change.abs())
    }

    /// Create price info from bonding curve state
    pub fn create_price_info(state: &BondingCurveState) -> PriceInfo {
        PriceInfo {
            token_mint: state.token_mint.clone(),
            price_per_token_sol: Self::calculate_price(state),
            market_cap_sol: Self::calculate_market_cap(state),
            sol_reserve: Decimal::from(state.sol_reserve) / dec!(1_000_000_000),
            token_reserve: state.token_reserve,
            virtual_sol_reserves: Decimal::from(state.virtual_sol_reserves) / dec!(1_000_000_000),
            virtual_token_reserves: state.virtual_token_reserves,
            progress_percentage: Self::calculate_progress(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> BondingCurveState {
        BondingCurveState {
            token_mint: "11111111111111111111111111111111".to_string(),
            creator: "22222222222222222222222222222222".to_string(),
            total_supply: 1_000_000_000, // 1B tokens
            sol_reserve: 10_000_000_000, // 10 SOL
            token_reserve: 900_000_000,  // 900M tokens
            graduated: false,
            created_at: 0,
            virtual_sol_reserves: 40_000_000_000, // 40 SOL (30 initial + 10 added)
            virtual_token_reserves: 900_000_000,  // 900M tokens (100M sold)
            initial_virtual_sol: 30_000_000_000,
            initial_virtual_tokens: 1_000_000_000,
        }
    }

    #[test]
    fn test_price_calculation() {
        let state = create_test_state();
        let price = BondingCurveMath::calculate_price(&state);

        // Price should be ~0.0000444 SOL per token (40 SOL / 900M tokens)
        assert!(price > dec!(0.00004) && price < dec!(0.00005));
    }

    #[test]
    fn test_tokens_out_calculation() {
        let state = create_test_state();
        let tokens_out = BondingCurveMath::calculate_tokens_out(&state, dec!(1)).unwrap();

        // Should get fewer tokens as price increases
        assert!(tokens_out > 0);
        assert!(tokens_out < 100_000_000); // Less than 100M tokens for 1 SOL
    }

    #[test]
    fn test_price_impact() {
        let state = create_test_state();

        // Test buy impact
        let buy_impact = BondingCurveMath::calculate_price_impact(&state, dec!(10), true).unwrap();
        assert!(buy_impact > 0.0); // Price should increase

        // Test sell impact
        let sell_impact =
            BondingCurveMath::calculate_price_impact(&state, dec!(10), false).unwrap();
        assert!(sell_impact > 0.0); // Price should decrease (absolute value)
    }
}
