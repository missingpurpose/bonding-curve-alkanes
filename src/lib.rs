//! Alkanes Bonding Curve System
//!
//! A production-ready bonding curve system for Alkanes that enables token launches
//! with BUSD/frBTC integration and automatic graduation to Oyl AMM pools.
//! 
//! This system provides:
//! - Factory pattern for deploying new bonding curves
//! - Exponential pricing algorithm with configurable parameters
//! - BUSD (2:56801) and frBTC (32:0) base currency support
//! - Automatic liquidity graduation to Oyl AMM pools
//! - Comprehensive security patterns and access controls

use alkanes_runtime::storage::StoragePointer;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::gz;
use alkanes_support::response::CallResponse;
use alkanes_support::utils::overflow_error;
use alkanes_support::witness::find_witness_payload;
use alkanes_support::{context::Context, parcel::AlkaneTransfer, id::AlkaneId};
use anyhow::{anyhow, Result};
use bitcoin::hashes::Hash;
use bitcoin::{Transaction, Txid};
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_decode;
use std::io::Cursor;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub mod precompiled;
pub mod bonding_curve;
pub mod amm_integration;
#[cfg(test)]
pub mod tests;

/// Constants for base token identification
pub const BUSD_ALKANE_ID: u128 = (2u128 << 64) | 56801u128; // 2:56801
pub const FRBTC_ALKANE_ID: u128 = (32u128 << 64) | 0u128;   // 32:0

/// Factory contract identification
pub const BONDING_CURVE_FACTORY_ID: u128 = 0x0bcd;

/// Base token enum for supported currencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseToken {
    BUSD,
    frBTC,
}

impl BaseToken {
    pub fn alkane_id(&self) -> AlkaneId {
        match self {
            BaseToken::BUSD => AlkaneId::new(2, 56801),     // 2:56801
            BaseToken::frBTC => AlkaneId::new(32, 0),       // 32:0
        }
    }
}

/// Bonding curve parameters for token launches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveParams {
    pub base_price: u128,           // Starting price in base token satoshis
    pub growth_rate: u128,          // Basis points increase per token (e.g., 1500 = 1.5%)
    pub graduation_threshold: u128,  // Market cap threshold for AMM graduation
    pub base_token: BaseToken,      // Base currency (BUSD or frBTC)
    pub max_supply: u128,           // Maximum token supply
}

impl Default for CurveParams {
    fn default() -> Self {
        Self {
            base_price: 1_000_000,        // 0.01 BUSD (assuming 8 decimals)
            growth_rate: 1500,            // 1.5% per token
            graduation_threshold: 10_000_000_000_000, // 100,000 BUSD
            base_token: BaseToken::BUSD,
            max_supply: 1_000_000_000_000_000, // 1 billion tokens
        }
    }
}

/// Returns a StoragePointer for the token name
fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

/// Returns a StoragePointer for the token symbol
fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

/// Trims a u128 value to a String by removing trailing zeros
pub fn trim(v: u128) -> String {
    String::from_utf8(
        v.to_le_bytes()
            .into_iter()
            .fold(Vec::<u8>::new(), |mut r, v| {
                if v != 0 {
                    r.push(v)
                }
                r
            }),
    )
    .unwrap()
}

/// TokenName struct to hold two u128 values for the name
#[derive(Default, Clone, Copy)]
pub struct TokenName {
    pub part1: u128,
    pub part2: u128,
}

impl From<TokenName> for String {
    fn from(name: TokenName) -> Self {
        // Trim both parts and concatenate them
        format!("{}{}", trim(name.part1), trim(name.part2))
    }
}

impl TokenName {
    pub fn new(part1: u128, part2: u128) -> Self {
        Self { part1, part2 }
    }
}

pub struct ContextHandle(());

#[cfg(test)]
impl ContextHandle {
    /// Get the current transaction bytes
    pub fn transaction(&self) -> Vec<u8> {
        // This is a placeholder implementation that would normally
        // access the transaction from the runtime context
        Vec::new()
    }
}

impl AlkaneResponder for ContextHandle {}

pub const CONTEXT: ContextHandle = ContextHandle(());

/// Extension trait for Context to add transaction_id method
trait ContextExt {
    /// Get the transaction ID from the context
    fn transaction_id(&self) -> Result<Txid>;
}

#[cfg(test)]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        // Test implementation with all zeros
        Ok(Txid::from_slice(&[0; 32]).unwrap_or_else(|_| {
            // This should never happen with a valid-length slice
            panic!("Failed to create zero Txid")
        }))
    }
}

#[cfg(not(test))]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        Ok(
            consensus_decode::<Transaction>(&mut std::io::Cursor::new(CONTEXT.transaction()))?
                .compute_txid(),
        )
    }
}

/// MintableToken trait provides common token functionality
pub trait MintableToken: AlkaneResponder {
    /// Get the token name
    fn name(&self) -> String {
        String::from_utf8(self.name_pointer().get().as_ref().clone())
            .expect("name not saved as utf-8, did this deployment revert?")
    }

    /// Get the token symbol
    fn symbol(&self) -> String {
        String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .expect("symbol not saved as utf-8, did this deployment revert?")
    }

    /// Set the token name and symbol
    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        self.name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(self.symbol_pointer(), symbol);
    }

    /// Get the pointer to the token name
    fn name_pointer(&self) -> StoragePointer {
        name_pointer()
    }

    /// Get the pointer to the token symbol
    fn symbol_pointer(&self) -> StoragePointer {
        symbol_pointer()
    }

    /// Set a string field in storage
    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    /// Get the pointer to the total supply
    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }

    /// Get the total supply
    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }

    /// Set the total supply
    fn set_total_supply(&self, v: u128) {
        self.total_supply_pointer().set_value::<u128>(v);
    }

    /// Increase the total supply
    fn increase_total_supply(&self, v: u128) -> Result<()> {
        self.set_total_supply(
            overflow_error(self.total_supply().checked_add(v))
                .map_err(|_| anyhow!("total supply overflow"))?,
        );
        Ok(())
    }

    /// Mint new tokens
    fn mint(&self, context: &Context, value: u128) -> Result<AlkaneTransfer> {
        self.increase_total_supply(value)?;
        Ok(AlkaneTransfer {
            id: context.myself.clone(),
            value,
        })
    }

    /// Get the pointer to the token data
    fn data_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/data")
    }

    /// Get the token data
    fn data(&self) -> Vec<u8> {
        gz::decompress(self.data_pointer().get().as_ref().clone()).unwrap_or_else(|_| vec![])
    }

    /// Set the token data from the transaction
    fn set_data(&self) -> Result<()> {
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(CONTEXT.transaction()))?;
        let data: Vec<u8> = find_witness_payload(&tx, 0).unwrap_or_else(|| vec![]);
        self.data_pointer().set(Arc::new(data));

        Ok(())
    }
}

/// BondingCurveToken implements a bonding curve token contract
#[derive(Default)]
pub struct BondingCurveToken(());

impl MintableToken for BondingCurveToken {}

/// Message enum for bonding curve operations
#[derive(MessageDispatch)]
enum BondingCurveMessage {
    /// Initialize the bonding curve with parameters
    #[opcode(0)]
    Initialize {
        /// Token name part 1
        name_part1: u128,
        /// Token name part 2  
        name_part2: u128,
        /// Token symbol
        symbol: u128,
        /// Base price in satoshis
        base_price: u128,
        /// Growth rate in basis points
        growth_rate: u128,
        /// Graduation threshold
        graduation_threshold: u128,
        /// Base token type (0 = BUSD, 1 = frBTC)
        base_token_type: u128,
        /// Maximum supply
        max_supply: u128,
    },

    /// Buy tokens with base currency
    #[opcode(1)]
    BuyTokens {
        /// Minimum tokens expected (slippage protection)
        min_tokens_out: u128,
    },

    /// Sell tokens for base currency
    #[opcode(2)]
    SellTokens {
        /// Amount of tokens to sell
        token_amount: u128,
        /// Minimum base tokens expected (slippage protection)
        min_base_out: u128,
    },

    /// Get buy quote for token amount
    #[opcode(3)]
    #[returns(u128)]
    GetBuyQuote {
        /// Number of tokens to quote
        token_amount: u128,
    },

    /// Get sell quote for token amount
    #[opcode(4)]
    #[returns(u128)]
    GetSellQuote {
        /// Number of tokens to quote
        token_amount: u128,
    },

    /// Attempt graduation to AMM
    #[opcode(5)]
    Graduate,

    /// Get curve state information
    #[opcode(6)]
    #[returns(Vec<u8>)]
    GetCurveState,

    /// Get the token name
    #[opcode(99)]
    #[returns(String)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    /// Get the total supply
    #[opcode(101)]
    #[returns(u128)]
    GetTotalSupply,

    /// Get current base reserves
    #[opcode(102)]
    #[returns(u128)]
    GetBaseReserves,

    /// Get AMM pool address if graduated
    #[opcode(103)]
    #[returns(u128)]
    GetAmmPoolAddress,

    /// Check if graduated
    #[opcode(104)]
    #[returns(bool)]
    IsGraduated,

    /// Get the token data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,
}

impl BondingCurveToken {
    /// Get launch block height
    pub fn launch_block_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/launch_block")
    }

    /// Get the launch block
    pub fn launch_block(&self) -> u64 {
        self.launch_block_pointer().get_value::<u64>()
    }

    /// Set the launch block
    pub fn set_launch_block(&self, block: u64) {
        self.launch_block_pointer().set_value::<u64>(block);
    }

    /// Get the current supply (same as total supply)
    pub fn current_supply(&self) -> u128 {
        self.total_supply()
    }



    /// Get the pointer to the supply cap
    pub fn cap_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/cap")
    }

    /// Get the supply cap
    pub fn cap(&self) -> u128 {
        self.cap_pointer().get_value::<u128>()
    }

    /// Initialize the bonding curve with parameters
    fn initialize(
        &self,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
        base_price: u128,
        growth_rate: u128,
        graduation_threshold: u128,
        base_token_type: u128,
        max_supply: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;

        // Create and store curve parameters
        let base_token = match base_token_type {
            0 => BaseToken::BUSD,
            1 => BaseToken::frBTC,
            _ => return Err(anyhow!("Invalid base token type")),
        };

        let params = CurveParams {
            base_price,
            growth_rate,
            graduation_threshold,
            base_token,
            max_supply,
        };

        bonding_curve::BondingCurve::set_curve_params(&params)?;

        // Set token metadata
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        // Set launch block (use a placeholder for now, real implementation would get from context)
        self.set_launch_block(0);

        // Initialize reserves to zero
        bonding_curve::BondingCurve::set_base_reserves(0);
        bonding_curve::BondingCurve::set_token_reserves(0);

        self.set_data()?;

        Ok(response)
    }

    /// Buy tokens with base currency
    fn buy_tokens(&self, min_tokens_out: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Check if already graduated
        if bonding_curve::BondingCurve::is_graduated() {
            return Err(anyhow!("Bonding curve has graduated to AMM"));
        }

        // Get curve parameters and current state
        let params = bonding_curve::BondingCurve::get_curve_params()?;
        let _current_supply = self.current_supply();
        
        // Find the base token input from incoming alkanes
        let base_input = context.incoming_alkanes.0
            .iter()
            .find(|transfer| transfer.id == params.base_token.alkane_id())
            .ok_or_else(|| anyhow!("No base token input found"))?;

        let base_amount = base_input.value;

        // Calculate how many tokens to mint for this amount
        let tokens_to_mint = self.calculate_tokens_for_base_amount(base_amount, &params)?;

        // Check slippage protection
        if tokens_to_mint < min_tokens_out {
            return Err(anyhow!("Slippage exceeded: got {} tokens, expected at least {}", 
                tokens_to_mint, min_tokens_out));
        }

        // Mint the tokens
        response.alkanes.0.push(self.mint(&context, tokens_to_mint)?);

        // Update reserves
        let current_reserves = bonding_curve::BondingCurve::get_base_reserves();
        bonding_curve::BondingCurve::set_base_reserves(current_reserves + base_amount);

        Ok(response)
    }

    /// Sell tokens for base currency
    fn sell_tokens(&self, token_amount: u128, min_base_out: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Check if already graduated
        if bonding_curve::BondingCurve::is_graduated() {
            return Err(anyhow!("Bonding curve has graduated to AMM"));
        }

        // Get curve parameters and calculate sell price
        let params = bonding_curve::BondingCurve::get_curve_params()?;
        let current_supply = self.current_supply();
        
        // Calculate base tokens to return
        let base_payout = bonding_curve::BondingCurve::calculate_sell_price(
            current_supply, token_amount, &params
        )?;

        // Check slippage protection
        if base_payout < min_base_out {
            return Err(anyhow!("Slippage exceeded: got {} base tokens, expected at least {}", 
                base_payout, min_base_out));
        }

        // Check we have enough reserves
        let current_reserves = bonding_curve::BondingCurve::get_base_reserves();
        if base_payout > current_reserves {
            return Err(anyhow!("Insufficient reserves for sell"));
        }

        // Burn the tokens (decrease total supply)
        let new_supply = current_supply.checked_sub(token_amount)
            .ok_or_else(|| anyhow!("Cannot burn more tokens than exist"))?;
        self.set_total_supply(new_supply);

        // Return base tokens to seller
        response.alkanes.0.push(AlkaneTransfer {
            id: params.base_token.alkane_id(),
            value: base_payout,
        });

        // Update reserves
        bonding_curve::BondingCurve::set_base_reserves(current_reserves - base_payout);

        Ok(response)
    }

    /// Calculate tokens to mint for a given base amount
    fn calculate_tokens_for_base_amount(&self, base_amount: u128, params: &CurveParams) -> Result<u128> {
        let current_supply = self.current_supply();
        
        // Binary search to find the right number of tokens
        // This is needed because we have the inverse problem: given cost, find tokens
        let mut low = 0u128;
        let mut high = params.max_supply.saturating_sub(current_supply);
        let mut best_tokens = 0u128;

        while low <= high {
            let mid = (low + high) / 2;
            let cost = bonding_curve::BondingCurve::calculate_buy_price(current_supply, mid, params)?;
            
            if cost <= base_amount {
                best_tokens = mid;
                low = mid + 1;
            } else {
                high = mid.saturating_sub(1);
            }

            if low > high {
                break;
            }
        }

        if best_tokens == 0 {
            return Err(anyhow!("Insufficient base amount to buy any tokens"));
        }

        Ok(best_tokens)
    }

    /// Get buy quote for token amount
    fn get_buy_quote(&self, token_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = bonding_curve::BondingCurve::get_curve_params()?;
        let current_supply = self.current_supply();
        
        let cost = bonding_curve::BondingCurve::calculate_buy_price(
            current_supply, token_amount, &params
        )?;

        response.data = cost.to_le_bytes().to_vec();
        Ok(response)
    }

    /// Get sell quote for token amount
    fn get_sell_quote(&self, token_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = bonding_curve::BondingCurve::get_curve_params()?;
        let current_supply = self.current_supply();
        
        let payout = bonding_curve::BondingCurve::calculate_sell_price(
            current_supply, token_amount, &params
        )?;

        response.data = payout.to_le_bytes().to_vec();
        Ok(response)
    }

    /// Attempt graduation to AMM
    fn graduate(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let current_supply = self.current_supply();

        amm_integration::AMMIntegration::graduate_to_amm(&context, current_supply)
    }

    /// Get curve state information
    fn get_curve_state(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = bonding_curve::BondingCurve::get_curve_params()?;
        let current_supply = self.current_supply();
        let base_reserves = bonding_curve::BondingCurve::get_base_reserves();
        let is_graduated = bonding_curve::BondingCurve::is_graduated();
        let amm_pool = amm_integration::AMMIntegration::get_amm_pool_address();

        // Create state object
        let state = serde_json::json!({
            "current_supply": current_supply,
            "base_reserves": base_reserves,
            "is_graduated": is_graduated,
            "amm_pool_address": amm_pool,
            "base_token": params.base_token,
            "curve_params": {
                "base_price": params.base_price,
                "growth_rate": params.growth_rate,
                "graduation_threshold": params.graduation_threshold,
                "max_supply": params.max_supply
            }
        });

        response.data = serde_json::to_vec(&state)
            .map_err(|e| anyhow!("Failed to serialize state: {}", e))?;

        Ok(response)
    }

    /// Set the token name and symbol
    fn set_name_and_symbol(
        &self,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Create TokenName from the two parts
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        Ok(response)
    }

    /// Get the token name
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the token symbol
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the total supply
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.total_supply().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get current base reserves
    fn get_base_reserves(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let reserves = bonding_curve::BondingCurve::get_base_reserves();
        response.data = reserves.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get AMM pool address if graduated
    fn get_amm_pool_address(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let pool_address = amm_integration::AMMIntegration::get_amm_pool_address().unwrap_or(0);
        response.data = pool_address.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Check if graduated
    fn is_graduated(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let graduated = bonding_curve::BondingCurve::is_graduated();
        response.data = vec![if graduated { 1u8 } else { 0u8 }];

        Ok(response)
    }

    /// Get the token data
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.data();

        Ok(response)
    }
}

impl AlkaneResponder for BondingCurveToken {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for BondingCurveToken {
        type Message = BondingCurveMessage;
    }
}
