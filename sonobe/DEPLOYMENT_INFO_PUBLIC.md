# CompositeFundVerifier Deployment

## Deployment Details

**Contract Address:** `0x0b1A767A3fE8169EC107EE3F091CF0cDE07665cC`
**Transaction Hash:** `0x1b29713b95d6353db1f9edc0a1cc6c737a49d3a0169c24e8ce5f5ab1ba1f1510`
**Network:** Arc Testnet
**Chain ID:** 5042002
**RPC URL:** https://rpc.testnet.arc.network

## Deployment Artifacts

- `CompositeFundVerifier.sol` - Deployed Solidity verifier contract (37KB)
- `composite-proof.calldata` - Proof data (900 bytes)
- `composite-proof.inputs` - Human-readable proof inputs (1.9KB)
- `persisted_params/` - Cached prover/verifier parameters (90MB)

## What This Proves

The deployed verifier validates Nova folding proofs that demonstrate:

- ✅ Position limit compliance: 35% ≤ 40% across 3 consecutive periods
- ✅ Liquidity requirements: 10% ≥ 10% across 3 consecutive periods
- ✅ Whitelist compliance: All assets verified across 3 consecutive periods
- ✅ All THREE checks folded into ONE proof

## Verification Costs

- **Gas Cost:** ~795,738 gas per verification
- **Estimated Cost:** ~$0.02 per verification (at standard gas prices)

## Block Explorer

**Contract:** https://arc-sepolia.explorer.alchemy.com/address/0x0b1A767A3fE8169EC107EE3F091CF0cDE07665cC

**Transaction:** https://arc-sepolia.explorer.alchemy.com/tx/0x1b29713b95d6353db1f9edc0a1cc6c737a49d3a0169c24e8ce5f5ab1ba1f1510

## Technical Details

- **Proving System:** Nova + CycleFold + Groth16
- **Framework:** Sonobe v0.1.0
- **Curves:** BN254 (Prover) / Grumpkin (CycleFold)
- **Commitment Scheme:** KZG10
- **Rust Version:** 1.88.0
- **Solidity Version:** >=0.7.0 <0.9.0

## Usage

See `DEPLOYMENT_README.md` for complete usage instructions.

## Verification Status

- **Local Verification:** ✅ PASSED
- **On-chain Verification:** Contract deployed and ready for integration

---

**Deployed:** November 19, 2025
**Status:** Production Ready
