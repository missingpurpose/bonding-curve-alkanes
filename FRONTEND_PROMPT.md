# 🎨 Frontend Development - Alkanes Bonding Curve Platform

## Mission
Build the "pump.fun for Bitcoin" - a React frontend for our production-ready bonding curve token launch platform.

## Context
- **Contract Repo**: https://github.com/missingpurpose/bonding-curve-alkanes
- **Contract Status**: Production-ready, 12 opcodes functional
- **Working Directory**: Create new React project
- **Target**: Full-featured platform competing with Solana's pump.fun

## Key Business Model
Users launch tokens with bonding curves. Trading happens against contract reserves until $69k market cap, then liquidity graduates to Oyl AMM pools.

## Core Features to Build

### 1. Token Launch Interface
```typescript
interface LaunchParams {
  // User customizable
  name_part1: string;          // "MyAwesome" 
  name_part2: string;          // "Token"
  symbol: string;              // "MAT"
  image: File;                 // Token artwork upload
  
  // Platform defaults (can be premium upgrades)
  base_price: 4000;            // $5k starting market cap
  growth_rate: 1500;           // 1.5% increase per token
  graduation_threshold: 69000; // $69k graduation
  base_token_type: 0;          // BUSD (more stable than frBTC)
  lp_distribution_strategy: 0; // Full Burn (maximum trust)
  max_supply: 1000000000;      // 1B tokens default
}
```

### 2. Real-Time Trading Engine
```typescript
interface TradingEngine {
  // Contract opcodes 3 & 4 - price quotes
  getBuyQuote(tokenAmount: number): Promise<number>;
  getSellQuote(tokenAmount: number): Promise<number>;
  
  // Contract opcodes 1 & 2 - execute trades
  buyTokens(baseAmount: number, minTokens: number): Promise<string>;
  sellTokens(tokenAmount: number, minBase: number): Promise<string>;
  
  // Live market data from contract
  currentSupply: number;
  baseReserves: number;      // BUSD/frBTC in contract
  marketCap: number;         // Current valuation
  progressToGraduation: number; // 0-100% toward $69k
}
```

### 3. Contract Integration - ALL 12 OPCODES

```typescript
interface BondingCurveContract {
  // Opcode 0: Deploy new token
  initialize(params: LaunchParams): Promise<string>;
  
  // Opcodes 1-2: Core trading
  buyTokens(minTokensOut: number): Promise<string>;
  sellTokens(tokenAmount: number, minBaseOut: number): Promise<string>;
  
  // Opcodes 3-4: Price quotes (real-time)
  getBuyQuote(tokenAmount: number): Promise<number>;
  getSellQuote(tokenAmount: number): Promise<number>;
  
  // Opcode 5: Force graduation
  graduate(): Promise<string>;
  
  // Opcodes 99-104: View functions
  getName(): Promise<string>;
  getSymbol(): Promise<string>; 
  getTotalSupply(): Promise<number>;
  getBaseReserves(): Promise<number>;
  getAmmPoolAddress(): Promise<string>;
  isGraduated(): Promise<boolean>;
  
  // Opcode 1000: Token metadata
  getData(): Promise<TokenMetadata>;
}
```

### 4. Required UI Components

#### **Launch Page**
- Token name, symbol, image upload with preview
- Show fixed economics: "$5k start → $69k graduation"
- "Launch Token" button → deploys new contract instance
- Preview bonding curve chart

#### **Trading Interface** 
- TradingView-style price chart
- Buy/Sell panels with live quotes
- Slippage protection settings
- Market cap progress bar toward graduation
- Recent trades feed

#### **Portfolio Dashboard**
```typescript
interface Portfolio {
  holdings: Array<{
    token: TokenInfo;
    balance: number;
    avgBuyPrice: number;
    currentPrice: number;
    pnl: number;
    pnlPercent: number;
  }>;
  totalValue: number;
  totalPnL: number;
  recentTrades: Transaction[];
}
```

#### **Market Discovery**
- **Trending**: Highest volume/price movement 
- **New Launches**: Recently deployed tokens
- **Near Graduation**: Close to $69k threshold
- **Graduated**: Now trading on AMM pools

### 5. Technical Stack

```json
{
  "framework": "Next.js 14+ with TypeScript",
  "styling": "Tailwind CSS + Framer Motion", 
  "state": "Zustand for global state",
  "wallet": "Custom Alkanes wallet integration",
  "charts": "TradingView Charting Library",
  "realtime": "WebSocket for live updates",
  "deployment": "Vercel with CDN"
}
```

### 6. Revenue Integration
```typescript
interface PlatformFees {
  launchFee: 0.001;        // BTC per token launch
  tradingFee: 0.005;       // 0.5% on all trades  
  graduationFee: 0.01;     // 1% of reserves at graduation
  
  premiumFeatures: {
    customEconomics: 0.01; // Custom start/graduation caps
    advancedLP: 0.005;     // Choose LP distribution strategy
  }
}
```

## User Experience Flow

### **New User Journey**
1. **Connect Wallet** → Check BUSD/frBTC balance
2. **Explore Market** → Browse trending/new tokens
3. **Launch Token** → Name, symbol, image → Deploy
4. **Share & Promote** → Social links, embed widgets
5. **Track Performance** → Portfolio dashboard

### **Trading Experience**
1. **Token Discovery** → Find interesting projects
2. **Chart Analysis** → View bonding curve progress  
3. **Quick Trade** → Buy/sell with live quotes
4. **Portfolio Sync** → Automatic balance updates
5. **Graduation Alert** → Notify when tokens hit AMM

## Key Differentiators vs pump.fun

### **Bitcoin-Native Advantages**
- ✅ **True Decentralization** (Bitcoin security)
- ✅ **Lower Trading Fees** (no Solana network fees)
- ✅ **Better Liquidity Path** (graduates to professional AMM)
- ✅ **Stable Base Currency** (BUSD > volatile SOL)

### **Platform Control Benefits**
- ✅ **Quality Standards** (curated launches)
- ✅ **Anti-Rug Features** (LP burn by default)
- ✅ **Professional UI/UX** (better than pump.fun)
- ✅ **Advanced Features** (custom economics, strategies)

## Success Metrics
- [ ] **Launch Flow**: <2 minutes from idea to deployed token
- [ ] **Trading Speed**: <3 second trade confirmations
- [ ] **Mobile Performance**: Full feature parity
- [ ] **User Retention**: 30% weekly active return rate
- [ ] **Revenue**: $10k+ monthly fees within 60 days

## Expected Deliverables
1. **Complete React Application** with all features
2. **Mobile-Responsive Design** (mobile-first)
3. **Alkanes Wallet Integration** with security best practices
4. **Real-Time Trading Interface** with live price feeds
5. **Portfolio Management** with P&L tracking
6. **Admin Dashboard** for platform management
7. **Testing Suite** with >90% coverage
8. **Production Deployment** ready for users

## Contract Economics (Pre-configured)
- **Starting Market Cap**: $5,000 USD
- **Graduation Threshold**: $69,000 USD  
- **Base Currencies**: BUSD (2:56801) or frBTC (32:0)
- **LP Strategy**: Full Burn (0) as platform default
- **Growth Rate**: 1.5% price increase per token minted

**Focus on creating an exceptional user experience that makes token launches feel magical while maintaining Bitcoin's security and decentralization.** 