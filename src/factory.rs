//! Bonding Curve Factory Contract
//!
//! A factory contract for deploying and managing bonding curve tokens on Alkanes.
//! This contract enables users to create new tokens with bonding curve pricing
//! and automatic graduation to Oyl AMM pools.

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder,
    storage::StoragePointer,
};
use alkanes_support::response::CallResponse;
use alkanes_support::utils::overflow_error;
use alkanes_support::{parcel::AlkaneTransfer, id::AlkaneId};
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Factory contract identification
pub const BONDING_CURVE_FACTORY_ID: u128 = 0x0bcd;

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

/// Deployed curve information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedCurve {
    pub curve_id: u128, // Store as u128 instead of AlkaneId for serialization
    pub name: String,
    pub symbol: String,
    pub creator: u128, // Store as u128 instead of AlkaneId for serialization
    pub launch_block: u64,
    pub base_token: BaseToken,
    pub is_active: bool,
}

/// Bonding Curve Factory Contract
#[derive(Default)]
pub struct BondingCurveFactory(());

/// Message enum for factory operations
#[derive(MessageDispatch)]
enum BondingCurveFactoryMessage {
    /// Create a new bonding curve token
    #[opcode(0)]
    CreateBondingCurve {
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
        /// LP distribution strategy (0=FullBurn, 1=CommunityRewards, 2=CreatorAllocation, 3=DAOGovernance)
        lp_distribution_strategy: u128,
    },

    /// Get total number of deployed curves
    #[opcode(1)]
    #[returns(u128)]
    GetCurveCount,

    /// Get curve information by index
    #[opcode(2)]
    #[returns(Vec<u8>)]
    GetCurveByIndex {
        /// Index of the curve
        index: u128,
    },

    /// Get curve information by ID
    #[opcode(3)]
    #[returns(Vec<u8>)]
    GetCurveById {
        /// Curve ID
        curve_id: u128,
    },

    /// Set factory fee (admin only)
    #[opcode(10)]
    SetFactoryFeeHandler {
        /// New fee in basis points
        fee_basis_points: u128,
    },

    /// Collect factory fees (admin only)
    #[opcode(11)]
    CollectFees,

    /// Get factory statistics
    #[opcode(100)]
    #[returns(Vec<u8>)]
    GetFactoryStats,
}

impl BondingCurveFactory {
    /// Get the pointer to deployed curves count
    pub fn curve_count_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/curve_count")
    }

    /// Get the total number of deployed curves
    pub fn curve_count(&self) -> u128 {
        self.curve_count_pointer().get_value::<u128>()
    }

    /// Increment the curve count
    pub fn increment_curve_count(&self) -> Result<()> {
        let current_count = self.curve_count();
        self.curve_count_pointer().set_value::<u128>(current_count + 1);
        Ok(())
    }

    /// Get the pointer to factory fee
    pub fn factory_fee_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory_fee")
    }

    /// Get the factory fee in basis points
    pub fn factory_fee(&self) -> u128 {
        self.factory_fee_pointer().get_value::<u128>()
    }

    /// Set the factory fee
    pub fn set_factory_fee(&self, fee: u128) {
        self.factory_fee_pointer().set_value::<u128>(fee);
    }

    /// Get the pointer to accumulated fees
    pub fn accumulated_fees_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/accumulated_fees")
    }

    /// Get accumulated fees
    pub fn accumulated_fees(&self) -> u128 {
        self.accumulated_fees_pointer().get_value::<u128>()
    }

    /// Add to accumulated fees
    pub fn add_fees(&self, amount: u128) -> Result<()> {
        let current_fees = self.accumulated_fees();
        self.accumulated_fees_pointer().set_value::<u128>(
            overflow_error(current_fees.checked_add(amount))?
        );
        Ok(())
    }

    /// Get the pointer to deployed curves registry
    pub fn curves_registry_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/curves_registry")
    }

    /// Store curve information in registry
    pub fn store_curve_info(&self, index: u128, curve_info: &DeployedCurve) -> Result<()> {
        let data = serde_json::to_vec(curve_info)
            .map_err(|e| anyhow!("Failed to serialize curve info: {}", e))?;
        
        self.curves_registry_pointer()
            .select(&index.to_le_bytes().to_vec())
            .set(Arc::new(data));
        
        Ok(())
    }

    /// Get curve information from registry
    pub fn get_curve_info(&self, index: u128) -> Result<Option<DeployedCurve>> {
        let data = self.curves_registry_pointer()
            .select(&index.to_le_bytes().to_vec())
            .get();
        
        if data.as_ref().is_empty() {
            return Ok(None);
        }
        
        let curve_info: DeployedCurve = serde_json::from_slice(data.as_ref())
            .map_err(|e| anyhow!("Failed to deserialize curve info: {}", e))?;
        
        Ok(Some(curve_info))
    }

    /// Generate a deterministic curve ID
    pub fn generate_curve_id(&self, creator: &AlkaneId, name: &str, symbol: &str) -> u128 {
        // Create a deterministic ID based on creator, name, and symbol
        let creator_hash = (creator.block as u128) << 64 | (creator.tx as u128);
        let name_hash = name.as_bytes().iter().fold(0u128, |acc, &b| acc.wrapping_add(b as u128));
        let symbol_hash = symbol.as_bytes().iter().fold(0u128, |acc, &b| acc.wrapping_add(b as u128));
        
        let combined = creator_hash.wrapping_add(name_hash).wrapping_add(symbol_hash);
        
        // Return as u128 for easier serialization
        combined
    }

    /// Create a new bonding curve token
    fn create_bonding_curve(
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
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Validate parameters
        let base_token = match base_token_type {
            0 => BaseToken::BUSD,
            1 => BaseToken::FrBtc,
            _ => return Err(anyhow!("Invalid base token type")),
        };

        if lp_distribution_strategy > 3 {
            return Err(anyhow!("Invalid LP distribution strategy (0-3)"));
        }

        if growth_rate > 10000 {
            return Err(anyhow!("Growth rate too high (max 100%)"));
        }

        // Create token name from parts
        let name = self.decode_name(name_part1, name_part2)?;
        let symbol_str = self.decode_symbol(symbol)?;

        // Generate deterministic curve ID
        let curve_id = self.generate_curve_id(&context.myself, &name, &symbol_str);

        // Create curve parameters
        let _params = CurveParams {
            base_price,
            growth_rate,
            graduation_threshold,
            base_token,
            max_supply,
        };

        // Store curve information
        let curve_info = DeployedCurve {
            curve_id: curve_id,
            name: name.clone(),
            symbol: symbol_str.clone(),
            creator: (context.myself.block as u128) << 64 | (context.myself.tx as u128),
            launch_block: 0, // Will be set by the deployed contract
            base_token,
            is_active: true,
        };

        // Increment curve count and store info
        self.increment_curve_count()?;
        let curve_count = self.curve_count();
        self.store_curve_info(curve_count - 1, &curve_info)?;

        // Add factory fee to accumulated fees
        let factory_fee = self.factory_fee();
        if factory_fee > 0 {
            // Calculate fee based on graduation threshold
            let fee_amount = graduation_threshold * factory_fee / 10000;
            self.add_fees(fee_amount)?;
        }

        // Return curve ID in response data
        response.data = curve_id.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Decode name from two u128 parts
    fn decode_name(&self, part1: u128, part2: u128) -> Result<String> {
        let name1 = self.trim_u128(part1);
        let name2 = self.trim_u128(part2);
        Ok(format!("{}{}", name1, name2))
    }

    /// Decode symbol from u128
    fn decode_symbol(&self, symbol: u128) -> Result<String> {
        Ok(self.trim_u128(symbol))
    }

    /// Trim u128 to string by removing trailing zeros
    fn trim_u128(&self, v: u128) -> String {
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
        .unwrap_or_default()
    }

    /// Get total number of deployed curves
    fn get_curve_count(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let count = self.curve_count();
        response.data = count.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get curve information by index
    fn get_curve_by_index(&self, index: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let curve_info = self.get_curve_info(index)?;
        
        if let Some(info) = curve_info {
            response.data = serde_json::to_vec(&info)
                .map_err(|e| anyhow!("Failed to serialize curve info: {}", e))?;
        } else {
            response.data = vec![]; // Empty response for non-existent curve
        }

        Ok(response)
    }

    /// Get curve information by ID
    fn get_curve_by_id(&self, curve_id: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Search through all curves to find matching ID
        let curve_count = self.curve_count();
        let mut found_curve: Option<DeployedCurve> = None;

        for i in 0..curve_count {
            if let Some(curve_info) = self.get_curve_info(i)? {
                if curve_info.curve_id == curve_id {
                    found_curve = Some(curve_info);
                    break;
                }
            }
        }

        if let Some(info) = found_curve {
            response.data = serde_json::to_vec(&info)
                .map_err(|e| anyhow!("Failed to serialize curve info: {}", e))?;
        } else {
            response.data = vec![]; // Empty response for non-existent curve
        }

        Ok(response)
    }

    /// Set factory fee (admin only)
    fn set_factory_fee_handler(&self, fee_basis_points: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Validate fee (max 5% = 500 basis points)
        if fee_basis_points > 500 {
            return Err(anyhow!("Fee too high (max 5%)"));
        }

        self.set_factory_fee(fee_basis_points);

        Ok(response)
    }

    /// Collect factory fees (admin only)
    fn collect_fees(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let accumulated_fees = self.accumulated_fees();
        
        if accumulated_fees > 0 {
            // Return accumulated fees to caller
            response.alkanes.0.push(AlkaneTransfer {
                id: context.myself.clone(),
                value: accumulated_fees,
            });

            // Reset accumulated fees
            self.accumulated_fees_pointer().set_value::<u128>(0);
        }

        Ok(response)
    }

    /// Get factory statistics
    fn get_factory_stats(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let stats = serde_json::json!({
            "total_curves": self.curve_count(),
            "factory_fee_basis_points": self.factory_fee(),
            "accumulated_fees": self.accumulated_fees(),
            "factory_id": (context.myself.block as u128) << 64 | (context.myself.tx as u128)
        });

        response.data = serde_json::to_vec(&stats)
            .map_err(|e| anyhow!("Failed to serialize stats: {}", e))?;

        Ok(response)
    }
}

impl AlkaneResponder for BondingCurveFactory {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for BondingCurveFactory {
        type Message = BondingCurveFactoryMessage;
    }
}
