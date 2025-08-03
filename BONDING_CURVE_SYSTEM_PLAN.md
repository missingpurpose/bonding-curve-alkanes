# Alkanes Bonding Curve System - Comprehensive Implementation Plan

## 🎯 Project Overview

**Goal**: Build a production-ready bonding curve system for Alkanes that enables new token launches with automatic liquidity provision using BUSD (2:56801) and frBTC (32:0) base pairs, with graduation to Oyl AMM pools.

**Timeline**: 24 hours
**Status**: In Development

## 🏗️ System Architecture

### Core Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Factory        │    │  Bonding Curve   │    │  AMM Integration│
│  Contract       │───▶│  Contract        │───▶│  Contract       │
│                 │    │                  │    │                 │
│ • Deploy curves │    │ • Price algorithm│    │ • Pool creation │
│ • Track launches│    │ • Buy/Sell logic │    │ • Liquidity     │
│ • Fee management│    │ • BUSD/frBTC     │    │   graduation    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │   Token          │
                       │   Contract       │
                       │                  │
                       │ • ERC20-like     │
                       │ • Mint/Transfer  │
                       │ • Metadata       │
                       └──────────────────┘
```

### Contract Interaction Flow

```
1. User calls Factory.createBondingCurve()
2. Factory deploys new Token and BondingCurve contracts
3. Users buy tokens through BondingCurve.buy() 
4. Price increases based on bonding curve algorithm
5. When criteria met, BondingCurve.graduate() moves liquidity to Oyl AMM
6. Token becomes tradeable on AMM with initial liquidity
```

## 📋 Technical Specifications

### 1. Factory Contract (`bonding-curve-factory`)

**Purpose**: Deploy and manage bonding curve projects

**Key Functions**:
- `create_bonding_curve(name, symbol, curve_params, base_token)` 
- `get_curve_count()` - Total deployed curves
- `get_curve_by_id(id)` - Get curve details
- `set_factory_fee(fee)` - Update factory fees
- `collect_fees()` - Withdraw accumulated fees

**Storage**:
- Deployed curve registry
- Factory configuration (fees, admin)
- Base token configurations (BUSD, frBTC)

### 2. Bonding Curve Contract (`bonding-curve-core`)

**Purpose**: Handle token sales with dynamic pricing

**Key Functions**:
- `buy(min_tokens_out)` - Buy tokens with base currency
- `sell(token_amount, min_base_out)` - Sell tokens back to curve
- `get_buy_price(token_amount)` - Calculate purchase price
- `get_sell_price(token_amount)` - Calculate sell price  
- `graduate()` - Move liquidity to AMM when criteria met
- `get_curve_state()` - Current price, supply, reserves

**Pricing Algorithm**:
```rust
// Exponential bonding curve: price = base_price * (1 + growth_rate) ^ supply
fn calculate_price(current_supply: u128, tokens_to_buy: u128) -> u128 {
    let base_price = 1_000_000; // 0.01 BUSD in satoshis
    let growth_rate = 1500; // 1.5% per token (in basis points)
    
    // Integral calculation for area under curve
    let start_price = base_price * pow_basis_points(10000 + growth_rate, current_supply);
    let end_price = base_price * pow_basis_points(10000 + growth_rate, current_supply + tokens_to_buy);
    
    (start_price + end_price) * tokens_to_buy / 2
}
```

**Graduation Criteria**:
- Minimum market cap (e.g., 100,000 BUSD)
- OR minimum liquidity (e.g., 50,000 BUSD + equivalent tokens)
- OR time-based (e.g., 7 days active)

### 3. AMM Integration Contract (`amm-integration`)

**Purpose**: Handle graduation to Oyl AMM pools

**Key Functions**:
- `create_pool(token_a, token_b, initial_liquidity)` - Create Oyl pool
- `add_initial_liquidity()` - Add bonding curve reserves to pool
- `verify_graduation()` - Check if graduation criteria met
- `finalize_graduation()` - Complete transition process

**Integration Points**:
- Oyl Factory contract calls
- Pool creation with proper ratios
- LP token handling for initial liquidity

### 4. Token Contract (`bonding-curve-token`)

**Purpose**: Standard token with bonding curve integration

**Key Functions**:
- Standard ERC20-like functions (`transfer`, `balance_of`, etc.)
- `mint(to, amount)` - Only callable by bonding curve
- `burn(from, amount)` - Only callable by bonding curve
- Metadata functions (`name`, `symbol`, `decimals`)

## 🛠️ Implementation Strategy

### Phase 1: Core Infrastructure (Hours 1-6)
1. **Project Setup**
   - Rust workspace configuration
   - Git repository with proper structure
   - Build scripts and CI/CD setup

2. **Base Contracts**  
   - Token contract implementation
   - Factory contract base structure
   - Storage patterns and utilities

### Phase 2: Bonding Curve Logic (Hours 7-12)
1. **Pricing Algorithm**
   - Mathematical functions for curve calculations
   - Buy/sell price computations
   - Slippage protection

2. **Core Trading Functions**
   - Token purchase logic
   - Token selling mechanism
   - Reserve management

### Phase 3: AMM Integration (Hours 13-18)
1. **Oyl AMM Integration**
   - Pool creation interfaces
   - Liquidity migration logic
   - Graduation criteria checking

2. **Security & Testing**
   - Comprehensive test suite
   - Security review and hardening
   - Edge case handling

### Phase 4: Deployment & Frontend (Hours 19-24)
1. **Deployment System**
   - Automated deployment scripts
   - Contract verification
   - Configuration management

2. **Frontend Planning**
   - API specifications
   - Integration documentation
   - UI/UX requirements

## 📁 Project Structure

```
bonding-curve-system/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # Project documentation
├── scripts/
│   ├── build-all.sh          # Build all contracts
│   ├── deploy.sh             # Deployment automation
│   └── test-all.sh           # Run all tests
├── contracts/
│   ├── factory/              # Factory contract
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── bonding-curve/        # Core bonding curve
│   │   ├── Cargo.toml  
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── token/                # Token implementation
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   └── amm-integration/      # AMM graduation
│       ├── Cargo.toml
│       ├── src/lib.rs
│       └── tests/
├── shared/                   # Shared utilities
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── math.rs           # Mathematical functions
│       ├── constants.rs      # System constants  
│       └── errors.rs         # Error definitions
└── docs/                     # Documentation
    ├── api.md               # API documentation
    ├── deployment.md        # Deployment guide
    └── integration.md       # Integration guide
```

## 🔗 Integration Points

### BUSD Integration (Alkane ID: 2:56801)
- Base currency for bonding curves
- Reserve token for liquidity
- Price denominated in BUSD

### frBTC Integration (Alkane ID: 32:0)  
- Alternative base currency
- BTC-denominated curves
- Cross-pair arbitrage opportunities

### Oyl AMM Integration
- Pool creation through Oyl Factory
- Liquidity migration from bonding curve
- LP token management
- Fee structure alignment

## 🧪 Testing Strategy

### Unit Tests
- Mathematical functions accuracy  
- Contract state transitions
- Access control mechanisms
- Error condition handling

### Integration Tests
- Cross-contract interactions
- AMM graduation flow
- Multi-user scenarios
- Edge case coverage

### Security Tests
- Reentrancy protection
- Integer overflow/underflow
- Access control bypass attempts
- Economic attack vectors

## 🚀 Deployment Strategy

### Development Environment
```bash
# Local testing with alkanes-dev-environment
docker-compose up -d
deezel alkanes deploy --contract factory.wasm --network regtest
```

### Testnet Deployment
```bash  
# Bitcoin testnet deployment
deezel alkanes deploy --contract factory.wasm --network testnet
deezel alkanes verify --contract <address> --source factory.rs
```

### Mainnet Deployment
```bash
# Production deployment with monitoring
deezel alkanes deploy --contract factory.wasm --network mainnet
deezel monitor --contract <address> --alerts all
```

## 🔄 AI Terminal Coordination Strategy

### Terminal 1: bonding-curve-system (Current)
**Scope**: Core contract development and architecture
**Responsibilities**:
- Contract implementation (Factory, BondingCurve, Token, AMM)
- Mathematical algorithms and pricing logic
- Security patterns and access control
- Testing and deployment scripts

### Terminal 2: oyl-sdk Integration 
**Scope**: `/Volumes/btc-node/everything-alkanes/oyl-sdk-main-/`
**Responsibilities**:
- SDK integration for bonding curve interactions
- Client-side price calculations and estimates
- Real-time data subscriptions
- TypeScript/JavaScript API wrappers

**Handoff Plan**: 
```typescript
// SDK integration requirements
interface BondingCurveSDK {
  getCurvePrice(curveId: string): Promise<number>;
  buyTokens(curveId: string, baseAmount: number): Promise<Transaction>;
  sellTokens(curveId: string, tokenAmount: number): Promise<Transaction>;  
  getCurveState(curveId: string): Promise<CurveState>;
  estimateGraduation(curveId: string): Promise<GraduationEstimate>;
}
```

### Terminal 3: Frontend Development
**Scope**: Web application development
**Responsibilities**:
- React/Next.js bonding curve interface
- Real-time price charts and analytics
- Token launch wizard and management
- Portfolio tracking and management

**Requirements Document**: Will be provided after core contracts are complete

## 📊 Success Metrics

### Technical Metrics
- [ ] All contracts compile without warnings
- [ ] 100% test coverage on core functions
- [ ] Gas optimization (< 2M gas per transaction)
- [ ] Security audit checklist completion

### Functional Metrics  
- [ ] Successful token launch end-to-end
- [ ] AMM graduation working correctly
- [ ] Price calculations accurate within 0.1%
- [ ] Multi-user concurrent testing passed

### Integration Metrics
- [ ] Oyl AMM pools created successfully
- [ ] BUSD/frBTC integrations functional
- [ ] SDK integration working
- [ ] Frontend prototype operational

## ⚠️ Risk Mitigation

### Security Risks
- **Reentrancy**: Use checks-effects-interactions pattern
- **Integer Overflow**: Safe math operations throughout
- **Access Control**: Multi-level permission system
- **Economic Attacks**: Slippage limits and circuit breakers

### Integration Risks
- **AMM Compatibility**: Thorough testing with Oyl contracts
- **Token Standards**: Ensure compatibility with ecosystem
- **Price Oracle**: Redundant price calculation methods
- **Liquidity**: Minimum thresholds and guarantees

### Operational Risks
- **Contract Bugs**: Comprehensive testing and audits  
- **User Errors**: Clear error messages and validation
- **Network Issues**: Graceful degradation and retries
- **Scalability**: Efficient algorithms and data structures

## 📅 Timeline Checkpoints

- **Hour 6**: Core contracts structure complete
- **Hour 12**: Bonding curve logic implemented and tested
- **Hour 18**: AMM integration complete with tests
- **Hour 24**: Deployment ready with documentation

---

*This plan serves as the master reference for the bonding curve system development. All AI terminals should reference this document for consistency and coordination.* 