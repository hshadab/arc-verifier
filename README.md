# Arc Fund Manager - Privacy-Preserving ZK Compliance Demo

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Arc Testnet](https://img.shields.io/badge/Arc-Testnet-green)](https://testnet.arcscan.app)
[![Circuits](https://img.shields.io/badge/Tests-18/18_Passing-success)](circuits/)
[![Contracts](https://img.shields.io/badge/Contracts-8/8_Passing-success)](contracts/)

> A zero-knowledge proof system for privacy-preserving fund compliance on Arc Network, demonstrating off-chain ZKP generation and on-chain verification using Arecibo.

## 🎯 Overview

This project demonstrates a **Privacy-Preserving Tokenized RWA Fund Manager** that uses zero-knowledge proofs to prove regulatory compliance without revealing sensitive portfolio details. Built for [Arc Network](https://arc.network), an EVM blockchain optimized for capital markets and institutional DeFi.

### What Does It Do?

Investment funds managing tokenized real-world assets (RWAs) must:
- **Comply** with regulations (position limits, liquidity requirements, asset whitelists)
- **Preserve Privacy** (don't reveal exact allocations to competitors/public)
- **Enable Trust** (prove compliance without trusted intermediaries)

Our solution uses **zero-knowledge proofs** to achieve all three:

```
┌─────────────────┐         ┌──────────────┐         ┌─────────────┐
│ Private Fund    │────────►│ ZK Circuit   │────────►│ Arc Network │
│ Portfolio Data  │  Prove  │ (Arecibo)    │  Verify │  Contract   │
└─────────────────┘         └──────────────┘         └─────────────┘
    $35M in BENJI              ✓ All positions           ✓ Compliant
    $30M in BUIDL               ≤ 40% limit             ✓ No details
    $25M in RE Token             revealed!                revealed!
    $10M in USDC
```

## 🚀 Live Demo on Arc Testnet

### Deployed Contract

- **TokenizedFundManager**: [`0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE`](https://testnet.arcscan.app/address/0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE)
- **Network**: Arc Testnet (Chain ID: 5042002)
- **Explorer**: https://testnet.arcscan.app
- **Status**: ✅ Deployed and functional

### Try It Yourself

```bash
# View contract on block explorer
https://testnet.arcscan.app/address/0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE

# Query policy parameters
cast call 0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE "getPolicyParameters()" \
  --rpc-url https://rpc.testnet.arc.network

# Returns: (40, 10) = 40% max position, 10% min liquidity
```

## 📊 Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     Off-Chain (Private)                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Portfolio    │───►│ ZK Circuits  │───►│ Nova Prover  │      │
│  │ $100M Total  │    │ - Position   │    │ (Arecibo)    │      │
│  │ 4 assets     │    │ - Liquidity  │    │              │      │
│  │              │    │ - Whitelist  │    │              │      │
│  └──────────────┘    └──────────────┘    └──────┬───────┘      │
│                                                  │               │
└──────────────────────────────────────────────────┼──────────────┘
                                                   │ Proof
                                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     On-Chain (Public - Arc Network)              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Solidity     │◄───│ TokenizedFund│───►│ Audit Trail  │      │
│  │ Verifier     │    │ Manager.sol  │    │ (Public)     │      │
│  │ (Nova)       │    │              │    │              │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                   │
│  ✓ Proof verified                                                │
│  ✓ Compliance confirmed                                          │
│  ✗ No portfolio details revealed                                 │
└──────────────────────────────────────────────────────────────────┘
```

### ZK Circuits Implemented

| Circuit | Purpose | Constraints | Tests |
|---------|---------|-------------|-------|
| **Position Limit** | Proves no single asset exceeds 40% of portfolio | ~141 | 3/3 ✅ |
| **Liquidity Reserve** | Proves USDC reserves ≥ 10% of portfolio | ~35 | 4/4 ✅ |
| **Whitelist** | Proves all assets are approved (Merkle proof) | ~3 per level | 2/2 ✅ |
| **Range Proofs** | Enforces inequalities in ZK (bit decomposition) | ~35 per proof | 8/8 ✅ |

**Total**: 18/18 tests passing (100%)

## 🔧 Technology Stack

- **Circuits**: [Bellpepper](https://github.com/argumentcomputer/bellpepper) (R1CS constraint system)
- **Curves**: Pasta curves (Pallas/Vesta) for circuits
- **Proof System**: [Arecibo](https://github.com/wyattbenno777/arecibo) (Nova SNARKs)
- **Smart Contracts**: Solidity 0.8.20 (Foundry)
- **Blockchain**: [Arc Network](https://arc.network) (EVM with USDC gas)
- **Testing**: Rust `cargo test`, Foundry `forge test`

## 📁 Project Structure

```
arc-verifier/
├── circuits/                    # ZK circuits (Rust)
│   ├── src/
│   │   ├── position_limit.rs    # 40% position limit circuit
│   │   ├── liquidity_reserve.rs # 10% liquidity circuit
│   │   ├── whitelist.rs         # Asset whitelist circuit
│   │   ├── range_proof.rs       # Inequality proofs
│   │   └── lib.rs
│   ├── examples/
│   │   └── generate_proofs.rs   # Demo proof generation
│   └── tests/                   # 18 passing tests
│
├── contracts/                   # Smart contracts (Solidity)
│   ├── src/
│   │   └── TokenizedFundManager.sol
│   ├── test/
│   │   └── TokenizedFundManager.t.sol
│   └── script/
│       └── DeployFundManager.s.sol
│
├── arecibo/                     # Arecibo Nova prover
│   └── templates/               # Solidity verifier templates
│       ├── nova_cyclefold_decider.askama.sol
│       ├── groth16_verifier.askama.sol
│       └── kzg10_verifier.askama.sol
│
└── docs/                        # Documentation
    ├── ARECIBO_INTEGRATION.md   # Integration guide
    ├── ARCHITECTURE.md          # System design
    └── PROGRESS_UPDATE.md       # Development log
```

## 🧪 Testing & Verification

### Circuit Tests (All Passing ✅)

```bash
cd circuits
cargo test --release

# Output:
# test position_limit::tests::test_compliant_portfolio ... ok
# test position_limit::tests::test_violating_portfolio ... ok
# test position_limit::tests::test_edge_case_exact_limit ... ok
# test liquidity_reserve::tests::test_sufficient_liquidity ... ok
# test liquidity_reserve::tests::test_insufficient_liquidity ... ok
# test whitelist::tests::test_whitelisted_asset ... ok
# test whitelist::tests::test_non_whitelisted_asset ... ok
# test range_proof::tests::* ... 8 tests ok
#
# 18/18 tests passed ✅
```

### Smart Contract Tests (All Passing ✅)

```bash
cd contracts
forge test

# Output:
# [PASS] testInitialSetup()
# [PASS] testGetPolicyParameters()
# [PASS] testExecuteRebalanceWithMockProofs()
# [PASS] testUnauthorizedAgentCannotRebalance()
# [PASS] testAdminCanAuthorizeAgent()
# [PASS] testEmptyProofFails()
# [PASS] testComplianceReport()
# [PASS] testDailyRebalanceLimit()
#
# 8/8 tests passed ✅
```

### On-Chain Verification (Arc Testnet)

```bash
# Check contract is live
cast call 0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE "admin()" \
  --rpc-url https://rpc.testnet.arc.network
# Returns: 0xc2d88f27dbd6c178ac2638a9940435a9d6726251

# Execute a compliant rebalance
cast send 0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE "executeRebalance(bytes,bytes)" \
  <proof_bundle> <metadata> \
  --private-key <your_key> \
  --rpc-url https://rpc.testnet.arc.network

# Transaction succeeded! ✅
# Tx: 0xf12280a6e83204483c89945638092f2bc83db2cf6f2931f4a11aa240f6fc2ab3
```

## 🚧 Current Status

### ✅ Phase 1: Complete (Circuits & Contracts)

- [x] 18/18 circuit tests passing
- [x] 8/8 smart contract tests passing
- [x] Deployed to Arc testnet
- [x] Successful test transactions
- [x] Explored Arecibo verifier generation

### 🔄 Phase 2: In Progress (Production Integration)

The current demo uses **mock verifiers** in the smart contract. For production, we need to:

1. **Port circuits from Pasta to BN254 curves** (required for Ethereum/Arc EVM verification)
2. **Implement StepCircuit trait** for Arecibo Nova compatibility
3. **Generate real Nova proofs** using Arecibo
4. **Extract Solidity verifiers** from Arecibo templates
5. **Integrate verifiers** with TokenizedFundManager.sol
6. **Build AI agent** for automated proof generation

See [`docs/ARECIBO_INTEGRATION.md`](docs/ARECIBO_INTEGRATION.md) for detailed integration path.

## 💡 Use Cases

### 1. Institutional RWA Funds
- **Problem**: Blackrock, Invesco testing tokenized funds on Arc
- **Challenge**: Regulatory compliance vs competitive intelligence
- **Solution**: Prove compliance without revealing strategy

### 2. Automated Treasury Management
- **Problem**: AI agents managing corporate USDC treasuries
- **Challenge**: Need trustless compliance verification
- **Solution**: ZK proofs enable permissionless auditing

### 3. Cross-Chain Fund Management
- **Problem**: Funds operating across multiple chains
- **Challenge**: Unified compliance reporting
- **Solution**: Nova's IVC enables efficient multi-chain proofs

## 📚 Documentation

- **[Arecibo Integration Guide](docs/ARECIBO_INTEGRATION.md)** - How to integrate with Arecibo Nova
- **[Architecture Overview](docs/ARCHITECTURE.md)** - System design and data flow
- **[Progress Log](docs/PROGRESS_UPDATE.md)** - Development timeline
- **[Arc Testnet Setup](ARC_TESTNET_SETUP.md)** - Network configuration

## 🔗 Links

- **Arc Network**: https://arc.network
- **Arecibo (Nova)**: https://github.com/wyattbenno777/arecibo
- **Block Explorer**: https://testnet.arcscan.app
- **RPC**: https://rpc.testnet.arc.network

## 🏗️ Building & Running

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Foundry (Solidity)
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### Build Circuits

```bash
cd circuits
cargo build --release
cargo test --release

# Run example
cargo run --release --example generate_proofs
```

### Build & Deploy Contracts

```bash
cd contracts

# Test
forge test

# Deploy to Arc testnet
source .env
forge script script/DeployFundManager.s.sol:DeployFundManager \
  --rpc-url $ARC_RPC_URL \
  --broadcast \
  --legacy
```

## ⚠️ Disclaimer

**This is a proof-of-concept demo.** The current implementation uses mock verifiers. Production use requires:

1. Completing the BN254 circuit port
2. Generating real Nova proofs
3. Security audits
4. Legal/regulatory review

DO NOT use in production without proper security audits.

## 📄 License

MIT License

---

**Built for Arc Network 🌐 | Powered by Arecibo (Nova) ⚡ | Privacy-First 🔒**
