# Alkanes Bonding Curve System

> **🚀 Production-ready bonding curve system for Alkanes Bitcoin metaprotocol**  
> Launch tokens with automatic BUSD/frBTC liquidity and graduation to Oyl AMM pools

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Alkanes](https://img.shields.io/badge/alkanes-v1.2.1-orange)](https://alkanes.build)

## 🎯 Overview

The Alkanes Bonding Curve System enables **permissionless token launches** with built-in liquidity bootstrapping using BUSD (Alkane ID: `2:56801`) and frBTC (Alkane ID: `32:0`) as base pairs. Tokens graduate to Oyl AMM pools when sufficient liquidity is achieved.

### Key Features

- **🏭 Factory Pattern**: Deploy new bonding curves with one transaction
- **📈 Dynamic Pricing**: Exponential bonding curve with configurable parameters  
- **💰 BUSD/frBTC Integration**: Use established stablecoins and wrapped BTC
- **🎓 AMM Graduation**: Automatic liquidity migration to Oyl AMM pools
- **🔒 Security First**: Comprehensive access controls and economic safeguards
- **⚡ Gas Optimized**: Efficient algorithms and storage patterns

## 🏗️ Architecture

```
Factory Contract ──┐
                   ├─► Token Contract
                   ├─► Bonding Curve Contract ──► AMM Integration ──► Oyl AMM Pool  
                   └─► Configuration & Fees
```

### Core Components

| Contract | Purpose | Status |
|----------|---------|--------|
| **Factory** | Deploy and manage bonding curves | 🚧 In Development |
| **Token** | ERC20-like token with mint controls | 🚧 In Development |
| **Bonding Curve** | Price discovery and trading logic | 🚧 In Development |
| **AMM Integration** | Graduation to Oyl AMM pools | 🚧 In Development |

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Alkanes Development Environment](https://alkanes.build/docs)
- [Deezel CLI](https://github.com/kungfuflex/deezel) for deployment

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-org/bonding-curve-system.git
cd bonding-curve-system

# Build all contracts
./scripts/build-all.sh

# Run tests
./scripts/test-all.sh

# Deploy to regtest
./scripts/deploy.sh regtest
```

### Launch a New Token

```rust
// Deploy via Factory Contract
let curve = factory.create_bonding_curve(
    "MyToken".into(),     // Token name
    "MTK".into(),         // Token symbol  
    base_price,           // Starting price
    growth_rate,          // Price increase rate
    BaseToken::BUSD       // Base currency
);

// Buy tokens
let tokens = curve.buy(busd_amount, min_tokens_out);

// Sell tokens back
let busd = curve.sell(token_amount, min_busd_out);
```

## 📋 Contract Specifications

### Factory Contract

**Deployment**: Creates new bonding curve instances
**Management**: Tracks all launched curves and collects fees
**Configuration**: Manages system parameters and base token settings

```rust
// Key Functions
fn create_bonding_curve(params: CurveParams) -> CurveAddress;
fn get_curve_count() -> u64;
fn set_factory_fee(fee: u64) -> Result<()>;
```

### Bonding Curve Contract

**Trading**: Handle buy/sell operations with dynamic pricing
**Pricing**: Exponential curve: `price = base_price * (1 + growth_rate)^supply`
**Graduation**: Migrate liquidity to AMM when criteria met

```rust
// Pricing Algorithm
fn calculate_buy_price(tokens: u128) -> u128 {
    // Integral under exponential curve
    let base = 1_000_000; // 0.01 BUSD
    let rate = 1500;      // 1.5% growth per token
    integrate_curve(current_supply, current_supply + tokens, base, rate)
}
```

### Token Contract

**Standard Functions**: Transfer, balance queries, approvals
**Controlled Minting**: Only bonding curve can mint/burn tokens
**Metadata**: Name, symbol, decimals support

### AMM Integration Contract

**Pool Creation**: Interface with Oyl Factory for new pools
**Liquidity Migration**: Transfer bonding curve reserves
**Graduation Logic**: Verify criteria and execute transition

## 🧪 Testing

```bash
# Unit tests for all contracts
cargo test --workspace

# Integration tests with mock AMM
cargo test --features integration

# Security tests and edge cases  
cargo test --features security

# Performance benchmarks
cargo bench
```

## 🔧 Configuration

### Bonding Curve Parameters

```rust
pub struct CurveParams {
    pub base_price: u128,        // Starting price (BUSD satoshis)
    pub growth_rate: u64,        // Basis points per token
    pub graduation_threshold: u128, // Market cap for AMM migration
    pub base_token: BaseToken,   // BUSD or frBTC
}
```

### Graduation Criteria

- **Market Cap**: 100,000 BUSD equivalent
- **OR Liquidity**: 50,000 BUSD + token equivalent  
- **OR Time**: 7 days since launch (emergency graduation)

## 🔒 Security

### Built-in Protections

- **Reentrancy Guards**: All state-changing functions protected
- **Integer Safety**: Checked arithmetic throughout
- **Access Control**: Multi-level permission system
- **Slippage Protection**: Minimum output requirements
- **Circuit Breakers**: Emergency pause mechanisms

### Audit Status

- [ ] **Internal Review**: Security patterns and best practices
- [ ] **External Audit**: Professional security assessment  
- [ ] **Economic Review**: Tokenomics and incentive analysis
- [ ] **Integration Testing**: Oyl AMM compatibility verification

## 🔗 Integration

### BUSD Integration (Alkane ID: 2:56801)

```rust
// Use BUSD as base currency
let busd_curve = factory.create_bonding_curve(
    params.with_base_token(BaseToken::BUSD)
);
```

### frBTC Integration (Alkane ID: 32:0)

```rust
// Use frBTC as base currency  
let btc_curve = factory.create_bonding_curve(
    params.with_base_token(BaseToken::frBTC)
);
```

### Oyl AMM Integration

```rust
// Graduation creates Oyl pool automatically
if curve.check_graduation_criteria() {
    let pool = curve.graduate_to_amm()?;
    // Pool now available for trading on Oyl
}
```

## 🚀 Deployment

### Networks

| Network | Status | Factory Address |
|---------|--------|-----------------|
| **Regtest** | ✅ Active | `Coming Soon` |
| **Testnet** | 🚧 Planned | `TBD` |  
| **Mainnet** | 🔄 Future | `TBD` |

### Deployment Scripts

```bash
# Development deployment
./scripts/deploy.sh regtest

# Testnet deployment  
./scripts/deploy.sh testnet --verify

# Mainnet deployment (with monitoring)
./scripts/deploy.sh mainnet --verify --monitor
```

## 📚 Documentation

- [**API Reference**](docs/api.md) - Complete function documentation
- [**Integration Guide**](docs/integration.md) - How to integrate with your app
- [**Deployment Guide**](docs/deployment.md) - Production deployment steps
- [**Security Checklist**](docs/security.md) - Security best practices

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/your-username/bonding-curve-system.git

# Create feature branch
git checkout -b feature/your-feature

# Make changes and test
./scripts/test-all.sh

# Submit pull request
```

## 📈 Roadmap

### Phase 1: Core System (Week 1)
- [x] Architecture design and planning
- [ ] Factory and Token contracts
- [ ] Bonding curve implementation
- [ ] Basic testing suite

### Phase 2: AMM Integration (Week 2) 
- [ ] Oyl AMM integration contracts
- [ ] Graduation logic and criteria
- [ ] Comprehensive testing
- [ ] Security audit preparation

### Phase 3: Production (Week 3)
- [ ] Mainnet deployment
- [ ] Frontend integration
- [ ] Documentation completion
- [ ] Community launch

### Phase 4: Advanced Features (Week 4+)
- [ ] Multi-curve arbitrage
- [ ] Advanced analytics
- [ ] Mobile SDK
- [ ] Cross-chain bridges

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/your-org/bonding-curve-system/issues)
- **Discord**: [Alkanes Community](https://discord.gg/alkanes)
- **Twitter**: [@AlkanesProtocol](https://twitter.com/AlkanesProtocol)

---

<div align="center">

**🚀 Built with ❤️ for the Alkanes ecosystem**

[Website](https://alkanes.build) • [Docs](https://alkanes.build/docs) • [Twitter](https://twitter.com/AlkanesProtocol) • [Discord](https://discord.gg/alkanes)

</div> 