# Active Context

## Current Focus
Implementing a production-ready bonding curve system with factory and token contracts for Alkanes, enabling permissionless token launches with BUSD/frBTC base pairs and graduation to Oyl AMM.

## Recent Changes
- Token contract compiles to 323KB WASM
- Factory contract compiles to 309KB WASM
- Comprehensive test suite added
- Documentation updated for multi-contract architecture
- Factory contract moved to separate crate

## Active Decisions

### Contract Architecture
- Separate WASM modules for factory and token contracts
- Factory deploys and tracks token contracts
- Token contracts handle bonding curve mechanics
- AMM integration for graduation

### Security Implementation
- CEI pattern for state changes
- Overflow protection throughout
- Access controls on critical functions
- Spam prevention in factory

### Storage Design
- Dedicated storage pointers per contract
- JSON serialization for complex data
- Efficient token registry in factory
- Secure state management

### Interface Design
- Clear opcode structure
- Consistent parameter types
- Comprehensive view functions
- Error handling patterns

## Next Steps

### AMM Integration
- Study Oyl SDK documentation
- Replace mock functions
- Test pool creation
- Verify graduation flow

### Security Audit
- Review state changes
- Check arithmetic operations
- Verify access controls
- Test economic scenarios

### Testnet Deployment
- Deploy both contracts
- Test interactions
- Monitor performance
- Track fees

## Open Questions
- What are the optimal graduation thresholds?
- Should LP distribution strategies be more configurable?
- How to handle failed graduations?
- What monitoring systems are needed?