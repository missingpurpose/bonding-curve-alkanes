//! Bonding Curve Implementation
//! 
//! This module contains the core bonding curve logic including:
//! - Exponential pricing algorithm
//! - Buy/sell mechanisms with slippage protection
//! - Base token integration (BUSD/frBTC)
//! - Reserve management and graduation criteria

use crate::CurveParams;
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::utils::overflow_error;
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use std::sync::Arc;

/// Fixed-point precision constants
const PRECISION: u128 = 1_000_000_000; // 9 decimal places for precision
const BASIS_POINTS: u128 = 10_000;     // 100% = 10,000 basis points
const MAX_PRICE: u128 = u128::MAX / 1_000_000; // Prevent overflow in calculations

/// Bonding curve state management
pub struct CurveCalculator;

impl CurveCalculator {
    /// Calculate the buy price for a given number of tokens
    /// Uses TRUE exponential bonding curve: price = base_price * (1 + growth_rate/10000)^supply
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

        // For small amounts or early supply, use precise token-by-token calculation
        if tokens_to_buy <= 100 || current_supply < 1000 {
            return Self::calculate_precise_buy_cost(current_supply, tokens_to_buy, params);
        }

        // For larger amounts, use optimized integral approximation
        let start_price = Self::price_at_supply_fixed_point(current_supply, params)?;
        let end_price = Self::price_at_supply_fixed_point(new_supply - 1, params)?;
        
        // Trapezoidal rule for integral approximation
        let average_price = (start_price + end_price) / 2;
        let total_cost = overflow_error(average_price.checked_mul(tokens_to_buy))?;
        
        Ok(total_cost)
    }

    /// Precise calculation for small token amounts using summation
    fn calculate_precise_buy_cost(
        current_supply: u128,
        tokens_to_buy: u128,
        params: &CurveParams,
    ) -> Result<u128> {
        let mut total_cost = 0u128;
        
        // Calculate price for each token individually for maximum precision
        for i in 0..tokens_to_buy {
            let supply_at_token = current_supply + i;
            let price = Self::price_at_supply_fixed_point(supply_at_token, params)?;
            total_cost = overflow_error(total_cost.checked_add(price))?;
        }
        
        Ok(total_cost)
    }

    /// Calculate the sell price for a given number of tokens with 2% discount
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
        
        // Calculate theoretical return value
        let theoretical_return = if tokens_to_sell <= 100 || new_supply < 1000 {
            Self::calculate_precise_sell_return(new_supply, tokens_to_sell, params)?
        } else {
            // Use integral approximation for large amounts
            let start_price = Self::price_at_supply_fixed_point(new_supply, params)?;
            let end_price = Self::price_at_supply_fixed_point(current_supply - 1, params)?;
            let average_price = (start_price + end_price) / 2;
            overflow_error(average_price.checked_mul(tokens_to_sell))?
        };
        
        // Apply 2% discount to incentivize holding and provide liquidity buffer
        let discounted_return = theoretical_return * 98 / 100;
        
        Ok(discounted_return)
    }

    /// Precise calculation for small sell amounts
    fn calculate_precise_sell_return(
        new_supply: u128,
        tokens_to_sell: u128,
        params: &CurveParams,
    ) -> Result<u128> {
        let mut total_return = 0u128;
        
        for i in 0..tokens_to_sell {
            let supply_at_token = new_supply + i;
            let price = Self::price_at_supply_fixed_point(supply_at_token, params)?;
            total_return = overflow_error(total_return.checked_add(price))?;
        }
        
        Ok(total_return)
    }

    /// Calculate the price at a specific supply level using fixed-point math
    pub fn price_at_supply(supply: u128, params: &CurveParams) -> Result<u128> {
        Self::price_at_supply_fixed_point(supply, params)
    }

    /// Calculate price using high-precision fixed-point exponential
    fn price_at_supply_fixed_point(supply: u128, params: &CurveParams) -> Result<u128> {
        if supply == 0 {
            return Ok(params.base_price);
        }

        // Convert growth rate from basis points to fixed-point multiplier
        // e.g., 150 bps = 1.015 = (10000 + 150) / 10000
        let growth_multiplier = BASIS_POINTS + params.growth_rate;
        
        // Use optimized binary exponentiation for (growth_multiplier/BASIS_POINTS)^supply
        let price_multiplier = Self::fixed_point_power(
            growth_multiplier,
            supply,
            BASIS_POINTS,
        )?;
        
        // Apply multiplier to base price with precision scaling
        let price = overflow_error(
            params.base_price
                .checked_mul(price_multiplier)
                .ok_or_else(|| anyhow!("Overflow in price calculation"))?
                .checked_div(PRECISION)
        )?;
        
        // Cap at maximum to prevent overflow in subsequent calculations
        Ok(price.min(MAX_PRICE))
    }

    /// Optimized fixed-point power calculation using binary exponentiation
    /// Returns (base/denominator)^exponent * PRECISION for high precision
    fn fixed_point_power(
        base: u128,
        exponent: u128,
        denominator: u128,
    ) -> Result<u128> {
        if exponent == 0 {
            return Ok(PRECISION);
        }

        let mut result = PRECISION;
        let mut base_power = base * PRECISION / denominator;
        let mut exp = exponent;

        // Binary exponentiation: O(log n) instead of O(n)
        while exp > 0 {
            if exp & 1 == 1 {
                // If bit is set, multiply result by current base power
                result = overflow_error(result.checked_mul(base_power))? / PRECISION;
            }
            
            if exp > 1 {
                // Square the base power for next bit
                base_power = overflow_error(base_power.checked_mul(base_power))? / PRECISION;
            }
            
            exp >>= 1;
            
            // Prevent overflow by capping intermediate results
            if result > MAX_PRICE || base_power > MAX_PRICE {
                return Ok(MAX_PRICE);
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
        // Calculate current market cap with precision scaling
        let current_price = Self::price_at_supply_fixed_point(current_supply, params).unwrap_or(0);
        let market_cap = current_supply.saturating_mul(current_price) / PRECISION;
        
        // Primary criteria: Market cap exceeds threshold
        if market_cap >= params.graduation_threshold {
            return true;
        }

        // Secondary criteria: Sufficient liquidity (50% of threshold)
        let min_reserves = params.graduation_threshold / 2;
        if base_reserves >= min_reserves {
            return true;
        }

        // Tertiary criteria: High trading volume (inferred from reserves relative to supply)
        // If reserves are > 30% of theoretical market cap, indicates strong trading
        let theoretical_value = market_cap * 30 / 100;
        if base_reserves >= theoretical_value && current_supply >= params.max_supply / 20 {
            return true;
        }

        false
    }

    /// Calculate optimal AMM liquidity ratios for graduation
    pub fn calculate_amm_liquidity(
        current_supply: u128,
        base_reserves: u128,
        params: &CurveParams,
    ) -> Result<(u128, u128)> {
        // Use 80% of reserves for liquidity (keep 20% as buffer)
        let base_liquidity = base_reserves * 80 / 100;
        
        // Calculate token amount to match current price ratio
        let current_price = Self::price_at_supply_fixed_point(current_supply, params)?;
        
        // tokens_needed = base_liquidity / current_price (with precision adjustment)
        let tokens_needed = overflow_error(
            base_liquidity
                .checked_mul(PRECISION)
                .ok_or_else(|| anyhow!("Overflow in AMM liquidity calculation"))?
                .checked_div(current_price)
        )?;
        
        // Cap at 20% of current supply to maintain scarcity
        let token_liquidity = tokens_needed.min(current_supply * 20 / 100);
        
        Ok((token_liquidity, base_liquidity))
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