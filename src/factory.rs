//! Bonding Curve Factory Contract
//! 
//! This is the main factory contract that allows ANYONE to create new tokens
//! with bonding curves. Each token creation deploys a new bonding curve contract.

use alkanes_runtime::{
    declare_alkane, 
    message::MessageDispatch, 
    runtime::AlkaneResponder,
    storage::StoragePointer,
};
use alkanes_support::{
    response::CallResponse,
    utils::overflow_error,
};
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Platform fee for creating new tokens (0.001 BTC equivalent)
const PLATFORM_FEE: u128 = 100_000; // In satoshis
const MAX_TOKENS_PER_CREATOR: u128 = 100; // Spam prevention

/// Factory contract for deploying bonding curve tokens
#[derive(Default)]
pub struct BondingCurveFactory(());

/// Deployed token information (simplified for storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedTokenInfo {
    pub token_block: u64,
    pub token_tx: u64,
    pub creator_block: u64,
    pub creator_tx: u64,
    pub name: String,
    pub symbol: String,
    pub base_token: u8,
    pub created_at_block: u64,
    pub graduated: bool,
    pub amm_pool_block: u64,
    pub amm_pool_tx: u64,
}

impl BondingCurveFactory {
    // Storage pointers
    fn total_tokens_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory/total_tokens")
    }
    
    fn token_by_index_pointer(&self, index: u128) -> StoragePointer {
        let index_bytes = index.to_le_bytes().to_vec();
        StoragePointer::from_keyword("/factory/tokens/")
            .select(&index_bytes)
    }
    
    fn tokens_by_creator_pointer(&self, creator_block: u64, creator_tx: u64) -> StoragePointer {
        let mut key = Vec::new();
        key.extend_from_slice(&creator_block.to_le_bytes());
        key.extend_from_slice(&creator_tx.to_le_bytes());
        StoragePointer::from_keyword("/factory/creator/")
            .select(&key)
    }
    
    fn platform_fees_collected_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory/fees_collected")
    }
    
    fn factory_owner_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory/owner")
    }
    
    fn paused_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory/paused")
    }

    /// Initialize the factory contract
    fn initialize(&self, owner_block: u128, owner_tx: u128) -> Result<CallResponse> {
        // Ensure single initialization
        self.observe_initialization()
            .map_err(|_| anyhow!("Factory already initialized"))?;
        
        // Set factory owner (store as two u64s)
        self.factory_owner_pointer().set_value::<u64>(owner_block as u64);
        self.factory_owner_pointer()
            .select(&"tx".as_bytes().to_vec())
            .set_value::<u64>(owner_tx as u64);
        
        // Initialize counters
        self.total_tokens_pointer().set_value::<u128>(0);
        self.platform_fees_collected_pointer().set_value::<u128>(0);
        self.paused_pointer().set_value::<u8>(0);
        
        Ok(CallResponse::default())
    }

    /// Create a new bonding curve token
    fn create_token(
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
        
        // Check if factory is paused
        if self.paused_pointer().get_value::<u8>() != 0 {
            return Err(anyhow!("Factory is paused"));
        }
        
        // Validate parameters
        if base_price == 0 || base_price > u128::MAX / 1000 {
            return Err(anyhow!("Invalid base price"));
        }
        
        if growth_rate > 1000 {
            return Err(anyhow!("Growth rate too high (max 10%)"));
        }
        
        if graduation_threshold < 10_000_000_000 {
            return Err(anyhow!("Graduation threshold too low (min $10k)"));
        }
        
        if base_token_type > 1 {
            return Err(anyhow!("Invalid base token type"));
        }
        
        if max_supply == 0 || max_supply > 1_000_000_000_000_000 {
            return Err(anyhow!("Invalid max supply"));
        }
        
        if lp_distribution_strategy > 3 {
            return Err(anyhow!("Invalid LP distribution strategy"));
        }
        
        // Check creator hasn't exceeded token limit
        let creator_tokens = self.get_creator_token_count(context.myself.block, context.myself.tx);
        if creator_tokens >= MAX_TOKENS_PER_CREATOR {
            return Err(anyhow!(
                "Creator has reached maximum token limit: {}",
                MAX_TOKENS_PER_CREATOR
            ));
        }
        
        // Deploy new bonding curve token contract
        // In production, this would actually deploy a new contract
        // For now, we simulate by generating a unique token ID
        let total_tokens = self.total_tokens_pointer().get_value::<u128>();
        let token_block = context.myself.block + total_tokens;
        let token_tx = context.myself.tx + total_tokens;
        
        // Record the deployment
        let deployed_token = DeployedTokenInfo {
            token_block: token_block as u64,
            token_tx: token_tx as u64,
            creator_block: context.myself.block as u64,
            creator_tx: context.myself.tx as u64,
            name: format!("{}{}", 
                Self::trim_u128(name_part1), 
                Self::trim_u128(name_part2)
            ),
            symbol: Self::trim_u128(symbol),
            base_token: base_token_type as u8,
            created_at_block: 0, // Would get from context in production
            graduated: false,
            amm_pool_block: 0,
            amm_pool_tx: 0,
        };
        
        // Store token information
        let token_data = serde_json::to_vec(&deployed_token)?;
        self.token_by_index_pointer(total_tokens).set(Arc::new(token_data));
        
        // Update counters
        self.total_tokens_pointer().set_value::<u128>(total_tokens + 1);
        self.increment_creator_tokens(context.myself.block, context.myself.tx)?;
        
        // Return token ID in response (as two u128s)
        response.data = Vec::new();
        response.data.extend_from_slice(&token_block.to_le_bytes());
        response.data.extend_from_slice(&token_tx.to_le_bytes());
        
        Ok(response)
    }

    /// Get number of tokens created by an address
    fn get_creator_token_count(&self, creator_block: u128, creator_tx: u128) -> u128 {
        let count_key = "count".as_bytes().to_vec();
        self.tokens_by_creator_pointer(creator_block as u64, creator_tx as u64)
            .select(&count_key)
            .get_value::<u128>()
    }
    
    /// Increment creator's token count
    fn increment_creator_tokens(&self, creator_block: u128, creator_tx: u128) -> Result<()> {
        let count_key = "count".as_bytes().to_vec();
        let mut count_pointer = self.tokens_by_creator_pointer(creator_block as u64, creator_tx as u64)
            .select(&count_key);
        let current = count_pointer.get_value::<u128>();
        let new_count = overflow_error(current.checked_add(1))?;
        count_pointer.set_value::<u128>(new_count);
        Ok(())
    }

    /// Notify factory when a token graduates to AMM
    fn notify_graduation(&self, token_block: u128, token_tx: u128, pool_block: u128, pool_tx: u128) -> Result<CallResponse> {
        // Find and update the token record
        let total_tokens = self.total_tokens_pointer().get_value::<u128>();
        
        for i in 0..total_tokens {
            let token_data = self.token_by_index_pointer(i).get();
            if let Ok(mut token) = serde_json::from_slice::<DeployedTokenInfo>(&token_data) {
                if token.token_block == token_block as u64 && token.token_tx == token_tx as u64 {
                    token.graduated = true;
                    token.amm_pool_block = pool_block as u64;
                    token.amm_pool_tx = pool_tx as u64;
                    
                    let updated_data = serde_json::to_vec(&token)?;
                    self.token_by_index_pointer(i).set(Arc::new(updated_data));
                    break;
                }
            }
        }
        
        Ok(CallResponse::default())
    }

    /// Get total number of deployed tokens
    fn get_total_tokens(&self) -> Result<CallResponse> {
        let mut response = CallResponse::default();
        let total = self.total_tokens_pointer().get_value::<u128>();
        response.data = total.to_le_bytes().to_vec();
        Ok(response)
    }

    /// Get token information by index
    fn get_token_by_index(&self, index: u128) -> Result<CallResponse> {
        let mut response = CallResponse::default();
        let token_data = self.token_by_index_pointer(index).get();
        response.data = token_data.as_ref().to_vec();
        Ok(response)
    }

    /// Pause/unpause factory
    fn set_paused(&self, paused: u128) -> Result<CallResponse> {
        let context = self.context()?;
        
        // Only owner can pause (simplified check for now)
        // In production, verify caller is owner
        
        self.paused_pointer().set_value::<u8>(paused as u8);
        
        Ok(CallResponse::forward(&context.incoming_alkanes))
    }

    /// Helper to trim u128 to string
    fn trim_u128(v: u128) -> String {
        String::from_utf8(
            v.to_le_bytes()
                .into_iter()
                .filter(|&b| b != 0)
                .collect()
        ).unwrap_or_default()
    }
}

/// Factory message dispatch - using only supported types
#[derive(MessageDispatch)]
enum BondingCurveFactoryMessage {
    /// Initialize factory with owner
    #[opcode(0)]
    Initialize {
        owner_block: u128,
        owner_tx: u128,
    },
    
    /// Create a new bonding curve token
    #[opcode(1)]
    CreateToken {
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
    
    /// Notify graduation (called by token contracts)
    #[opcode(2)]
    NotifyGraduation {
        token_block: u128,
        token_tx: u128,
        pool_block: u128,
        pool_tx: u128,
    },
    
    /// Get total deployed tokens
    #[opcode(10)]
    GetTotalTokens,
    
    /// Get token by index
    #[opcode(11)]
    GetTokenByIndex {
        index: u128,
    },
    
    /// Pause factory
    #[opcode(20)]
    SetPaused {
        paused: u128,
    },
}

impl MessageDispatch<BondingCurveFactoryMessage> for BondingCurveFactory {
    fn dispatch(&self, message: &BondingCurveFactoryMessage) -> Result<CallResponse> {
        match message {
            BondingCurveFactoryMessage::Initialize { owner_block, owner_tx } => {
                self.initialize(*owner_block, *owner_tx)
            },
            BondingCurveFactoryMessage::CreateToken { 
                name_part1, name_part2, symbol, base_price, 
                growth_rate, graduation_threshold, base_token_type, 
                max_supply, lp_distribution_strategy 
            } => {
                self.create_token(
                    *name_part1, *name_part2, *symbol, *base_price,
                    *growth_rate, *graduation_threshold, *base_token_type,
                    *max_supply, *lp_distribution_strategy
                )
            },
            BondingCurveFactoryMessage::NotifyGraduation { token_block, token_tx, pool_block, pool_tx } => {
                self.notify_graduation(*token_block, *token_tx, *pool_block, *pool_tx)
            },
            BondingCurveFactoryMessage::GetTotalTokens => {
                self.get_total_tokens()
            },
            BondingCurveFactoryMessage::GetTokenByIndex { index } => {
                self.get_token_by_index(*index)
            },
            BondingCurveFactoryMessage::SetPaused { paused } => {
                self.set_paused(*paused)
            },
        }
    }

    fn from_opcode(opcode: u128, args: Vec<u128>) -> Result<BondingCurveFactory> {
        let _ = (opcode, args);
        Ok(BondingCurveFactory(()))
    }

    fn export_abi() -> Vec<u8> {
        Vec::new()
    }
}

impl AlkaneResponder for BondingCurveFactory {}

declare_alkane! {
    impl AlkaneResponder for BondingCurveFactory {
        type Message = BondingCurveFactoryMessage;
    }
}