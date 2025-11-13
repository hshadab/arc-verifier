# Architecture: Privacy-Preserving Tokenized Fund Manager

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   FUND GOVERNANCE                            │
│               (On-chain Policy Contract)                     │
│                                                              │
│  • Max 40% in any single asset                              │
│  • Min 10% USDC liquidity                                   │
│  • Only approved assets (whitelist)                         │
│  • Investment grade securities only                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ↓
┌─────────────────────────────────────────────────────────────┐
│              AI FUND MANAGER AGENT                          │
│                  (Off-chain)                                 │
│                                                              │
│  Inputs:                                                     │
│  • Market data (yields, prices, volatility)                 │
│  • Current portfolio state (encrypted)                      │
│  • Investment mandates                                      │
│                                                              │
│  Processing:                                                 │
│  1. Analyze opportunities                                   │
│  2. Make rebalancing decisions                              │
│  3. Generate ZK proofs for each constraint:                 │
│     ├─→ Position Limit Proof                                │
│     ├─→ Liquidity Reserve Proof                             │
│     └─→ Whitelist Membership Proof                          │
│                                                              │
│  Output:                                                     │
│  • Encrypted transaction                                    │
│  • Bundle of ZK proofs                                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ↓
┌─────────────────────────────────────────────────────────────┐
│           FUND SMART CONTRACT (Arc EVM)                     │
│                                                              │
│  function executeRebalance(                                 │
│      bytes encryptedTx,                                     │
│      bytes proofBundle                                      │
│  ) {                                                        │
│      // Verify all compliance proofs                        │
│      require(verifier.verifyPositionLimit(proof1));        │
│      require(verifier.verifyLiquidity(proof2));            │
│      require(verifier.verifyWhitelist(proof3));            │
│                                                              │
│      // Execute if all proofs valid                         │
│      executeTrade(encryptedTx);                            │
│      emit RebalanceExecuted(hash, timestamp);              │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. ZK Proof Circuits (Arecibo/Nova)

#### Position Limit Circuit
**Purpose:** Prove no single asset exceeds portfolio concentration limit

**Public Inputs:**
- `max_position_percentage`: Maximum allowed % (e.g., 40)

**Private Inputs:**
- `asset_values[]`: Value of each asset in portfolio
- `total_portfolio_value`: Sum of all assets

**Constraints:**
- For each asset `i`: `(asset[i] * 100) / total <= max_percentage`
- Sum of assets equals total (integrity check)

**Output:** Proof that all positions are within limits

---

#### Liquidity Reserve Circuit
**Purpose:** Prove sufficient USDC liquidity maintained

**Public Inputs:**
- `min_liquidity_percentage`: Minimum required % (e.g., 10)

**Private Inputs:**
- `usdc_balance`: Current USDC holdings
- `total_portfolio_value`: Total fund value

**Constraints:**
- `(usdc_balance * 100) / total >= min_liquidity_percentage`

**Output:** Proof that liquidity requirements met

---

#### Whitelist Circuit (Merkle Proof)
**Purpose:** Prove asset is in approved list without revealing which one

**Public Inputs:**
- `merkle_root`: Root of approved assets tree

**Private Inputs:**
- `asset_hash`: Hash of asset address
- `merkle_path[]`: Sibling hashes up to root
- `path_indices[]`: Left/right directions

**Constraints:**
- Recompute root from leaf using path
- Computed root equals public root

**Output:** Proof that asset is whitelisted

---

### 2. Smart Contract Architecture

```solidity
// Fund governance and execution
contract TokenizedFundManager {
    // Policy parameters
    uint256 constant MAX_SINGLE_POSITION = 40;  // 40%
    uint256 constant MIN_LIQUIDITY = 10;        // 10%
    bytes32 public assetWhitelistRoot;

    // Verifier contracts (deployed Arecibo verifiers)
    IPositionLimitVerifier positionVerifier;
    ILiquidityVerifier liquidityVerifier;
    IWhitelistVerifier whitelistVerifier;

    // State
    mapping(bytes32 => bool) public executedTrades;

    // Execute verified rebalancing
    function executeRebalance(
        address to,
        uint256 amount,
        PositionLimitProof memory proof1,
        LiquidityProof memory proof2,
        WhitelistProof memory proof3
    ) external onlyAgent {
        // Verify position limit
        require(
            positionVerifier.verify(proof1, MAX_SINGLE_POSITION),
            "Position limit violated"
        );

        // Verify liquidity
        require(
            liquidityVerifier.verify(proof2, MIN_LIQUIDITY),
            "Insufficient liquidity"
        );

        // Verify whitelist
        require(
            whitelistVerifier.verify(proof3, assetWhitelistRoot),
            "Asset not whitelisted"
        );

        // Execute trade
        IERC20(USDC).transfer(to, amount);

        emit TradeExecuted(to, amount, block.timestamp);
    }
}
```

---

### 3. AI Agent Implementation

```rust
struct TreasuryAgent {
    arecibo_params: PublicParams<PallasEngine>,
    policy: FundPolicy,
    wallet: Arc<Wallet>,
}

impl TreasuryAgent {
    async fn execute_rebalancing(&self) -> Result<()> {
        // 1. Analyze market and find opportunity
        let opportunity = self.analyze_markets().await?;

        // 2. Validate against policy
        if !self.validate_opportunity(&opportunity) {
            return Err("Opportunity violates policy");
        }

        // 3. Generate ZK proofs
        let position_proof = self.prove_position_limit(&opportunity)?;
        let liquidity_proof = self.prove_liquidity(&opportunity)?;
        let whitelist_proof = self.prove_whitelist(&opportunity)?;

        // 4. Submit to Arc blockchain
        let tx = self.build_transaction(
            opportunity,
            position_proof,
            liquidity_proof,
            whitelist_proof,
        );

        let receipt = self.submit_to_arc(tx).await?;

        Ok(())
    }

    fn prove_position_limit(&self, opp: &Opportunity) -> Result<Proof> {
        // Create circuit with private witness
        let circuit = PositionLimitCircuit::new(
            40, // max percentage
            self.get_asset_values(),
            self.get_total_value(),
        );

        // Generate Nova proof
        let proof = self.generate_nova_proof(circuit)?;
        Ok(proof)
    }
}
```

---

## Data Flow

### Rebalancing Workflow

1. **Market Analysis** (Off-chain)
   ```
   Agent monitors:
   - Tokenized RWA yields
   - Market conditions
   - Portfolio drift
   ```

2. **Decision Making** (Off-chain)
   ```
   Agent decides:
   - Which asset to buy/sell
   - Amount to trade
   - Expected new allocation
   ```

3. **Compliance Checking** (Off-chain ZK)
   ```
   Generate proofs that new state:
   ✓ Respects position limits
   ✓ Maintains liquidity
   ✓ Uses whitelisted assets
   ```

4. **Submission** (On-chain)
   ```
   Transaction includes:
   - Encrypted trade details
   - ZK proof bundle
   - Agent signature
   ```

5. **Verification** (On-chain)
   ```
   Smart contract:
   ✓ Verifies all proofs
   ✓ Executes trade
   ✓ Emits event
   ```

6. **Settlement** (On-chain)
   ```
   USDC settlement on Arc:
   - Sub-second finality
   - Predictable gas costs
   - Atomic execution
   ```

---

## Privacy Properties

### What's Hidden:
- ❌ Exact portfolio positions
- ❌ Specific asset allocations
- ❌ Trading strategy/signals
- ❌ Future investment plans
- ❌ Individual investor allocations

### What's Proven:
- ✅ All positions within limits
- ✅ Liquidity requirements met
- ✅ Only approved assets used
- ✅ Policy compliance maintained

### What's Public:
- ✅ Transaction occurred (hash)
- ✅ Timestamp
- ✅ Proof verified successfully

---

## Security Considerations

### Trusted Setup
- **Arecibo/Nova:** No trusted setup required (uses folding schemes)
- **Groth16 (if used for compression):** Requires ceremony

### Soundness
- Agent cannot create false proofs
- Circuits enforce all constraints
- Verifiers check cryptographic validity

### Completeness
- Valid operations always produce valid proofs
- No false rejections

### Privacy
- Zero-knowledge property: No information leaked beyond statement truth
- Verifier learns nothing about witness values

---

## Performance Characteristics

### Proof Generation (Off-chain)
- **Position Limit:** ~50ms per asset (estimated)
- **Liquidity:** ~30ms (estimated)
- **Whitelist:** ~100ms for depth-20 tree (estimated)

### Proof Verification (On-chain)
- **Gas Cost:** ~300k-500k per proof (estimated)
- **Arc Gas:** Paid in USDC (predictable)
- **Latency:** Sub-second finality

### Recursion (Nova IVC)
- Can accumulate proofs over time
- Constant verification cost regardless of steps
- Enables historical compliance proofs

---

## Deployment Architecture

```
Development:
├─ Local: Rust tests with test constraint system
├─ Integration: Full proof generation testing
└─ Simulation: Mock Arc environment

Testnet:
├─ Arc Testnet RPC
├─ Deployed verifier contracts
├─ Test USDC tokens
└─ Mock RWA tokens

Production:
├─ Arc Mainnet
├─ Audited contracts
├─ Real USDC
└─ Real tokenized RWAs (BENJI, BUIDL, etc.)
```

---

## Future Enhancements

1. **Range Proofs:** Add bit decomposition for inequality enforcement
2. **Poseidon Hash:** Replace simple addition with proper hash in Merkle trees
3. **Batch Verification:** Aggregate multiple proofs
4. **Historical Proofs:** Use Nova IVC for quarterly compliance reports
5. **Multi-Agent:** Coordinate multiple fund managers with ZK
6. **Cross-Chain:** Bridge proofs to other chains

---

**Related Documents:**
- [Current Status](./CURRENT_STATUS.md)
- [Main README](../README.md)
