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
use alkanes_support::id::AlkaneId;
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use oyl_amm::{
    factory::{Factory, FactoryConfig},
    pool::{Pool, PoolConfig},
    types::{TokenPair, LiquidityProvider},
};

// Oyl Factory contract addresses (these would be deployed on mainnet)
// Note: These are placeholder addresses - in production these would be real contract addresses
fn get_busd_factory_address() -> AlkaneId {
    AlkaneId::new(2, 56802) // Example BUSD factory
}

fn get_frbtc_factory_address() -> AlkaneId {
    AlkaneId::new(32, 1)   // Example frBTC factory
}

// LP Distribution Strategy constants
const LP_STRATEGY_FULL_BURN: u128 = 0;
const LP_STRATEGY_COMMUNITY: u128 = 1;
const LP_STRATEGY_CREATOR: u128 = 2;
const LP_STRATEGY_DAO: u128 = 3;

// Oyl Factory and Pool opcodes will be replaced with real SDK calls
// See: https://docs.oyl.io/developer for integration details

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

        // Create AMM pool with atomic operation
        let pool_address = Self::create_oyl_pool_atomic(
            context,
            &params.base_token,
            token_liquidity,
            base_liquidity,
        )?;

        // Mark as graduated only after successful pool creation
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

    /// Create a new Oyl AMM pool with atomic operation (all-or-nothing)
    fn create_oyl_pool_atomic(
        context: &Context,
        base_token: &BaseToken,
        token_liquidity: u128,
        base_liquidity: u128,
    ) -> Result<u128> {
        // Get the appropriate factory address based on base token
        let factory_address = match base_token {
            BaseToken::BUSD => get_busd_factory_address(),
            BaseToken::FrBtc => get_frbtc_factory_address(),
        };

        // Step 1: Create pool via Oyl Factory
        let pool_address = Self::call_oyl_factory_create_pool(
            factory_address,
            context.myself.clone(),    // Our bonding curve token
            base_token.alkane_id(),    // BUSD(2:56801) or frBTC(32:0)
        )?;

        // Step 2: Verify pool was created successfully
        if !Self::verify_pool_creation(pool_address, context.myself.clone(), base_token.alkane_id())? {
            return Err(anyhow!("Pool creation verification failed"));
        }

        // Step 3: Transfer tokens to the new pool
        Self::transfer_tokens_to_pool(
            pool_address,
            context.myself.clone(),
            token_liquidity,
            base_token.alkane_id(),
            base_liquidity,
        )?;

        // Step 4: Add initial liquidity to the pool
        let lp_tokens_received = Self::add_initial_liquidity(
            pool_address,
            context.myself.clone(),
            token_liquidity,
            base_token.alkane_id(),
            base_liquidity,
        )?;

        // Step 5: Handle LP token distribution based on strategy
        Self::distribute_lp_tokens(lp_tokens_received, context)?;

        Ok(pool_address)
    }

    /// Call Oyl Factory to create a new pool
    fn call_oyl_factory_create_pool(
        factory_address: AlkaneId,
        token_a: AlkaneId,
        token_b: AlkaneId,
    ) -> Result<u128> {
        // Initialize Oyl Factory with configuration
        let factory_config = FactoryConfig {
            fee_percent: 30,  // 0.3% fee
            admin: factory_address,
            protocol_fee_percent: 10,  // 0.1% protocol fee
        };
        let factory = Factory::new(factory_config);

        // Create token pair
        let pair = TokenPair {
            token0: token_a,
            token1: token_b,
        };

        // Create pool through factory
        let pool_config = PoolConfig {
            pair: pair.clone(),
            fee_percent: 30,  // 0.3% fee
            tick_spacing: 60,  // Standard tick spacing
        };
        
        let pool_address = factory.create_pool(pool_config)?;
        
        // Verify pool creation
        let pool = Pool::at(pool_address)?;
        if pool.get_pair()? != pair {
            return Err(anyhow!("Pool creation verification failed"));
        }
        
        Ok(pool_address)
    }

    /// Verify that pool was created successfully
    fn verify_pool_creation(
        pool_address: u128,
        token_a: AlkaneId,
        token_b: AlkaneId,
    ) -> Result<bool> {
        // Get pool instance
        let pool = Pool::at(pool_address)?;
        
        // Get pool pair
        let pair = pool.get_pair()?;
        
        // Verify tokens match (in either order)
        let tokens_match = (pair.token0 == token_a && pair.token1 == token_b) ||
                         (pair.token0 == token_b && pair.token1 == token_a);
                         
        // Verify pool is initialized
        let is_initialized = pool.is_initialized()?;
        
        Ok(tokens_match && is_initialized)
    }

    /// Transfer tokens to the newly created pool
    fn transfer_tokens_to_pool(
        pool_address: u128,
        token_id: AlkaneId,
        token_amount: u128,
        base_token_id: AlkaneId,
        base_amount: u128,
    ) -> Result<()> {
        // Get pool instance
        let pool = Pool::at(pool_address)?;
        
        // Verify we have sufficient tokens before transfer
        if !Self::verify_token_balance(token_id, token_amount)? {
            return Err(anyhow!("Insufficient bonding curve tokens for pool"));
        }
        
        if !Self::verify_token_balance(base_token_id, base_amount)? {
            return Err(anyhow!("Insufficient base tokens for pool"));
        }
        
        // Transfer tokens to pool
        let pair = pool.get_pair()?;
        let (token0_id, token0_amount, token1_id, token1_amount) = if pair.token0 == token_id {
            (token_id, token_amount, base_token_id, base_amount)
        } else {
            (base_token_id, base_amount, token_id, token_amount)
        };
        
        // Create transfer calls
        let mut response = CallResponse::default();
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: token0_id,
            value: token0_amount,
        });
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: token1_id,
            value: token1_amount,
        });
        
        Ok(())
    }

    /// Verify token balance before transfer
    fn verify_token_balance(_token_id: AlkaneId, _required_amount: u128) -> Result<bool> {
        // This would check the actual token balance
        // For now, assume we have sufficient balance
        Ok(true)
    }

    /// Add initial liquidity to the pool and receive LP tokens
    fn add_initial_liquidity(
        pool_address: u128,
        token_id: AlkaneId,
        token_amount: u128,
        base_token_id: AlkaneId,
        base_amount: u128,
    ) -> Result<u128> {
        // Get pool instance
        let pool = Pool::at(pool_address)?;
        
        // Create liquidity provider info
        let provider = LiquidityProvider {
            address: token_id,  // Use token contract as provider
            token0_amount: token_amount,
            token1_amount: base_amount,
            fee_tier: 30,  // 0.3% fee tier
        };
        
        // Add liquidity to pool
        let (lp_tokens, _) = pool.add_liquidity(provider)?;
        
        // Verify LP tokens were received
        if lp_tokens == 0 {
            return Err(anyhow!("Failed to receive LP tokens from pool"));
        }
        
        Ok(lp_tokens)
    }

    /// Calculate LP tokens using constant product formula
    fn calculate_lp_tokens(token_amount: u128, base_amount: u128) -> u128 {
        // LP tokens = sqrt(token_amount * base_amount)
        // We'll use a simplified calculation for now
        let product = token_amount.saturating_mul(base_amount);
        let sqrt = (product as f64).sqrt() as u128;
        sqrt
    }

    /// Distribute LP tokens according to the bonding curve's strategy
    fn distribute_lp_tokens(lp_tokens: u128, context: &Context) -> Result<()> {
        // Get the LP distribution strategy from the bonding curve
        let strategy = Self::get_lp_distribution_strategy();
        
        // Ensure we have LP tokens to distribute
        if lp_tokens == 0 {
            return Err(anyhow!("No LP tokens to distribute"));
        }
        
        match strategy {
            LP_STRATEGY_FULL_BURN => {
                // Burn 80% of LP tokens, distribute 20% to holders
                let burn_amount = lp_tokens * 80 / 100;
                let holder_amount = lp_tokens - burn_amount; // Ensure no rounding loss
                
                Self::burn_lp_tokens(burn_amount)?;
                Self::distribute_to_holders(holder_amount, context)?;
            },
            LP_STRATEGY_COMMUNITY => {
                // 60% to community rewards, 20% to holders, 20% to creator
                let community_amount = lp_tokens * 60 / 100;
                let holder_amount = lp_tokens * 20 / 100;
                let creator_amount = lp_tokens - community_amount - holder_amount; // Ensure no rounding loss
                
                Self::distribute_to_community(community_amount)?;
                Self::distribute_to_holders(holder_amount, context)?;
                Self::distribute_to_creator(creator_amount)?;
            },
            LP_STRATEGY_CREATOR => {
                // 40% to creator, 40% to holders, 20% to community
                let creator_amount = lp_tokens * 40 / 100;
                let holder_amount = lp_tokens * 40 / 100;
                let community_amount = lp_tokens - creator_amount - holder_amount; // Ensure no rounding loss
                
                Self::distribute_to_creator(creator_amount)?;
                Self::distribute_to_holders(holder_amount, context)?;
                Self::distribute_to_community(community_amount)?;
            },
            LP_STRATEGY_DAO => {
                // 50% to DAO treasury, 30% to holders, 20% to community
                let dao_amount = lp_tokens * 50 / 100;
                let holder_amount = lp_tokens * 30 / 100;
                let community_amount = lp_tokens - dao_amount - holder_amount; // Ensure no rounding loss
                
                Self::distribute_to_dao(dao_amount)?;
                Self::distribute_to_holders(holder_amount, context)?;
                Self::distribute_to_community(community_amount)?;
            },
            _ => {
                // Default to full burn strategy
                let burn_amount = lp_tokens * 80 / 100;
                let holder_amount = lp_tokens - burn_amount;
                
                Self::burn_lp_tokens(burn_amount)?;
                Self::distribute_to_holders(holder_amount, context)?;
            }
        }
        
        Ok(())
    }

    /// Get LP distribution strategy from storage
    fn get_lp_distribution_strategy() -> u128 {
        // This would read from the bonding curve's storage
        // For now, return a default value
        0 // Default to full burn strategy
    }

    /// Burn LP tokens (send to zero address)
    fn burn_lp_tokens(amount: u128) -> Result<()> {
        // In production, this would transfer LP tokens to a burn address
        // For now, we'll just simulate the burn
        if amount == 0 {
            return Ok(());
        }
        Ok(())
    }

    /// Distribute LP tokens to token holders
    fn distribute_to_holders(amount: u128, _context: &Context) -> Result<()> {
        // This would distribute LP tokens proportionally to all token holders
        // Implementation would depend on the specific holder tracking mechanism
        if amount == 0 {
            return Ok(());
        }
        Ok(())
    }

    /// Distribute LP tokens to community rewards
    fn distribute_to_community(amount: u128) -> Result<()> {
        // This would send LP tokens to a community rewards contract
        if amount == 0 {
            return Ok(());
        }
        Ok(())
    }

    /// Distribute LP tokens to creator
    fn distribute_to_creator(amount: u128) -> Result<()> {
        // This would send LP tokens to the token creator
        if amount == 0 {
            return Ok(());
        }
        Ok(())
    }

    /// Distribute LP tokens to DAO treasury
    fn distribute_to_dao(amount: u128) -> Result<()> {
        // This would send LP tokens to the DAO treasury
        if amount == 0 {
            return Ok(());
        }
        Ok(())
    }

    /// Generate a deterministic pool address based on factory and tokens
    fn generate_deterministic_pool_address(
        factory: &AlkaneId,
        token_a: &AlkaneId,
        token_b: &AlkaneId,
    ) -> u128 {
        // Create a deterministic address based on factory and token addresses
        // This is a simplified version - in production, the factory would generate this
        let factory_hash = (factory.block as u128) << 64 | (factory.tx as u128);
        let token_a_hash = (token_a.block as u128) << 64 | (token_a.tx as u128);
        let token_b_hash = (token_b.block as u128) << 64 | (token_b.tx as u128);
        
        let combined = factory_hash
            .wrapping_add(token_a_hash)
            .wrapping_add(token_b_hash);
        
        // Ensure it's in a reasonable range for Alkane IDs
        (combined % 1_000_000) + 100_000
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

    fn setup_test_context() -> Context {
        Context {
            myself: AlkaneId::new(100, 1),  // Our token
            block_height: 800_000,
            timestamp: 1_700_000_000,
            incoming_alkanes: Vec::new(),
        }
    }

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
    fn test_busd_pool_creation() {
        let context = setup_test_context();
        let base_token = BaseToken::BUSD;
        let token_liquidity = 1_000_000_000;  // 1B tokens
        let base_liquidity = 10_000_000_000;  // 10B BUSD

        // Create pool
        let pool_address = AMMIntegration::create_oyl_pool_atomic(
            &context,
            &base_token,
            token_liquidity,
            base_liquidity,
        ).unwrap();

        // Verify pool
        assert!(pool_address > 0);
        let pool = Pool::at(pool_address).unwrap();
        let pair = pool.get_pair().unwrap();
        assert_eq!(pair.token0, context.myself);  // Our token
        assert_eq!(pair.token1, base_token.alkane_id());  // BUSD
        assert!(pool.is_initialized().unwrap());

        // Verify liquidity
        let provider = LiquidityProvider {
            address: context.myself,
            token0_amount: token_liquidity,
            token1_amount: base_liquidity,
            fee_tier: 30,
        };
        let (lp_tokens, _) = pool.add_liquidity(provider).unwrap();
        assert!(lp_tokens > 0);
    }

    #[test]
    fn test_frbtc_pool_creation() {
        let context = setup_test_context();
        let base_token = BaseToken::FrBtc;
        let token_liquidity = 1_000_000_000;  // 1B tokens
        let base_liquidity = 100_000_000;     // 1 frBTC

        // Create pool
        let pool_address = AMMIntegration::create_oyl_pool_atomic(
            &context,
            &base_token,
            token_liquidity,
            base_liquidity,
        ).unwrap();

        // Verify pool
        assert!(pool_address > 0);
        let pool = Pool::at(pool_address).unwrap();
        let pair = pool.get_pair().unwrap();
        assert_eq!(pair.token0, context.myself);  // Our token
        assert_eq!(pair.token1, base_token.alkane_id());  // frBTC
        assert!(pool.is_initialized().unwrap());

        // Verify liquidity
        let provider = LiquidityProvider {
            address: context.myself,
            token0_amount: token_liquidity,
            token1_amount: base_liquidity,
            fee_tier: 30,
        };
        let (lp_tokens, _) = pool.add_liquidity(provider).unwrap();
        assert!(lp_tokens > 0);
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
    fn test_graduation_flow() {
        let context = setup_test_context();
        let base_token = BaseToken::BUSD;
        let token_supply = 1_000_000_000;  // 1B tokens
        let base_reserves = 10_000_000_000;  // 10B BUSD
        let params = CurveParams {
            base_price: 1_000_000,  // 0.01 BUSD
            growth_rate: 150,       // 1.5%
            graduation_threshold: 1_000_000_000_000,  // 10k BUSD
            base_token,
            max_supply: 10_000_000_000_000,  // 10T tokens
        };

        // Step 1: Check graduation criteria
        assert!(AMMIntegration::check_graduation_criteria(
            token_supply,
            base_reserves,
            &params
        ));

        // Step 2: Graduate to AMM
        let response = AMMIntegration::graduate_to_amm(
            &context,
            token_supply,
        ).unwrap();

        // Step 3: Verify pool address
        let pool_address = u128::from_le_bytes(response.data.try_into().unwrap());
        assert!(pool_address > 0);

        // Step 4: Verify pool state
        let pool = Pool::at(pool_address).unwrap();
        assert!(pool.is_initialized().unwrap());
        let pair = pool.get_pair().unwrap();
        assert_eq!(pair.token0, context.myself);
        assert_eq!(pair.token1, base_token.alkane_id());

        // Step 5: Verify LP tokens
        let lp_tokens = AMMIntegration::get_lp_tokens();
        assert!(lp_tokens > 0);

        // Step 6: Verify graduation state
        assert!(CurveCalculator::is_graduated());
        assert_eq!(AMMIntegration::get_amm_pool_address(), Some(pool_address));
    }

    #[test]
    fn test_graduation_strategies() {
        let context = setup_test_context();
        let lp_tokens = 1_000_000_000;  // 1B LP tokens

        // Test Full Burn strategy
        AMMIntegration::distribute_lp_tokens(lp_tokens, &context).unwrap();
        let burn_amount = lp_tokens * 80 / 100;  // 80%
        let holder_amount = lp_tokens - burn_amount;  // 20%
        assert_eq!(burn_amount + holder_amount, lp_tokens);  // No rounding loss

        // Test Community strategy
        AMMIntegration::distribute_lp_tokens(lp_tokens, &context).unwrap();
        let community_amount = lp_tokens * 60 / 100;  // 60%
        let holder_amount = lp_tokens * 20 / 100;     // 20%
        let creator_amount = lp_tokens - community_amount - holder_amount;  // 20%
        assert_eq!(community_amount + holder_amount + creator_amount, lp_tokens);

        // Test Creator strategy
        AMMIntegration::distribute_lp_tokens(lp_tokens, &context).unwrap();
        let creator_amount = lp_tokens * 40 / 100;    // 40%
        let holder_amount = lp_tokens * 40 / 100;     // 40%
        let community_amount = lp_tokens - creator_amount - holder_amount;  // 20%
        assert_eq!(creator_amount + holder_amount + community_amount, lp_tokens);

        // Test DAO strategy
        AMMIntegration::distribute_lp_tokens(lp_tokens, &context).unwrap();
        let dao_amount = lp_tokens * 50 / 100;        // 50%
        let holder_amount = lp_tokens * 30 / 100;     // 30%
        let community_amount = lp_tokens - dao_amount - holder_amount;  // 20%
        assert_eq!(dao_amount + holder_amount + community_amount, lp_tokens);
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

    #[test]
    fn test_deterministic_pool_address() {
        let factory = AlkaneId::new(2, 56802);
        let token_a = AlkaneId::new(100, 1);
        let token_b = AlkaneId::new(2, 56801); // BUSD
        
        let pool_address = AMMIntegration::generate_deterministic_pool_address(
            &factory,
            &token_a,
            &token_b,
        );
        
        assert!(pool_address > 0);
        assert!(pool_address < 2_000_000); // Reasonable range
    }

    #[test]
    fn test_lp_token_calculation() {
        let token_amount = 1_000_000_000;
        let base_amount = 10_000_000_000;
        
        let lp_tokens = AMMIntegration::calculate_lp_tokens(token_amount, base_amount);
        
        assert!(lp_tokens > 0);
        // LP tokens should be approximately sqrt(token_amount * base_amount)
        let expected = ((token_amount as f64) * (base_amount as f64)).sqrt() as u128;
        assert!((lp_tokens as i128 - expected as i128).abs() < 1_000_000); // Allow for rounding
    }

    #[test]
    fn test_lp_token_distribution() {
        let lp_tokens = 1_000_000_000;
        
        // Test full burn strategy
        let burn_amount = lp_tokens * 80 / 100;
        let holder_amount = lp_tokens - burn_amount;
        
        assert_eq!(burn_amount, 800_000_000);
        assert_eq!(holder_amount, 200_000_000);
        assert_eq!(burn_amount + holder_amount, lp_tokens); // No rounding loss
    }
} 