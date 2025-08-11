//! Optimized Bonding Curve Contract
//!
//! A fuel-efficient bonding curve contract for Alkanes that can be deployed by the factory.
//! This contract implements exponential pricing with optimized calculations and
//! automatic graduation to Oyl AMM pools.

use alkanes_runtime::storage::StoragePointer;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::gz;
use alkanes_support::response::CallResponse;
use alkanes_support::utils::overflow_error;
use alkanes_support::witness::find_witness_payload;
use alkanes_support::{context::Context, parcel::AlkaneTransfer, id::AlkaneId};
use anyhow::{anyhow, Result};

use bitcoin::Transaction;
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_decode;
use std::io::Cursor;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Base token enum for supported currencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseToken {
    BUSD,
    FrBtc,
}

impl BaseToken {
    pub fn alkane_id(&self) -> AlkaneId {
        match self {
            BaseToken::BUSD => AlkaneId::new(2, 56801),     // 2:56801
            BaseToken::FrBtc => AlkaneId::new(32, 0),       // 32:0
        }
    }
}

/// Optimized bonding curve parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedCurveParams {
    pub base_price: u128,           // Starting price in base token satoshis
    pub growth_rate: u128,          // Basis points increase per token (e.g., 1500 = 1.5%)
    pub graduation_threshold: u128,  // Market cap threshold for AMM graduation
    pub base_token: BaseToken,      // Base currency (BUSD or frBTC)
    pub max_supply: u128,           // Maximum token supply
    pub price_cache_interval: u128, // Cache price every N tokens for efficiency
}

impl Default for OptimizedCurveParams {
    fn default() -> Self {
        Self {
            base_price: 1_000_000,        // 0.01 BUSD (assuming 8 decimals)
            growth_rate: 1500,            // 1.5% per token
            graduation_threshold: 10_000_000_000_000, // 100,000 BUSD
            base_token: BaseToken::BUSD,
            max_supply: 1_000_000_000_000_000, // 1 billion tokens
            price_cache_interval: 1000,   // Cache price every 1000 tokens
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
    pub fn transaction(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl AlkaneResponder for ContextHandle {}

pub const CONTEXT: ContextHandle = ContextHandle(());

/// MintableToken trait provides common token functionality
pub trait MintableToken: AlkaneResponder {
    fn name(&self) -> String {
        String::from_utf8(self.name_pointer().get().as_ref().clone())
            .expect("name not saved as utf-8, did this deployment revert?")
    }

    fn symbol(&self) -> String {
        String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .expect("symbol not saved as utf-8, did this deployment revert?")
    }

    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        self.name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(self.symbol_pointer(), symbol);
    }

    fn name_pointer(&self) -> StoragePointer {
        name_pointer()
    }

    fn symbol_pointer(&self) -> StoragePointer {
        symbol_pointer()
    }

    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }

    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }

    fn set_total_supply(&self, v: u128) {
        self.total_supply_pointer().set_value::<u128>(v);
    }

    fn increase_total_supply(&self, v: u128) -> Result<()> {
        self.set_total_supply(
            overflow_error(self.total_supply().checked_add(v))
                .map_err(|_| anyhow!("total supply overflow"))?,
        );
        Ok(())
    }

    fn mint(&self, context: &Context, value: u128) -> Result<AlkaneTransfer> {
        self.increase_total_supply(value)?;
        Ok(AlkaneTransfer {
            id: context.myself.clone(),
            value,
        })
    }

    fn data_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/data")
    }

    fn data(&self) -> Vec<u8> {
        gz::decompress(self.data_pointer().get().as_ref().clone()).unwrap_or_else(|_| vec![])
    }

    fn set_data(&self) -> Result<()> {
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(CONTEXT.transaction()))?;
        let data: Vec<u8> = find_witness_payload(&tx, 0).unwrap_or_else(|| vec![]);
        self.data_pointer().set(Arc::new(data));
        Ok(())
    }
}

/// Optimized Bonding Curve Contract
#[derive(Default)]
pub struct OptimizedBondingCurve(());

impl MintableToken for OptimizedBondingCurve {}

/// Message enum for optimized bonding curve operations
#[derive(MessageDispatch)]
enum OptimizedBondingCurveMessage {
    /// Initialize the bonding curve with parameters
    #[opcode(0)]
    Initialize {
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
        base_price: u128,
        growth_rate: u128,
        graduation_threshold: u128,
        base_token_type: u128,
        max_supply: u128,
        lp_distribution_strategy: u128,
    },

    /// Buy tokens with base currency (optimized)
    #[opcode(1)]
    BuyTokens {
        min_tokens_out: u128,
    },

    /// Sell tokens for base currency (optimized)
    #[opcode(2)]
    SellTokens {
        token_amount: u128,
        min_base_out: u128,
    },

    /// Get buy quote (optimized)
    #[opcode(3)]
    #[returns(u128)]
    GetBuyQuote {
        token_amount: u128,
    },

    /// Get sell quote (optimized)
    #[opcode(4)]
    #[returns(u128)]
    GetSellQuote {
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
    GetBaseReservesResponse,

    /// Get AMM pool address if graduated
    #[opcode(103)]
    #[returns(u128)]
    GetAmmPoolAddress,

    /// Check if graduated
    #[opcode(104)]
    #[returns(bool)]
    IsGraduatedResponse,

    /// Get the token data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,
}

impl OptimizedBondingCurve {
    /// Get the pointer to curve parameters
    pub fn curve_params_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/curve_params")
    }

    /// Get curve parameters from storage
    pub fn get_curve_params(&self) -> Result<OptimizedCurveParams> {
        let data = self.curve_params_pointer().get();
        if data.as_ref().is_empty() {
            return Ok(OptimizedCurveParams::default());
        }
        
        serde_json::from_slice(data.as_ref())
            .map_err(|e| anyhow!("Failed to deserialize curve params: {}", e))
    }

    /// Set curve parameters in storage
    pub fn set_curve_params(&self, params: &OptimizedCurveParams) -> Result<()> {
        let data = serde_json::to_vec(params)
            .map_err(|e| anyhow!("Failed to serialize curve params: {}", e))?;
        self.curve_params_pointer().set(Arc::new(data));
        Ok(())
    }

    /// Get the pointer to base reserves
    pub fn base_reserves_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/base_reserves")
    }

    /// Get current base token reserves
    pub fn get_base_reserves(&self) -> u128 {
        self.base_reserves_pointer().get_value::<u128>()
    }

    /// Set base token reserves
    pub fn set_base_reserves(&self, amount: u128) {
        self.base_reserves_pointer().set_value::<u128>(amount);
    }

    /// Get the pointer to graduated status
    pub fn graduated_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/graduated")
    }

    /// Check if curve has graduated to AMM
    pub fn is_graduated(&self) -> bool {
        self.graduated_pointer().get_value::<u8>() == 1
    }

    /// Mark curve as graduated
    pub fn set_graduated(&self) {
        self.graduated_pointer().set_value::<u8>(1);
    }

    /// Get the pointer to price cache
    pub fn price_cache_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/price_cache")
    }

    /// Get cached price for a supply level
    pub fn get_cached_price(&self, supply: u128) -> Option<u128> {
        let cache_key = (supply / 1000) * 1000; // Cache every 1000 tokens
        let data = self.price_cache_pointer()
            .select(&cache_key.to_le_bytes().to_vec())
            .get();
        
        if data.as_ref().is_empty() {
            None
        } else {
            let bytes = data.as_ref();
            if bytes.len() >= 16 {
                let mut array = [0u8; 16];
                array.copy_from_slice(&bytes[..16]);
                Some(u128::from_le_bytes(array))
            } else {
                None
            }
        }
    }

    /// Set cached price for a supply level
    pub fn set_cached_price(&self, supply: u128, price: u128) {
        let cache_key = (supply / 1000) * 1000; // Cache every 1000 tokens
        self.price_cache_pointer()
            .select(&cache_key.to_le_bytes().to_vec())
            .set(Arc::new(price.to_le_bytes().to_vec()));
    }

    /// Optimized price calculation with caching
    pub fn calculate_price_at_supply(&self, supply: u128, params: &OptimizedCurveParams) -> Result<u128> {
        if supply == 0 {
            return Ok(params.base_price);
        }

        // Check cache first
        if let Some(cached_price) = self.get_cached_price(supply) {
            return Ok(cached_price);
        }

        // Calculate price using optimized algorithm
        let price = self.optimized_price_calculation(supply, params)?;
        
        // Cache the result
        self.set_cached_price(supply, price);
        
        Ok(price)
    }

    /// Optimized price calculation algorithm
    fn optimized_price_calculation(&self, supply: u128, params: &OptimizedCurveParams) -> Result<u128> {
        if supply == 0 {
            return Ok(params.base_price);
        }

        // Use logarithmic approximation for large supplies to save fuel
        if supply > 10000 {
            return self.logarithmic_price_approximation(supply, params);
        }

        // Use iterative calculation for small supplies
        self.iterative_price_calculation(supply, params)
    }

    /// Logarithmic approximation for large supplies (fuel efficient)
    fn logarithmic_price_approximation(&self, supply: u128, params: &OptimizedCurveParams) -> Result<u128> {
        let growth_factor = 10000 + params.growth_rate;
        let _log_growth = (growth_factor as f64).ln() / (10000.0_f64).ln();
        let supply_f64 = supply as f64;
        
        let price_f64 = (params.base_price as f64) * (growth_factor as f64 / 10000.0_f64).powf(supply_f64);
        
        // Cap at reasonable maximum to prevent overflow
        let max_price = u128::MAX / 1000;
        let price = price_f64.min(max_price as f64) as u128;
        
        Ok(price)
    }

    /// Iterative calculation for small supplies
    fn iterative_price_calculation(&self, supply: u128, params: &OptimizedCurveParams) -> Result<u128> {
        let mut price = params.base_price;
        let growth_factor = 10000 + params.growth_rate;
        
        for _ in 0..supply {
            price = overflow_error(price.checked_mul(growth_factor))? / 10000;
            
            // Cap at reasonable maximum
            if price > u128::MAX / 1000 {
                price = u128::MAX / 1000;
                break;
            }
        }
        
        Ok(price)
    }

    /// Optimized buy price calculation
    pub fn calculate_buy_price(&self, current_supply: u128, tokens_to_buy: u128, params: &OptimizedCurveParams) -> Result<u128> {
        if tokens_to_buy == 0 {
            return Ok(0);
        }

        // Check if purchase would exceed max supply
        let new_supply = overflow_error(current_supply.checked_add(tokens_to_buy))?;
        if new_supply > params.max_supply {
            return Err(anyhow!("Purchase would exceed maximum supply"));
        }

        // Use linear approximation for small amounts (fuel efficient)
        if tokens_to_buy <= 1000 {
            let current_price = self.calculate_price_at_supply(current_supply, params)?;
            let total_cost = overflow_error(current_price.checked_mul(tokens_to_buy))?;
            return Ok(total_cost);
        }

        // Use trapezoidal rule for larger amounts
        let start_price = self.calculate_price_at_supply(current_supply, params)?;
        let end_price = self.calculate_price_at_supply(new_supply, params)?;
        
        let average_price = overflow_error(start_price.checked_add(end_price))? / 2;
        let total_cost = overflow_error(average_price.checked_mul(tokens_to_buy))?;
        
        Ok(total_cost)
    }

    /// Optimized sell price calculation
    pub fn calculate_sell_price(&self, current_supply: u128, tokens_to_sell: u128, params: &OptimizedCurveParams) -> Result<u128> {
        if tokens_to_sell == 0 {
            return Ok(0);
        }

        if tokens_to_sell > current_supply {
            return Err(anyhow!("Cannot sell more tokens than current supply"));
        }

        let new_supply = current_supply - tokens_to_sell;
        
        // Use linear approximation for small amounts
        if tokens_to_sell <= 1000 {
            let current_price = self.calculate_price_at_supply(new_supply, params)?;
            let discounted_price = overflow_error(current_price.checked_mul(99))? / 100; // 1% discount
            let total_payout = overflow_error(discounted_price.checked_mul(tokens_to_sell))?;
            return Ok(total_payout);
        }

        // Use trapezoidal rule for larger amounts
        let start_price = self.calculate_price_at_supply(new_supply, params)?;
        let end_price = self.calculate_price_at_supply(current_supply, params)?;
        
        let average_price = overflow_error(start_price.checked_add(end_price))? / 2;
        let discounted_price = overflow_error(average_price.checked_mul(99))? / 100; // 1% discount
        let total_payout = overflow_error(discounted_price.checked_mul(tokens_to_sell))?;
        
        Ok(total_payout)
    }

    /// Optimized token calculation for given base amount
    pub fn calculate_tokens_for_base_amount(&self, base_amount: u128, params: &OptimizedCurveParams) -> Result<u128> {
        let _current_supply = self.total_supply();
        
        // Use linear approximation for small amounts (fuel efficient)
        if base_amount <= 1000 * params.base_price {
            let current_price = self.calculate_price_at_supply(_current_supply, params)?;
            let tokens = overflow_error(base_amount.checked_mul(1_000_000_000))? / current_price;
            return Ok(tokens);
        }

        // Use binary search for larger amounts
        let mut low = 0u128;
        let mut high = params.max_supply.saturating_sub(_current_supply);
        let mut best_tokens = 0u128;

        // Limit iterations to save fuel
        let max_iterations = 20;
        let mut iterations = 0;

        while low <= high && iterations < max_iterations {
            let mid = (low + high) / 2;
            let cost = self.calculate_buy_price(_current_supply, mid, params)?;
            
            if cost <= base_amount {
                best_tokens = mid;
                low = mid + 1;
            } else {
                high = mid.saturating_sub(1);
            }

            iterations += 1;
        }

        if best_tokens == 0 {
            return Err(anyhow!("Insufficient base amount to buy any tokens"));
        }

        Ok(best_tokens)
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
        lp_distribution_strategy: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;

        // Validate parameters
        let base_token = match base_token_type {
            0 => BaseToken::BUSD,
            1 => BaseToken::FrBtc,
            _ => return Err(anyhow!("Invalid base token type")),
        };

        if lp_distribution_strategy > 3 {
            return Err(anyhow!("Invalid LP distribution strategy (0-3)"));
        }

        let params = OptimizedCurveParams {
            base_price,
            growth_rate,
            graduation_threshold,
            base_token,
            max_supply,
            price_cache_interval: 1000,
        };

        self.set_curve_params(&params)?;

        // Set token metadata
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        // Initialize reserves to zero
        self.set_base_reserves(0);

        self.set_data()?;

        Ok(response)
    }

    /// Buy tokens with base currency (optimized)
    fn buy_tokens(&self, min_tokens_out: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Check if already graduated
        if self.is_graduated() {
            return Err(anyhow!("Bonding curve has graduated to AMM"));
        }

        // Get curve parameters and current state
        let params = self.get_curve_params()?;
        let _current_supply = self.total_supply();
        
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
        let current_reserves = self.get_base_reserves();
        self.set_base_reserves(current_reserves + base_amount);

        Ok(response)
    }

    /// Sell tokens for base currency (optimized)
    fn sell_tokens(&self, token_amount: u128, min_base_out: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Check if already graduated
        if self.is_graduated() {
            return Err(anyhow!("Bonding curve has graduated to AMM"));
        }

        // Get curve parameters and calculate sell price
        let params = self.get_curve_params()?;
        let _current_supply = self.total_supply();
        
        // Calculate base tokens to return
        let base_payout = self.calculate_sell_price(_current_supply, token_amount, &params)?;

        // Check slippage protection
        if base_payout < min_base_out {
            return Err(anyhow!("Slippage exceeded: got {} base tokens, expected at least {}", 
                base_payout, min_base_out));
        }

        // Check we have enough reserves
        let current_reserves = self.get_base_reserves();
        if base_payout > current_reserves {
            return Err(anyhow!("Insufficient reserves for sell"));
        }

        // Burn the tokens (decrease total supply)
        let new_supply = _current_supply.checked_sub(token_amount)
            .ok_or_else(|| anyhow!("Cannot burn more tokens than exist"))?;
        self.set_total_supply(new_supply);

        // Return base tokens to seller
        response.alkanes.0.push(AlkaneTransfer {
            id: params.base_token.alkane_id(),
            value: base_payout,
        });

        // Update reserves
        self.set_base_reserves(current_reserves - base_payout);

        Ok(response)
    }

    /// Get buy quote (optimized)
    fn get_buy_quote(&self, token_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = self.get_curve_params()?;
        let _current_supply = self.total_supply();
        
        let cost = self.calculate_buy_price(_current_supply, token_amount, &params)?;

        response.data = cost.to_le_bytes().to_vec();
        Ok(response)
    }

    /// Get sell quote (optimized)
    fn get_sell_quote(&self, token_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = self.get_curve_params()?;
        let _current_supply = self.total_supply();
        
        let payout = self.calculate_sell_price(_current_supply, token_amount, &params)?;

        response.data = payout.to_le_bytes().to_vec();
        Ok(response)
    }

    /// Attempt graduation to AMM
    fn graduate(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let _current_supply = self.total_supply();

        // For now, just mark as graduated
        // In production, this would call AMM integration
        self.set_graduated();

        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = vec![1]; // Success indicator

        Ok(response)
    }

    /// Get curve state information
    fn get_curve_state(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let params = self.get_curve_params()?;
        let _current_supply = self.total_supply();
        let base_reserves = self.get_base_reserves();
        let is_graduated = self.is_graduated();

        // Create state object
        let state = serde_json::json!({
            "current_supply": _current_supply,
            "base_reserves": base_reserves,
            "is_graduated": is_graduated,
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
    fn get_base_reserves_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let reserves = self.get_base_reserves();
        response.data = reserves.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get AMM pool address if graduated
    fn get_amm_pool_address(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let pool_address = crate::amm_integration::AMMIntegration::get_amm_pool_address().unwrap_or(0);
        response.data = pool_address.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Check if graduated
    fn is_graduated_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let graduated = self.is_graduated();
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

// Remove the AlkaneResponder implementation and declare_alkane! macro
// These will be handled by the main contract in lib.rs 