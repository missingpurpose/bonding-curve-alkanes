# Bonding Curve Factory Contract Guide

> Instructions for implementing the factory contract for the Alkanes bonding curve system.

## Overview

The factory contract is a separate WASM module that deploys and manages bonding curve token contracts. It must be implemented as a separate crate because Alkanes only allows one contract per WASM module.

## Repository

**Location**: github.com/missingpurpose/bonding-curve-factory
**Local Path**: `/Volumes/btc-node/everything-alkanes/external-contracts/bonding-curve-factory/`

## Contract Architecture

### Core Components

1. **Factory Contract** (309KB WASM)
   - Deploys new token contracts
   - Tracks deployed tokens
   - Manages platform fees
   - Prevents spam

2. **Token Registry**
   - Stores token metadata
   - Tracks graduation status
   - Maintains creator limits
   - Enables token discovery

### Opcodes

| Code | Function | Description |
|------|----------|-------------|
| 0 | `initialize(owner_block, owner_tx)` | Set factory owner |
| 1 | `create_token(params...)` | Deploy new token |
| 2 | `notify_graduation(token, pool)` | Update token AMM status |
| 10 | `get_total_tokens()` | Count deployed tokens |
| 11 | `get_token_by_index(index)` | Get token details |
| 20 | `set_paused(paused)` | Emergency pause |

### Storage Structure

```rust
// Storage pointers
/factory/total_tokens       -> u128
/factory/tokens/{index}     -> DeployedTokenInfo
/factory/creator/{id}/count -> u128
/factory/fees_collected    -> u128
/factory/owner            -> (block: u64, tx: u64)
/factory/paused          -> u8
```

### Token Creation Parameters

```rust
struct DeployedTokenInfo {
    token_block: u64,
    token_tx: u64,
    creator_block: u64,
    creator_tx: u64,
    name: String,
    symbol: String,
    base_token: u8,      // 0 = BUSD, 1 = frBTC
    created_at_block: u64,
    graduated: bool,
    amm_pool_block: u64,
    amm_pool_tx: u64,
}
```

## Implementation Steps

### 1. Project Setup
```bash
# Create factory crate
cd /Volumes/btc-node/everything-alkanes/external-contracts/
cargo new --lib bonding-curve-factory

# Add dependencies to Cargo.toml
[dependencies]
alkanes-runtime = { git = "https://github.com/kungfuflex/alkanes-rs" }
alkanes-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
metashrew-support = { git = "https://github.com/sandshrewmetaprotocols/metashrew" }
anyhow = "1.0.94"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. Core Implementation

1. **Storage Management**
   ```rust
   // Implement storage pointers
   fn total_tokens_pointer(&self) -> StoragePointer;
   fn token_by_index_pointer(&self, index: u128) -> StoragePointer;
   fn tokens_by_creator_pointer(&self, creator_block: u64, creator_tx: u64) -> StoragePointer;
   ```

2. **Token Deployment**
   ```rust
   // Deploy new token contract
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
   ) -> Result<CallResponse>;
   ```

3. **Registry Management**
   ```rust
   // Track deployed tokens
   fn record_deployment(
       &self,
       token_block: u64,
       token_tx: u64,
       creator_block: u64,
       creator_tx: u64,
       name: String,
       symbol: String,
       base_token: u8,
   ) -> Result<()>;
   ```

### 3. Security Features

1. **Initialization Guard**
   ```rust
   fn initialize(&self, owner_block: u128, owner_tx: u128) -> Result<CallResponse> {
       self.observe_initialization()?;
       // Set owner and initialize state
   }
   ```

2. **Spam Prevention**
   ```rust
   const MAX_TOKENS_PER_CREATOR: u128 = 100;
   
   fn check_creator_limit(&self, creator_block: u64, creator_tx: u64) -> Result<()>;
   ```

3. **Fee Collection**
   ```rust
   const PLATFORM_FEE: u128 = 100_000; // 0.001 BTC
   
   fn collect_fee(&self) -> Result<()>;
   ```

### 4. Testing Requirements

1. **Unit Tests**
   ```rust
   #[test]
   fn test_initialization();
   #[test]
   fn test_token_creation();
   #[test]
   fn test_graduation_notification();
   #[test]
   fn test_creator_limits();
   #[test]
   fn test_fee_collection();
   ```

2. **Integration Tests**
   - Test with token contract
   - Verify graduation flow
   - Check registry updates
   - Test fee collection

### 5. Deployment Process

```bash
# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
alkanes deploy --wasm target/wasm32-unknown-unknown/release/factory.wasm \
  --network testnet \
  --init-params "owner_block=<block> owner_tx=<tx>"
```

## Integration Points

### 1. Token Contract
- Factory deploys token contract
- Token notifies factory on graduation
- Factory tracks token status

### 2. Frontend Integration
- Token launch wizard
- Deployed tokens list
- Creator analytics
- Fee tracking

### 3. AMM Integration
- Track graduation status
- Store pool addresses
- Monitor liquidity

## Success Criteria

1. **Functionality**
   - [ ] Token deployment works
   - [ ] Registry updates correctly
   - [ ] Graduation tracking works
   - [ ] Fee collection works

2. **Security**
   - [ ] Initialization guard works
   - [ ] Creator limits enforced
   - [ ] Access controls verified
   - [ ] Fee collection secure

3. **Integration**
   - [ ] Works with token contract
   - [ ] Works with frontend
   - [ ] Works with AMM

## Next Steps

1. **Implementation**
   - [ ] Set up project structure
   - [ ] Implement core functions
   - [ ] Add security features
   - [ ] Write tests

2. **Testing**
   - [ ] Run unit tests
   - [ ] Test with token contract
   - [ ] Test on testnet
   - [ ] Verify fee collection

3. **Documentation**
   - [ ] Update technical docs
   - [ ] Add deployment guide
   - [ ] Document frontend integration

4. **Deployment**
   - [ ] Deploy to testnet
   - [ ] Monitor operations
   - [ ] Track fees
   - [ ] Verify security

## References

- [Alkanes Developer Docs](https://alkanes.build/docs/developers/disclaimer)
- [Token Contract Repo](https://github.com/missingpurpose/bonding-curve-alkanes)
- [Factory Contract Repo](https://github.com/missingpurpose/bonding-curve-factory)
