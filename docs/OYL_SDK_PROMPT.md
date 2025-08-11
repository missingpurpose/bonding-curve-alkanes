# 🔗 OYL SDK Integration - Replace Mock Function

## Mission
Replace ONE mock function in our production-ready bonding curve contract with real Oyl AMM calls.

## Context
- **Contract Repo**: https://github.com/missingpurpose/bonding-curve-alkanes
- **Status**: 95% complete, only AMM graduation needs real implementation
- **Working Directory**: `/Volumes/btc-node/everything-alkanes/oyl-sdk-main-/`
- **Target File**: `../bonding-curve-system/src/amm_integration.rs` lines 99-115

## Your Single Task: Replace This Mock Function

```rust
// File: src/amm_integration.rs (lines 99-115)
fn create_oyl_pool(
    _context: &Context,
    base_token: &BaseToken,
    token_liquidity: u128,
    base_liquidity: u128,
) -> Result<u128> {
    // TODO: REPLACE THIS MOCK WITH REAL OYL CALLS
    // 1. Call Oyl Factory to create Token/BUSD or Token/frBTC pool
    // 2. Transfer our tokens + base currency to new pool
    // 3. Handle LP tokens based on lp_distribution_strategy
    // 4. Return real pool address
    
    let pool_address = Self::generate_pool_address(base_token, token_liquidity, base_liquidity);
    Ok(pool_address)
}
```

## Real Implementation Requirements

### 1. Oyl Factory Integration
```rust
fn create_oyl_pool(
    context: &Context,
    base_token: &BaseToken,
    token_liquidity: u128,
    base_liquidity: u128,
) -> Result<u128> {
    // Import Oyl Factory from oyl-sdk
    let factory_address = match base_token {
        BaseToken::BUSD => BUSD_FACTORY_ADDRESS,
        BaseToken::FrBtc => FRBTC_FACTORY_ADDRESS,
    };
    
    // 1. Create pool via Factory
    let pool_address = call_oyl_factory_create_pool(
        factory_address,
        context.myself.clone(),    // Our bonding curve token
        base_token.alkane_id(),    // BUSD(2:56801) or frBTC(32:0)
    )?;
    
    // 2. Add initial liquidity
    transfer_to_pool(pool_address, token_liquidity, base_liquidity)?;
    
    // 3. Handle LP tokens (get amount from pool)
    let lp_tokens = get_lp_token_balance(pool_address, context.myself.clone())?;
    distribute_lp_tokens(lp_tokens, context)?;
    
    Ok(pool_address)
}
```

### 2. Key Integration Points
- **Base Tokens**: BUSD (AlkaneId 2:56801) and frBTC (AlkaneId 32:0)
- **LP Distribution**: Handle 4 strategies (0=Full Burn, 1=Community, 2=Creator, 3=DAO)
- **Atomic Operation**: All steps succeed or none (no partial graduations)
- **Error Handling**: Return proper Result<u128> with descriptive errors

### 3. Expected Function Calls
```rust
// Use oyl-sdk to implement these:
call_oyl_factory_create_pool(factory_addr, token_a, token_b) -> Result<u128>
transfer_to_pool(pool_addr, amount_a, amount_b) -> Result<()>
get_lp_token_balance(pool_addr, owner) -> Result<u128>
distribute_lp_tokens(lp_amount, strategy) -> Result<()>
```

## Success Criteria
- [ ] Real pool creation (no mock addresses)
- [ ] Successful token transfers to pool
- [ ] LP tokens handled per strategy
- [ ] Works with both BUSD and frBTC
- [ ] Returns actual pool address

## Context Variables Available
```rust
// Available in function scope:
context.myself           // Our bonding curve token AlkaneId
base_token.alkane_id()   // BUSD or frBTC AlkaneId
token_liquidity          // Amount of our token for pool
base_liquidity           // Amount of BUSD/frBTC for pool
self.lp_distribution_strategy.get_value::<u128>() // Strategy 0-3
```

**This is the ONLY remaining piece. Everything else works perfectly!** 