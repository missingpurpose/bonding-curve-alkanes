//! Shared constants for the bonding curve system

use alkanes_support::id::AlkaneId;

// Supported base tokens
pub const BUSD_ALKANE_ID: u128 = (2u128 << 64) | 56801u128;  // 2:56801
pub const FRBTC_ALKANE_ID: u128 = (32u128 << 64) | 0u128;    // 32:0

// Platform configuration
pub const PLATFORM_FEE: u128 = 100_000;           // 0.001 BTC in satoshis
pub const DEFAULT_GRADUATION_THRESHOLD: u128 = 69_000_000_000; // $69k USD
pub const MIN_GRADUATION_THRESHOLD: u128 = 10_000_000_000;     // $10k USD
pub const MAX_TOKENS_PER_CREATOR: u128 = 100;     // Spam prevention

// Bonding curve defaults
pub const DEFAULT_BASE_PRICE: u128 = 4_000;       // 4,000 sats (~$5k initial mcap)
pub const DEFAULT_GROWTH_RATE: u128 = 150;        // 1.5% per token (in basis points)
pub const MAX_GROWTH_RATE: u128 = 1_000;          // 10% max growth per token
pub const DEFAULT_MAX_SUPPLY: u128 = 1_000_000_000; // 1 billion tokens

// LP Distribution Strategies
pub const LP_STRATEGY_FULL_BURN: u128 = 0;        // 100% burned
pub const LP_STRATEGY_COMMUNITY: u128 = 1;        // 80% burn, 20% to holders
pub const LP_STRATEGY_CREATOR: u128 = 2;          // 90% burn, 10% to creator
pub const LP_STRATEGY_DAO: u128 = 3;              // 80% burn, 20% to DAO

// Trading parameters
pub const SELL_DISCOUNT_BPS: u128 = 200;          // 2% discount on sells
pub const MAX_SLIPPAGE_BPS: u128 = 500;           // 5% max slippage default
pub const MIN_BUY_AMOUNT: u128 = 10_000;          // Minimum buy in sats
pub const MAX_BUY_PERCENTAGE: u128 = 10;          // Max 10% of supply per tx

// Graduation parameters
pub const MIN_HOLDERS_FOR_GRADUATION: u128 = 100; // Minimum unique holders
pub const GRADUATION_TIME_BLOCKS: u64 = 4_320;    // ~30 days at 10 min blocks
pub const AMM_LIQUIDITY_PERCENTAGE: u128 = 80;    // 80% of reserves to AMM
pub const TOKEN_LIQUIDITY_CAP: u128 = 20;         // Max 20% of supply to AMM

// Math constants
pub const PRECISION: u128 = 1_000_000_000;        // 9 decimal places
pub const BASIS_POINTS: u128 = 10_000;            // 100% = 10,000 bps
pub const MAX_PRICE: u128 = u128::MAX / 1_000_000; // Price ceiling

/// Get AlkaneId for BUSD
pub fn busd_id() -> AlkaneId {
    AlkaneId::new(2, 56801)
}

/// Get AlkaneId for frBTC
pub fn frbtc_id() -> AlkaneId {
    AlkaneId::new(32, 0)
}