# Additional Zero-Knowledge Compliance Circuits

This document outlines additional fund properties that can be proven using zero-knowledge circuits, expanding beyond the current implementation.

---

## ✅ Currently Implemented Circuits

### 1. **Position Limit Circuit** ✅
**Purpose**: Proves no single asset exceeds maximum portfolio percentage

```rust
// Proves: Each asset ≤ 40% of portfolio (without revealing amounts)
Private: [$35M BENJI, $30M BUIDL, $25M RE, $10M USDC]
Public: max_position = 40%
Output: ✓ Compliant (all positions ≤ 40%)
```

**File**: `circuits/src/position_limit.rs`

### 2. **Liquidity Reserve Circuit** ✅
**Purpose**: Proves minimum USDC/stablecoin reserve

```rust
// Proves: USDC ≥ 10% of portfolio (without revealing amounts)
Private: USDC = $10M, Total = $100M
Public: min_liquidity = 10%
Output: ✓ Compliant (10% ≥ 10%)
```

**File**: `circuits/src/liquidity_reserve.rs`

### 3. **Asset Whitelist Circuit** ✅
**Purpose**: Proves all holdings are approved assets using Merkle proofs

```rust
// Proves: Asset is in approved list (without revealing which asset)
Private: asset_hash, merkle_path
Public: merkle_root
Output: ✓ Asset approved
```

**File**: `circuits/src/whitelist.rs`

---

## 🚀 Proposed New Circuits

### 📊 **Allocation & Diversification**

#### 4. **Asset Class Allocation Circuit**
**Purpose**: Prove portfolio meets target allocation ranges

**Use Case**: Pension fund must maintain 60% stocks, 30% bonds, 10% cash (±5%)

```rust
Circuit: AssetClassAllocationCircuit {
    // Private inputs
    stock_total: $60M,     // Hidden
    bond_total: $30M,      // Hidden
    cash_total: $10M,      // Hidden
    portfolio_total: $100M, // Hidden

    // Public constraints
    stock_range: (55%, 65%),   // 60% ± 5%
    bond_range: (25%, 35%),    // 30% ± 5%
    cash_range: (5%, 15%),     // 10% ± 5%
}

Output: ✓ All allocations within target ranges
Privacy: Exact amounts remain hidden
```

**Complexity**: ~150 constraints per asset class

#### 5. **Geographic Diversification Circuit**
**Purpose**: Prove exposure limits by country/region

**Use Case**: Maximum 30% in emerging markets, 20% in any single country

```rust
Circuit: GeographicDiversificationCircuit {
    // Private inputs
    us_exposure: $40M,
    eu_exposure: $30M,
    asia_exposure: $20M,
    emerging_markets: $10M,

    // Public constraints
    max_emerging_markets: 30%,
    max_single_country: 20%,
}

Output: ✓ Geographic limits respected
Privacy: Country-specific holdings hidden
```

#### 6. **Sector Concentration Circuit**
**Purpose**: Prove sector exposure within limits

**Use Case**: No more than 25% in tech, 20% in finance, etc.

```rust
Circuit: SectorConcentrationCircuit {
    sector_exposures: [tech, finance, healthcare, ...],
    sector_limits: [25%, 20%, 20%, ...],
}
```

---

### 💰 **Spending & Flow Controls**

#### 7. **Daily Spending Limit Circuit**
**Purpose**: Prove total spending doesn't exceed daily limit

**Use Case**: Maximum $5M in transactions per day

```rust
Circuit: DailySpendingCircuit {
    // Private inputs
    transactions: [
        { amount: $1.2M, timestamp: 08:00 },
        { amount: $800K, timestamp: 10:30 },
        { amount: $2.0M, timestamp: 14:15 },
    ],

    // Public constraints
    daily_limit: $5M,
    current_date: 2024-11-13,
}

Computation:
- Filter transactions for current_date
- Sum transaction amounts
- Prove: sum ≤ daily_limit

Output: ✓ Spent $4M < $5M limit
Privacy: Individual transaction amounts hidden
```

**Complexity**: ~50 constraints per transaction

#### 8. **Withdrawal Rate Circuit**
**Purpose**: Prove redemptions don't exceed maximum rate

**Use Case**: Maximum 10% portfolio withdrawals per month

```rust
Circuit: WithdrawalRateCircuit {
    // Private inputs
    withdrawal_total_month: $8M,
    portfolio_value_start: $100M,

    // Public constraints
    max_withdrawal_rate: 10%,
}

Output: ✓ Withdrew 8% < 10% limit
Privacy: Exact withdrawal amounts hidden
```

#### 9. **Transaction Velocity Circuit**
**Purpose**: Prove number of transactions within limits

**Use Case**: Maximum 50 trades per day (prevent wash trading)

```rust
Circuit: TransactionVelocityCircuit {
    // Private inputs
    transaction_count_24h: 35,

    // Public constraints
    max_transactions_daily: 50,
}

Output: ✓ 35 transactions < 50 limit
```

---

### 📈 **Risk Management**

#### 10. **Leverage Ratio Circuit**
**Purpose**: Prove leverage doesn't exceed maximum

**Use Case**: Total leverage ≤ 2x (for margin trading funds)

```rust
Circuit: LeverageRatioCircuit {
    // Private inputs
    total_assets: $100M,
    borrowed_assets: $50M,
    equity: $50M,

    // Public constraints
    max_leverage: 2.0,
}

Computation:
- leverage = (total_assets) / equity
- Prove: leverage ≤ max_leverage

Output: ✓ Leverage = 2.0x ≤ 2.0x limit
Privacy: Asset and debt amounts hidden
```

**Complexity**: ~100 constraints (division required)

#### 11. **Value at Risk (VaR) Circuit**
**Purpose**: Prove portfolio risk within acceptable range

**Use Case**: 95% VaR ≤ 5% of portfolio

```rust
Circuit: VaRCircuit {
    // Private inputs
    asset_values: [$35M, $30M, $25M, $10M],
    asset_volatilities: [0.20, 0.18, 0.25, 0.02],
    correlations: [[1.0, 0.6, ...], ...],

    // Public constraints
    confidence_level: 95%,
    max_var_percentage: 5%,
}

Output: ✓ 95% VaR = 4.2% < 5% limit
Privacy: Holdings and risk metrics hidden
```

**Complexity**: ~500+ constraints (complex calculations)

#### 12. **Drawdown Limit Circuit**
**Purpose**: Prove losses don't exceed maximum drawdown

**Use Case**: Maximum 20% loss from peak

```rust
Circuit: DrawdownCircuit {
    // Private inputs
    peak_portfolio_value: $120M,
    current_portfolio_value: $100M,

    // Public constraints
    max_drawdown_percentage: 20%,
}

Computation:
- drawdown = (peak - current) / peak
- Prove: drawdown ≤ max_drawdown

Output: ✓ Drawdown = 16.7% < 20% limit
```

---

### 🔒 **Regulatory Compliance**

#### 13. **Accredited Investor Circuit**
**Purpose**: Prove investor meets accreditation requirements

**Use Case**: Minimum $1M net worth or $200K income

```rust
Circuit: AccreditedInvestorCircuit {
    // Private inputs
    net_worth: $2.5M,        // Hidden
    annual_income: $150K,    // Hidden

    // Public constraints
    min_net_worth: $1M,
    min_income: $200K,
}

Output: ✓ Investor accredited (net worth criterion met)
Privacy: Exact net worth/income hidden
```

#### 14. **AML Transaction Threshold Circuit**
**Purpose**: Prove transactions flagged if exceeding AML thresholds

**Use Case**: Flag if single transaction ≥ $10K

```rust
Circuit: AMLThresholdCircuit {
    // Private inputs
    transaction_amount: $8500,

    // Public constraints
    aml_threshold: $10000,

    // Output (public)
    requires_reporting: false,
}

Output: Transaction = $8.5K < $10K (no reporting needed)
Privacy: Exact amount hidden if below threshold
```

#### 15. **Diversification Requirement Circuit** (REG D)
**Purpose**: Prove fund meets SEC diversification rules

**Use Case**: 50% in 5+ issuers, no more than 25% in one issuer

```rust
Circuit: RegDDiversificationCircuit {
    // Private inputs
    issuer_allocations: [$30M, $25M, $20M, $15M, $10M],

    // Public constraints (SEC Reg D)
    min_issuers_for_50pct: 5,
    max_single_issuer: 25%,
}

Output: ✓ Meets SEC Regulation D diversification
```

---

### 💵 **Fee & Performance**

#### 16. **Fee Structure Compliance Circuit**
**Purpose**: Prove fees charged match stated structure

**Use Case**: 2% management fee + 20% performance fee

```rust
Circuit: FeeComplianceCircuit {
    // Private inputs
    starting_aum: $100M,
    ending_aum: $110M,
    fees_charged: $2.2M,

    // Public constraints
    management_fee_rate: 2%,
    performance_fee_rate: 20%,
}

Computation:
- expected_mgmt_fee = starting_aum × 2% = $2M
- gains = ending_aum - starting_aum = $10M
- expected_perf_fee = gains × 20% = $2M
- expected_total = $2M + $2M = $4M

// But circuit could prove alternative structures
Output: ✓ Fees calculated correctly
```

#### 17. **Performance Range Circuit**
**Purpose**: Prove returns within claimed range (without exact value)

**Use Case**: Prove "returned 8-12%" without revealing exact 9.7%

```rust
Circuit: PerformanceRangeCircuit {
    // Private inputs
    starting_value: $100M,
    ending_value: $109.7M,

    // Public constraints
    min_return: 8%,
    max_return: 12%,
}

Computation:
- return_pct = (ending - starting) / starting × 100
- Prove: min_return ≤ return_pct ≤ max_return

Output: ✓ Returns within stated 8-12% range
Privacy: Exact 9.7% return hidden
```

---

### ⏱️ **Time-Based Controls**

#### 18. **Lock-up Period Circuit**
**Purpose**: Prove investor cannot withdraw during lock-up

**Use Case**: 12-month lock-up from investment date

```rust
Circuit: LockupPeriodCircuit {
    // Private inputs
    investment_timestamp: 1672531200,  // Jan 1, 2023
    current_timestamp: 1704067200,     // Jan 1, 2024

    // Public constraints
    lockup_period_days: 365,
}

Computation:
- days_elapsed = (current - investment) / 86400
- Prove: days_elapsed ≥ lockup_period_days

Output: ✓ Lock-up period completed (can withdraw)
```

#### 19. **Rebalancing Frequency Circuit**
**Purpose**: Prove minimum time between rebalances

**Use Case**: Minimum 24 hours between portfolio rebalances

```rust
Circuit: RebalancingFrequencyCircuit {
    // Private inputs
    last_rebalance_timestamp: 1699920000,
    current_timestamp: 1700006400,

    // Public constraints
    min_hours_between_rebalances: 24,
}

Output: ✓ 24 hours elapsed since last rebalance
```

---

### 🔗 **Cross-Asset Constraints**

#### 20. **Correlation Limit Circuit**
**Purpose**: Prove correlated assets don't exceed concentration

**Use Case**: Assets with correlation > 0.8 limited to 30% combined

```rust
Circuit: CorrelationLimitCircuit {
    // Private inputs
    asset_a_value: $20M,
    asset_b_value: $15M,
    correlation: 0.85,
    portfolio_total: $100M,

    // Public constraints
    correlation_threshold: 0.8,
    max_combined_percentage: 30%,
}

Output: ✓ Correlated assets = 35% total
⚠️  Warning: Exceeds 30% limit
```

#### 21. **Currency Exposure Circuit**
**Purpose**: Prove foreign currency exposure within limits

**Use Case**: Maximum 40% in non-USD assets

```rust
Circuit: CurrencyExposureCircuit {
    // Private inputs
    usd_assets: $65M,
    eur_assets: $20M,
    gbp_assets: $10M,
    jpy_assets: $5M,

    // Public constraints
    max_non_usd_percentage: 40%,
}

Output: ✓ Non-USD exposure = 35% < 40%
```

---

## 🏗️ **Implementation Complexity**

| Circuit | Constraints | Proving Time | Use Case Priority |
|---------|-------------|--------------|-------------------|
| **Asset Class Allocation** | ~150 | ~100ms | High |
| **Daily Spending Limit** | ~50/tx | ~200ms | High |
| **Leverage Ratio** | ~100 | ~150ms | Medium |
| **Withdrawal Rate** | ~80 | ~100ms | High |
| **Performance Range** | ~120 | ~150ms | Medium |
| **Lock-up Period** | ~30 | ~50ms | High |
| **Accredited Investor** | ~100 | ~100ms | Low (off-chain) |
| **VaR Calculation** | ~500+ | ~800ms | Low (complex) |
| **Correlation Limit** | ~200 | ~300ms | Medium |

---

## 💡 **Practical Examples**

### **Example 1: Hedge Fund Compliance Suite**
```rust
ProveBundle {
    circuits: [
        PositionLimit { max: 40% },
        LiquidityReserve { min: 10% },
        LeverageRatio { max: 3.0 },
        DailySpending { limit: $10M },
        Drawdown { max: 25% },
    ]
}

// Single recursive proof covers all constraints
// Gas cost: ~1.2M gas (amortized over 5 checks)
// Privacy: All balances, leverage, and flows hidden
```

### **Example 2: Pension Fund Quarterly Report**
```rust
ProveBundle {
    circuits: [
        AssetClassAllocation {
            stock_range: (55%, 65%),
            bond_range: (25%, 35%),
            cash_range: (8%, 12%),
        },
        GeographicDiversification {
            max_emerging: 30%,
            max_country: 20%,
        },
        PerformanceRange {
            min_return: 5%,
            max_return: 8%,
        },
    ]
}

// Quarterly proof to regulators
// Privacy: Exact allocations and returns hidden
// Compliance: Proven cryptographically
```

### **Example 3: DAO Treasury Management**
```rust
ProveBundle {
    circuits: [
        DailySpending { limit: $5M },
        TransactionVelocity { max_tx: 50 },
        WhitelistCheck { approved_assets_only: true },
        RebalancingFrequency { min_hours: 24 },
    ]
}

// On-chain DAO governance
// Privacy: Individual transactions hidden
// Transparency: Compliance publicly verifiable
```

---

## 🎯 **Next Steps: Priority Implementation**

### **Phase 3: High-Value Additions** (2-3 weeks)
1. ✅ Asset Class Allocation Circuit
2. ✅ Daily Spending Limit Circuit
3. ✅ Withdrawal Rate Circuit
4. ✅ Lock-up Period Circuit

### **Phase 4: Advanced Features** (1-2 months)
1. Leverage Ratio Circuit
2. Performance Range Circuit
3. Geographic Diversification
4. Correlation Limit Circuit

### **Phase 5: Enterprise** (3+ months)
1. Value at Risk (VaR) Circuit
2. AML/KYC Integration Circuits
3. Multi-party computation circuits
4. Cross-chain compliance proofs

---

## 📚 **Resources**

- **Current Implementation**: `circuits/src/`
- **Range Proof Primitives**: `circuits/src/range_proof.rs`
- **Nova Integration**: `sonobe/examples/fund_compliance_full_flow.rs`
- **Deployment Guide**: `DEPLOYMENT.md`

---

## 🤝 **Contributing**

To add a new circuit:

1. Create circuit file: `circuits/src/your_circuit.rs`
2. Implement `Circuit<F>` trait (Bellpepper)
3. Add unit tests (prove/verify)
4. Add Nova integration test
5. Generate Solidity verifier (Sonobe)
6. Deploy to Arc testnet

See existing circuits for examples.

---

**The sky is the limit with zero-knowledge proofs for fund compliance!** 🚀

Every property that can be expressed as a mathematical constraint can be proven in zero-knowledge, enabling complete privacy while maintaining regulatory compliance.
