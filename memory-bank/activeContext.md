# Active Context

## Current Focus
Implement a production-ready Alkanes bonding curve token contract with exponential pricing and graduation to Oyl AMM, plus a separate factory contract to enable permissionless token launches.

## Recent Changes
- Core bonding curve token contract compiles to 323KB WASM
- True exponential pricing with fixed-point math and overflow protection
- AMM graduation module scaffolded (mocked)
- Factory contract logic written and moved toward separate crate
- Documentation updated with multi-contract architecture and roadmap

## Active Decisions

### Security Implementation
- CEI pattern for all state changes
- Overflow-checked arithmetic throughout
- Graduation safeguards and thresholds

### Storage Design
- Dedicated storage pointers for names, symbols, params, reserves, graduation state
- JSON-serialized parameters struct for curve config

### Interface Design
- MessageDispatch macro for opcode handling
- Separate factory and token contracts (single contract per WASM)
- AMM integration behind dedicated module

## Next Steps

### Implementation Tasks
1. **Factory Crate**
   - Create standalone `bonding-curve-factory` crate
   - Wire token deployment and registry
2. **Oyl Integration**
   - Replace mock functions with Oyl SDK calls
   - Implement pool creation and liquidity migration
3. **Testing**
   - Unit tests for buy/sell/quotes
   - Integration tests for graduation and factory flow
4. **Docs**
   - Keep roadmap and deployment guides updated

### Open Questions
- Should we implement a more sophisticated transaction tracking system for high-volume usage?
- Is there a need for additional access control beyond the initialization guard?
- Should we add a mechanism to update the value per mint after initialization?