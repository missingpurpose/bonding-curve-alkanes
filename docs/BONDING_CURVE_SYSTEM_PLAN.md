# Alkanes Bonding Curve System - Comprehensive Implementation Plan

## 🎯 Project Overview

**Goal**: Build a production-ready bonding curve system for Alkanes that enables new token launches with automatic liquidity provision using BUSD (2:56801) and frBTC (32:0) base pairs, with graduation to Oyl AMM pools.

**Timeline**: 4 weeks (modular approach)
**Status**: Phase 1 Complete - Core Contract Working

## 🏗️ System Architecture - CORRECTED

### **Multiple Contract Architecture Required**

Based on [Alkanes documentation](https://alkanes.build/docs/developers/disclaimer), we need **separate contracts** because Alkanes only allows one contract per WASM module.

```
┌─────────────────────────────────────────────────────────────────┐
│                    ALKANES TOKEN LAUNCHPAD ECOSYSTEM            │
├─────────────────┬───────────────────┬───────────────────────────┤
│   Factory       │  Bonding Curve    │   Oyl AMM                │
│   Contract      │   Token Contract  │   Integration             │
│   (Separate)    │   (Separate)      │   (Separate)              │
│                 │                   │                           │
│ • Deploy tokens │ • Exponential     │ • Pool creation          │
│ • Track all     │   pricing         │ • Liquidity migration    │
│ • Fee mgmt      │ • Buy/Sell logic  │ • LP distribution        │
│ • Spam control  │ • BUSD/frBTC      │ • Graduation triggers    │
└─────────────────┴───────────────────┴───────────────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │   User Frontend  │
                       │   (React/Next.js)│
                       │                  │
                       │ • Token Launch   │
                       │ • Trading UI     │
                       │ • Portfolio      │
                       │ • Analytics      │
                       └──────────────────┘
```

### **Contract Interaction Flow**

```
1. User calls Factory.createToken() with parameters
2. Factory deploys new BondingCurveToken contract instance
3. Users buy/sell tokens through BondingCurveToken.buy()/sell()
4. Price increases exponentially based on bonding curve algorithm
5. When graduation criteria met, BondingCurveToken.graduate() 
6. Oyl AMM pool created with initial liquidity from bonding curve
7. Token becomes tradeable on decentralized AMM
```

## 📋 Technical Specifications - UPDATED

### 1. Factory Contract (`bonding-curve-factory`)

**Purpose**: Deploy and manage bonding curve token launches

**Key Functions**:
- `initialize(owner_block, owner_tx)` - Set factory owner
- `create_token(name, symbol, params, base_token)` - Deploy new token
- `get_total_tokens()` - Count deployed tokens
- `get_token_by_index(index)` - Get token details
- `set_paused(paused)` - Emergency pause
- `notify_graduation(token_id, pool_id)` - Track AMM migration

**Storage**:
- Deployed token registry
- Creator token limits (anti-spam)
- Platform fee collection
- Factory configuration

**Requirements**:
- Separate crate (cannot share with token contract)
- Owner controls for emergency functions
- Spam prevention (max 100 tokens per creator)

### 2. Bonding Curve Token Contract (`bonding_curve_system`) ✅ COMPLETED

**Purpose**: Individual token with exponential bonding curve

**Key Functions**:
- `initialize(name, symbol, params)` - Set token configuration
- `buy_tokens(min_tokens_out)` - Purchase tokens with base currency
- `sell_tokens(token_amount, min_base_out)` - Sell tokens for base currency
- `get_buy_quote(token_amount)` - Price quote for buying
- `get_sell_quote(token_amount)` - Price quote for selling
- `graduate()` - Trigger AMM graduation

**Pricing Algorithm** ✅ IMPLEMENTED:
```rust
// True exponential bonding curve: price = base_price * (1 + growth_rate/10000)^supply
fn calculate_buy_price(current_supply: u128, tokens_to_buy: u128, params: &CurveParams) -> Result<u128> {
    // Fixed-point arithmetic with 9 decimal precision
    // Binary exponentiation for gas efficiency
    // Overflow protection on all calculations
}
```

**Graduation Criteria**:
- Market cap threshold ($69k USD default)
- Minimum liquidity reserves (50% of threshold)
- Time-based criteria (30 days with activity)

### 3. AMM Integration (Mock - Needs Real Oyl SDK) 🚧 IN PROGRESS

**Purpose**: Handle graduation to Oyl AMM pools

**Current Status**: Mock implementation complete, needs real Oyl integration

**Required Functions**:
- `create_oyl_pool(token_a, token_b, liquidity)` - Create real AMM pool
- `transfer_liquidity_to_pool()` - Move bonding curve reserves
- `distribute_lp_tokens(strategy)` - Handle LP token distribution

**LP Distribution Strategies**:
- **Full Burn (0)**: 100% LP tokens burned for permanent liquidity
- **Community Rewards (1)**: 80% burned, 20% to top holders
- **Creator Allocation (2)**: 90% burned, 10% to creator
- **DAO Governance (3)**: 80% burned, 20% to governance

## 🛠️ Implementation Strategy - MODULAR APPROACH

### **Phase 1: Core Infrastructure** ✅ COMPLETED
**Duration**: Week 1
**Status**: 100% Complete

1. ✅ **Project Setup**
   - Rust workspace configuration
   - Git repository structure
   - Build scripts and WASM compilation

2. ✅ **Bonding Curve Contract**
   - Exponential pricing algorithm
   - Buy/sell mechanisms
   - Graduation framework
   - 323KB WASM binary ready

3. ✅ **Storage & Security**
   - CEI pattern implementation
   - Overflow protection
   - Access controls
   - State management

### **Phase 2: Factory Contract** 🚧 IN PROGRESS
**Duration**: Week 2
**Status**: 80% Complete (needs separate crate)

1. ✅ **Contract Logic**
   - Token deployment functions
   - Registry management
   - Fee collection
   - Spam prevention

2. 🚧 **Deployment Structure**
   - Create separate `bonding-curve-factory` crate
   - Adapt for standalone compilation
   - Test deployment workflow

3. ⏳ **Integration Testing**
   - Factory + Token contract interaction
   - Multi-token deployment testing
   - Fee collection verification

### **Phase 3: Oyl AMM Integration** ⏳ PENDING
**Duration**: Week 3
**Status**: 0% Complete

1. ⏳ **Oyl SDK Integration**
   - Replace mock functions with real Oyl calls
   - Pool creation interfaces
   - Liquidity migration logic

2. ⏳ **Graduation Testing**
   - End-to-end graduation flow
   - LP token distribution
   - AMM pool verification

3. ⏳ **Security Hardening**
   - Atomic graduation operations
   - Rollback mechanisms
   - Edge case handling

### **Phase 4: Frontend & Deployment** ⏳ PENDING
**Duration**: Week 4
**Status**: 0% Complete

1. ⏳ **Frontend Development**
   - React/Next.js application
   - Wallet integration
   - Trading interface
   - Analytics dashboard

2. ⏳ **Production Deployment**
   - Testnet validation
   - Security audit
   - Mainnet launch
   - Monitoring setup

## 📁 Project Structure - UPDATED

```
alkanes-bonding-curve/                    # Main repository
├── bonding-curve-token/                  # Individual token contract ✅
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                       # Main contract logic
│   │   ├── bonding_curve.rs             # Pricing engine
│   │   ├── amm_integration.rs           # AMM graduation
│   │   └── constants.rs                 # Shared constants
│   └── target/
│       └── wasm32-unknown-unknown/
│           └── release/
│               └── bonding_curve_system.wasm  # 323KB ✅
│
├── bonding-curve-factory/                # Factory contract 🚧
│   ├── Cargo.toml                       # Separate crate
│   ├── src/
│   │   └── lib.rs                       # Factory logic
│   └── target/
│       └── wasm32-unknown-unknown/
│           └── release/
│               └── factory.wasm         # To be built
│
├── shared/                               # Shared utilities
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── math.rs                      # Mathematical functions
│       └── constants.rs                 # System constants
│
├── frontend/                             # React application ⏳
│   ├── package.json
│   ├── src/
│   │   ├── components/                  # UI components
│   │   ├── hooks/                       # Custom hooks
│   │   └── utils/                       # Utility functions
│   └── public/
│
├── scripts/                              # Deployment scripts
│   ├── build-all.sh                     # Build all contracts
│   ├── deploy-token.sh                  # Deploy token contract
│   ├── deploy-factory.sh                # Deploy factory contract
│   └── test-all.sh                      # Run all tests
│
└── docs/                                 # Documentation
    ├── BONDING_CURVE_SYSTEM_PLAN.md     # This file
    ├── DEPLOYMENT_GUIDE.md              # Deployment instructions
    ├── INTEGRATION_GUIDE.md             # Oyl integration guide
    └── FRONTEND_SPEC.md                 # Frontend requirements
```

## 🔗 Integration Points - DETAILED

### **BUSD Integration (Alkane ID: 2:56801)**

**What it is**: Bitcoin USD stablecoin on Alkanes
**How it works**: 
- Represents USD value in Bitcoin ecosystem
- 8 decimal precision (like Bitcoin)
- Used as base currency for bonding curves
- Provides stable pricing reference

**Contract Requirements**:
```rust
// In bonding curve contract
const BUSD_ALKANE_ID: u128 = (2u128 << 64) | 56801u128;

// Transfer BUSD to contract reserves
response.alkanes.0.push(AlkaneTransfer {
    id: AlkaneId::new(2, 56801),  // BUSD
    value: base_amount,
});
```

**Integration Points**:
- Bonding curve reserves storage
- Price calculations (denominated in BUSD)
- Graduation threshold checking
- AMM pool liquidity

### **frBTC Integration (Alkane ID: 32:0)**

**What it is**: Bitcoin-pegged token on Alkanes
**How it works**:
- 1:1 backed by Bitcoin
- Alternative base currency option
- Enables BTC-denominated curves
- Cross-pair arbitrage opportunities

**Contract Requirements**:
```rust
// In bonding curve contract
const FRBTC_ALKANE_ID: u128 = (32u128 << 64) | 0u128;

// Support both base currencies
pub enum BaseToken {
    BUSD,   // 2:56801
    FrBtc,  // 32:0
}
```

**Integration Points**:
- Base currency selection at token launch
- Reserve management in chosen currency
- Price calculations and thresholds
- AMM pool creation with correct base

### **Oyl AMM Integration**

**What it is**: Decentralized exchange protocol on Alkanes
**How it works**:
- Automated market maker (AMM) pools
- Constant product formula (x * y = k)
- Liquidity providers earn fees
- Pool creation through factory contracts

**Contract Requirements**:
```rust
// Replace mock functions with real Oyl calls
fn create_oyl_pool(
    token_a: AlkaneId,
    token_b: AlkaneId,
    initial_liquidity: u128,
) -> Result<AlkaneId> {
    // Call Oyl Factory contract
    // Create new pool
    // Return pool address
}
```

**Integration Points**:
- Pool creation during graduation
- Liquidity migration from bonding curve
- LP token distribution strategies
- Fee structure alignment

## 🧪 Testing Strategy - UPDATED

### **Unit Tests** ✅ PARTIALLY COMPLETE
- ✅ Mathematical functions accuracy
- ✅ Contract state transitions
- ✅ Access control mechanisms
- ⏳ Error condition handling

### **Integration Tests** ⏳ PENDING
- ⏳ Cross-contract interactions
- ⏳ AMM graduation flow
- ⏳ Multi-user scenarios
- ⏳ Edge case coverage

### **Security Tests** ✅ PARTIALLY COMPLETE
- ✅ Reentrancy protection (CEI pattern)
- ✅ Integer overflow/underflow protection
- ✅ Access control implementation
- ⏳ Economic attack vector testing

## 🚀 Deployment Strategy - DETAILED

### **Development Environment**
```bash
# Local testing with alkanes-dev-environment
cd bonding-curve-token
cargo build --target wasm32-unknown-unknown --release

# Deploy token contract
alkanes deploy --wasm target/wasm32-unknown-unknown/release/bonding_curve_system.wasm \
  --network regtest \
  --init-params "name_part1=TestToken name_part2=Demo symbol=TTD base_price=4000 growth_rate=150 graduation_threshold=69000000000 base_token_type=0 max_supply=1000000000 lp_distribution_strategy=0"
```

### **Testnet Deployment**
```bash
# Deploy to Bitcoin testnet
cd bonding-curve-token
cargo build --target wasm32-unknown-unknown --release

alkanes deploy --wasm target/wasm32-unknown-unknown/release/bonding_curve_system.wasm \
  --network testnet \
  --init-params "name_part1=TestToken name_part2=Demo symbol=TTD base_price=4000 growth_rate=150 graduation_threshold=69000000000 base_token_type=0 max_supply=1000000000 lp_distribution_strategy=0"

# Verify contract
alkanes verify --contract <deployed_address> --source src/lib.rs
```

### **Factory Deployment** (Separate)
```bash
# Create and deploy factory contract
cd bonding-curve-factory
cargo build --target wasm32-unknown-unknown --release

alkanes deploy --wasm target/wasm32-unknown-unknown/release/factory.wasm \
  --network testnet \
  --init-params "owner_block=<your_block> owner_tx=<your_tx>"
```

## 🔄 AI Terminal Coordination Strategy - UPDATED

### **Terminal 1: bonding-curve-system** ✅ COMPLETED
**Scope**: Core bonding curve token contract
**Status**: 100% Complete
**Deliverables**:
- ✅ 323KB WASM binary
- ✅ Exponential pricing algorithm
- ✅ Buy/sell mechanisms
- ✅ Graduation framework

**Next**: Move to Phase 2 (Factory Contract)

### **Terminal 2: Factory Contract Development** 🚧 IN PROGRESS
**Scope**: Create separate factory contract crate
**Responsibilities**:
- Create `bonding-curve-factory` project
- Adapt factory logic for standalone compilation
- Test factory + token interaction
- Deploy to testnet

**Deliverables**:
- Separate factory crate
- Compiled factory.wasm
- Factory deployment scripts
- Integration testing

### **Terminal 3: Oyl SDK Integration** ⏳ PENDING
**Scope**: Real AMM integration
**Responsibilities**:
- Replace mock AMM functions
- Integrate real Oyl SDK
- Test pool creation
- Verify graduation flow

**Deliverables**:
- Real AMM integration
- Pool creation working
- LP token distribution
- End-to-end testing

### **Terminal 4: Frontend Development** ⏳ PENDING
**Scope**: React/Next.js application
**Responsibilities**:
- Token launch wizard
- Trading interface
- Portfolio management
- Analytics dashboard

**Deliverables**:
- Complete frontend application
- Wallet integration
- Real-time data
- Production deployment

## 📊 Success Metrics - UPDATED

### **Technical Metrics**
- ✅ Bonding curve contract compiles (323KB WASM)
- ✅ Exponential pricing algorithm implemented
- ✅ Security patterns implemented
- ⏳ Factory contract compiles separately
- ⏳ Oyl AMM integration working
- ⏳ 100% test coverage

### **Functional Metrics**  
- ✅ Token initialization working
- ✅ Buy/sell price calculations accurate
- ✅ Graduation criteria checking
- ⏳ Factory deployment working
- ⏳ AMM graduation end-to-end
- ⏳ Multi-user testing passed

### **Integration Metrics**
- ⏳ Oyl AMM pools created successfully
- ⏳ BUSD/frBTC integrations functional
- ⏳ Factory + Token interaction working
- ⏳ Frontend prototype operational

## ⚠️ Risk Mitigation - UPDATED

### **Security Risks** ✅ ADDRESSED
- ✅ **Reentrancy**: CEI pattern implemented
- ✅ **Integer Overflow**: Safe math operations throughout
- ✅ **Access Control**: Multi-level permission system
- ⏳ **Economic Attacks**: Slippage limits and circuit breakers

### **Integration Risks** 🚧 ADDRESSING
- ✅ **AMM Compatibility**: Framework ready, needs real Oyl calls
- ✅ **Token Standards**: Compatible with Alkanes ecosystem
- ✅ **Price Oracle**: Exponential curve with precision
- ⏳ **Liquidity**: Minimum thresholds implemented

### **Operational Risks** ⏳ PENDING
- ✅ **Contract Bugs**: Core logic tested and working
- ⏳ **User Errors**: Frontend validation needed
- ⏳ **Network Issues**: Error handling needed
- ✅ **Scalability**: Efficient algorithms implemented

## 📅 Timeline Checkpoints - UPDATED

### **Week 1: Core Infrastructure** ✅ COMPLETED
- ✅ Bonding curve contract working
- ✅ Exponential pricing implemented
- ✅ WASM compilation successful
- ✅ Basic security patterns

### **Week 2: Factory Contract** 🚧 IN PROGRESS
- 🚧 Create separate factory crate
- ⏳ Compile factory.wasm
- ⏳ Test factory + token interaction
- ⏳ Deploy to testnet

### **Week 3: Oyl Integration** ⏳ PENDING
- ⏳ Replace mock AMM functions
- ⏳ Test real pool creation
- ⏳ Verify graduation flow
- ⏳ End-to-end testing

### **Week 4: Frontend & Launch** ⏳ PENDING
- ⏳ React application development
- ⏳ Wallet integration
- ⏳ Production deployment
- ⏳ Community launch

## 🎯 **Immediate Next Steps**

### **This Week (Factory Contract)**:
1. Create `bonding-curve-factory` crate
2. Adapt factory logic for standalone compilation
3. Test factory deployment
4. Verify factory + token interaction

### **Next Week (Oyl Integration)**:
1. Study Oyl SDK documentation
2. Replace mock AMM functions
3. Test real pool creation
4. Verify graduation flow

### **Following Week (Frontend)**:
1. Set up React/Next.js project
2. Implement wallet integration
3. Build trading interface
4. Create token launch wizard

---

*This plan has been updated to reflect current progress and correct architecture. The bonding curve system is now ~70% complete with a working core contract. Focus next on creating the separate factory contract and integrating with real Oyl AMM.* 