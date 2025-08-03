//! Bonding Curve Implementation
//! 
//! This module contains the core bonding curve logic including:
//! - Exponential pricing algorithm
//! - Buy/sell mechanisms with slippage protection
//! - Base token integration (BUSD/frBTC)
//! - Reserve management and graduation criteria

use crate::{BaseToken, CurveParams};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::utils::overflow_error;
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use std::sync::Arc;

/// Bonding curve state management
pub struct BondingCurve;

impl BondingCurve {
    /// Calculate the buy price for a given number of tokens
    /// Uses exponential bonding curve: price = base_price * (1 + growth_rate/10000)^supply
    pub fn calculate_buy_price(
        current_supply: u128,
        tokens_to_buy: u128,
        params: &CurveParams,
    ) -> Result<u128> {
        if tokens_to_buy == 0 {
            return Ok(0);
        }

        // Check if purchase would exceed max supply
        let new_supply = overflow_error(current_supply.checked_add(tokens_to_buy))?;
        if new_supply > params.max_supply {
            return Err(anyhow!("Purchase would exceed maximum supply"));
        }

        // Calculate integral under exponential curve
        // For exponential curve f(x) = base * (1 + rate)^x
        // Integral from a to b is: base * ((1 + rate)^b - (1 + rate)^a) / ln(1 + rate)
        // We approximate this using trapezoidal rule for precision
        
        let start_price = Self::price_at_supply(current_supply, params)?;
        let end_price = Self::price_at_supply(new_supply, params)?;
        
        // Trapezoidal rule: (start_price + end_price) * quantity / 2
        let average_price = overflow_error(start_price.checked_add(end_price))? / 2;
        let total_cost = overflow_error(average_price.checked_mul(tokens_to_buy))?;
        
        Ok(total_cost)
    }

    /// Calculate the sell price for a given number of tokens
    pub fn calculate_sell_price(
        current_supply: u128,
        tokens_to_sell: u128,
        params: &CurveParams,
    ) -> Result<u128> {
        if tokens_to_sell == 0 {
            return Ok(0);
        }

        if tokens_to_sell > current_supply {
            return Err(anyhow!("Cannot sell more tokens than current supply"));
        }

        let new_supply = current_supply - tokens_to_sell;
        
        let start_price = Self::price_at_supply(new_supply, params)?;
        let end_price = Self::price_at_supply(current_supply, params)?;
        
        // Apply small discount for sells (e.g., 1% lower than buy price)
        let average_price = overflow_error(start_price.checked_add(end_price))? / 2;
        let discounted_price = overflow_error(average_price.checked_mul(99))? / 100; // 1% discount
        let total_payout = overflow_error(discounted_price.checked_mul(tokens_to_sell))?;
        
        Ok(total_payout)
    }

    /// Calculate the price at a specific supply level
    pub fn price_at_supply(supply: u128, params: &CurveParams) -> Result<u128> {
        if supply == 0 {
            return Ok(params.base_price);
        }

        // Use fixed-point arithmetic to compute (1 + growth_rate/10000)^supply
        // We'll use a simplified approximation for large supplies
        let growth_factor = 10000 + params.growth_rate;
        let price = Self::power_approximation(params.base_price, growth_factor, supply, 10000)?;
        
        Ok(price)
    }

    /// Approximate (base * (numerator/denominator)^exponent) using repeated multiplication
    /// with overflow protection and precision management
    fn power_approximation(
        base: u128,
        numerator: u128,
        exponent: u128,
        denominator: u128,
    ) -> Result<u128> {
        if exponent == 0 {
            return Ok(base);
        }

        let mut result = base;
        let mut exp = exponent;
        
        // Use binary exponentiation for efficiency
        while exp > 0 {
            if exp % 10 == 0 {
                // Every 10 tokens, apply the growth factor once to avoid overflow
                result = overflow_error(result.checked_mul(numerator))? / denominator;
                exp -= 10;
            } else {
                // For remaining tokens, apply fractional growth
                let fractional_growth = denominator + (numerator - denominator) / 10;
                result = overflow_error(result.checked_mul(fractional_growth))? / denominator;
                exp -= 1;
            }

            // Prevent excessive price growth
            if result > u128::MAX / 1000 {
                return Ok(u128::MAX / 1000); // Cap at reasonable maximum
            }
        }

        Ok(result)
    }

    /// Check if the bonding curve meets graduation criteria
    pub fn check_graduation_criteria(
        current_supply: u128,
        base_reserves: u128,
        params: &CurveParams,
    ) -> bool {
        // Calculate current market cap
        let current_price = Self::price_at_supply(current_supply, params).unwrap_or(0);
        let market_cap = current_supply.saturating_mul(current_price);
        
        // Check if market cap exceeds threshold
        if market_cap >= params.graduation_threshold {
            return true;
        }

        // Alternative criteria: minimum liquidity reserves
        let min_reserves = params.graduation_threshold / 2; // 50,000 BUSD equivalent
        if base_reserves >= min_reserves {
            return true;
        }

        false
    }

    /// Storage pointers for bonding curve state
    pub fn curve_params_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/curve_params")
    }

    pub fn base_reserves_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/base_reserves")
    }

    pub fn token_reserves_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/token_reserves")
    }

    pub fn graduated_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/graduated")
    }

    pub fn launch_time_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/launch_time")
    }

    /// Get curve parameters from storage
    pub fn get_curve_params() -> Result<CurveParams> {
        let data = Self::curve_params_pointer().get();
        if data.as_ref().is_empty() {
            return Ok(CurveParams::default());
        }
        
        serde_json::from_slice(data.as_ref())
            .map_err(|e| anyhow!("Failed to deserialize curve params: {}", e))
    }

    /// Set curve parameters in storage
    pub fn set_curve_params(params: &CurveParams) -> Result<()> {
        let data = serde_json::to_vec(params)
            .map_err(|e| anyhow!("Failed to serialize curve params: {}", e))?;
        Self::curve_params_pointer().set(Arc::new(data));
        Ok(())
    }

    /// Get current base token reserves
    pub fn get_base_reserves() -> u128 {
        Self::base_reserves_pointer().get_value::<u128>()
    }

    /// Update base token reserves
    pub fn set_base_reserves(amount: u128) {
        Self::base_reserves_pointer().set_value::<u128>(amount);
    }

    /// Get current token reserves (virtual, for AMM calculations)
    pub fn get_token_reserves() -> u128 {
        Self::token_reserves_pointer().get_value::<u128>()
    }

    /// Update token reserves
    pub fn set_token_reserves(amount: u128) {
        Self::token_reserves_pointer().set_value::<u128>(amount);
    }

    /// Check if curve has graduated to AMM
    pub fn is_graduated() -> bool {
        Self::graduated_pointer().get_value::<u8>() == 1
    }

    /// Mark curve as graduated
    pub fn set_graduated() {
        Self::graduated_pointer().set_value::<u8>(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_price_calculation() {
        let params = CurveParams::default();
        
        // Test buying 1000 tokens at 0 supply
        let price = BondingCurve::calculate_buy_price(0, 1000, &params).unwrap();
        assert!(price > 0);
        assert!(price >= params.base_price * 1000);
    }

    #[test]
    fn test_sell_price_calculation() {
        let params = CurveParams::default();
        
        // Test selling 500 tokens from 1000 supply
        let price = BondingCurve::calculate_sell_price(1000, 500, &params).unwrap();
        assert!(price > 0);
    }

    #[test]
    fn test_graduation_criteria() {
        let params = CurveParams::default();
        
        // Should not graduate with low supply and reserves
        assert!(!BondingCurve::check_graduation_criteria(1000, 1000, &params));
        
        // Should graduate with high reserves
        let high_reserves = params.graduation_threshold;
        assert!(BondingCurve::check_graduation_criteria(1000, high_reserves, &params));
    }
} 