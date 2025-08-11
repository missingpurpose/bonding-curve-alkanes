# System Patterns

## Contract Architecture

### Multi-Contract Pattern
- Factory contract deploys token contracts
- One contract per WASM module
- Clear separation of concerns
- Modular upgradability

### Factory Pattern
- Permissionless token deployment
- Registry management
- Fee collection
- Spam prevention

### Token Pattern
- Individual bonding curve mechanics
- State management
- Graduation logic
- Base currency support

## Security Patterns

### CEI Pattern
1. **Checks**: Validate all inputs and state
2. **Effects**: Update internal state
3. **Interactions**: External calls last

### Storage Pattern
- Dedicated storage pointers
- Efficient serialization
- Secure state management
- Clear access patterns

### Access Control
- Owner functions in factory
- Graduation validation
- Fee collection
- Emergency pause

### Arithmetic Safety
- Overflow protection
- Fixed-point precision
- Safe math operations
- Clear error handling

## Integration Patterns

### AMM Integration
- Pool creation
- Liquidity migration
- LP token distribution
- Graduation tracking

### Base Token Support
- BUSD (2:56801)
- frBTC (32:0)
- Reserve management
- Price calculations

### Frontend Integration
- Token launch wizard
- Trading interface
- Portfolio management
- Analytics dashboard

## Testing Patterns

### Unit Tests
- Individual functions
- State transitions
- Error conditions
- View functions

### Integration Tests
- Contract interaction
- AMM graduation
- Fee collection
- Registry updates

### Economic Tests
- Price calculations
- Liquidity scenarios
- Market dynamics
- Attack vectors

## Deployment Patterns

### Testnet Flow
1. Deploy factory
2. Deploy test tokens
3. Test graduation
4. Monitor performance

### Mainnet Flow
1. Security audit
2. Controlled launch
3. Monitor operations
4. Scale gradually

## Error Handling

### Contract Errors
- Clear error messages
- State validation
- Rollback mechanisms
- Event logging

### Frontend Errors
- User feedback
- Transaction status
- Error recovery
- Help system

## Monitoring Patterns

### Contract Monitoring
- Token deployments
- Trading volume
- Graduation events
- Fee collection

### Economic Monitoring
- Price movements
- Liquidity levels
- Market dynamics
- User behavior

### Technical Monitoring
- Gas usage
- State size
- Error rates
- Performance metrics