//! AMM Integration Module
//!
//! This module handles the graduation of bonding curves to Oyl AMM pools.
//! It provides functionality to:
//! - Verify graduation criteria are met
//! - Create new AMM pools with initial liquidity
//! - Transfer bonding curve reserves to AMM
//! - Handle LP token distribution

use crate::{BaseToken, CurveParams, bonding_curve::CurveCalculator};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;


/// AMM integration handler
pub struct AMMIntegration;

impl AMMIntegration {
    /// Attempt to graduate the bonding curve to an AMM pool
    pub fn graduate_to_amm(
        context: &Context,
        token_supply: u128,
    ) -> Result<CallResponse> {
        // Check if already graduated
        if CurveCalculator::is_graduated() {
            return Err(anyhow!("Bonding curve has already graduated"));
        }

        // Get curve parameters and reserves
        let params = CurveCalculator::get_curve_params()?;
        let base_reserves = CurveCalculator::get_base_reserves();

        // Verify graduation criteria
        if !CurveCalculator::check_graduation_criteria(token_supply, base_reserves, &params) {
            return Err(anyhow!("Graduation criteria not met"));
        }

        // Calculate AMM pool ratios
        let (token_liquidity, base_liquidity) = Self::calculate_pool_ratios(
            token_supply,
            base_reserves,
            &params,
        )?;

        // Create AMM pool (this would interface with Oyl Factory)
        let pool_address = Self::create_oyl_pool(
            context,
            &params.base_token,
            token_liquidity,
            base_liquidity,
        )?;

        // Mark as graduated
        CurveCalculator::set_graduated();

        // Store pool information
        Self::set_amm_pool_address(pool_address);
        Self::set_graduation_block(0); // Placeholder - real implementation would get from context

        // Return success response
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = pool_address.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Calculate optimal token and base liquidity for AMM pool
    fn calculate_pool_ratios(
        token_supply: u128,
        base_reserves: u128,
        params: &CurveParams,
    ) -> Result<(u128, u128)> {
        // Reserve some percentage of tokens for AMM (e.g., 20%)
        let token_liquidity_percentage = 20; // 20%
        let token_liquidity = token_supply * token_liquidity_percentage / 100;

                 // Calculate corresponding base token amount using current price
        let current_price = crate::bonding_curve::CurveCalculator::price_at_supply(token_supply, params)
            .unwrap_or(params.base_price);
        
        let base_liquidity_needed = token_liquidity * current_price / 1_000_000_000; // Adjust for decimals
        
        // Ensure we have enough base reserves
        let base_liquidity = if base_liquidity_needed <= base_reserves {
            base_liquidity_needed
        } else {
            // Use all available reserves and adjust token amount proportionally
            base_reserves
        };

        Ok((token_liquidity, base_liquidity))
    }

    /// Create a new Oyl AMM pool (mock implementation)
    /// In a real implementation, this would call the Oyl Factory contract
    fn create_oyl_pool(
        _context: &Context,
        base_token: &BaseToken,
        token_liquidity: u128,
        base_liquidity: u128,
    ) -> Result<u128> {
        // Mock pool creation - in reality this would:
        // 1. Call Oyl Factory contract
        // 2. Transfer tokens to the new pool
        // 3. Receive LP tokens in return
        // 4. Return the pool address

        // For now, generate a mock pool address based on inputs
        let pool_address = Self::generate_pool_address(base_token, token_liquidity, base_liquidity);
        
        Ok(pool_address)
    }

    /// Generate a deterministic pool address (mock)
    fn generate_pool_address(
        base_token: &BaseToken,
        token_liquidity: u128,
        base_liquidity: u128,
    ) -> u128 {
        // Simple hash-like generation for demo
        let base_block = match base_token {
            BaseToken::BUSD => 2u128,
            BaseToken::FrBtc => 32u128,
        };
        let combined = base_block
            .wrapping_add(token_liquidity)
            .wrapping_add(base_liquidity);
        
        // Ensure it's in a reasonable range for Alkane IDs
        (combined % 1_000_000) + 100_000
    }

    /// Check if sufficient liquidity exists for graduation
    pub fn check_liquidity_sufficiency(
        token_supply: u128,
        base_reserves: u128,
        params: &CurveParams,
    ) -> bool {
        let (token_needed, base_needed) = match Self::calculate_pool_ratios(
            token_supply,
            base_reserves,
            params,
        ) {
            Ok(ratios) => ratios,
            Err(_) => return false,
        };

        // Minimum thresholds for meaningful liquidity
        let min_token_liquidity = 1_000_000; // 1M tokens minimum
        let min_base_liquidity = 1_000_000_000; // Minimum base tokens

        token_needed >= min_token_liquidity && base_needed >= min_base_liquidity
    }

    /// Storage functions for AMM integration state
    pub fn amm_pool_address_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/amm_pool_address")
    }

    pub fn graduation_block_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/graduation_block")
    }

    pub fn lp_tokens_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/lp_tokens")
    }

    /// Get AMM pool address if graduated
    pub fn get_amm_pool_address() -> Option<u128> {
        let addr = Self::amm_pool_address_pointer().get_value::<u128>();
        if addr == 0 {
            None
        } else {
            Some(addr)
        }
    }

    /// Set AMM pool address
    pub fn set_amm_pool_address(address: u128) {
        Self::amm_pool_address_pointer().set_value::<u128>(address);
    }

    /// Get graduation block height
    pub fn get_graduation_block() -> u64 {
        Self::graduation_block_pointer().get_value::<u64>()
    }

    /// Set graduation block height
    pub fn set_graduation_block(block: u64) {
        Self::graduation_block_pointer().set_value::<u64>(block);
    }

    /// Get LP token balance
    pub fn get_lp_tokens() -> u128 {
        Self::lp_tokens_pointer().get_value::<u128>()
    }

    /// Set LP token balance
    pub fn set_lp_tokens(amount: u128) {
        Self::lp_tokens_pointer().set_value::<u128>(amount);
    }

    /// Emergency graduation after time limit (e.g., 30 days)
    pub fn check_emergency_graduation(
        current_block: u64,
        launch_block: u64,
        token_supply: u128,
        base_reserves: u128,
    ) -> bool {
        let blocks_elapsed = current_block.saturating_sub(launch_block);
        let emergency_threshold = 30 * 24 * 6; // ~30 days at 10min blocks

        if blocks_elapsed >= emergency_threshold {
            // More lenient criteria for emergency graduation
            let min_supply = 1_000_000; // 1M tokens
            let min_reserves = 100_000_000; // Lower reserve requirement

            return token_supply >= min_supply && base_reserves >= min_reserves;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_ratio_calculation() {
        let params = CurveParams::default();
        let token_supply = 1_000_000_000; // 1B tokens
        let base_reserves = 10_000_000_000; // 10B base tokens

        let (token_liquidity, base_liquidity) = 
            AMMIntegration::calculate_pool_ratios(token_supply, base_reserves, &params).unwrap();

        assert!(token_liquidity > 0);
        assert!(base_liquidity > 0);
        assert!(token_liquidity <= token_supply);
        assert!(base_liquidity <= base_reserves);
    }

    #[test]
    fn test_liquidity_sufficiency() {
        let params = CurveParams::default();
        
        // Should be insufficient with low amounts
        assert!(!AMMIntegration::check_liquidity_sufficiency(1000, 1000, &params));
        
        // Should be sufficient with high amounts
        assert!(AMMIntegration::check_liquidity_sufficiency(
            1_000_000_000, 
            10_000_000_000, 
            &params
        ));
    }

    #[test]
    fn test_emergency_graduation() {
        let current_block = 100_000;
        let launch_block = 1000;
        let token_supply = 2_000_000;
        let base_reserves = 200_000_000;

        // Should trigger emergency graduation after sufficient time
        assert!(AMMIntegration::check_emergency_graduation(
            current_block,
            launch_block,
            token_supply,
            base_reserves
        ));

        // Should not trigger with recent launch
        assert!(!AMMIntegration::check_emergency_graduation(
            5000,
            launch_block,
            token_supply,
            base_reserves
        ));
    }
} 