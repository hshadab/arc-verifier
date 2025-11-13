# Arecibo Integration Guide

## Overview

This document explains how Arecibo's proof system works and the path to integrate our fund compliance circuits with on-chain verifiers.

## Arecibo Architecture

### Proof Systems Available

Arecibo provides multiple proof systems:

1. **Nova** (`arecibo::nebula::rs`)
   - Recursive SNARK with IVC (Incrementally Verifiable Computation)
   - No trusted setup
   - Uses Pasta curves (Pallas/Vesta) or BN254
   - Best for iterative computations

2. **Compressed SNARK** (`arecibo::nebula::compression`)
   - Compresses Nova proofs for efficient on-chain verification
   - Uses Groth16 + KZG commitments
   - Outputs Solidity verifier contracts

3. **HyperNova** (`arecibo::hypernova`)
   - Extension of Nova with better prover time
   - Supports non-uniform computation

### Verifier Generation Workflow

Based on `/home/hshadab/arc-verifier/arecibo/src/onchain/test/full_flow.rs`:

```rust
// 1. Define a StepCircuit
impl StepCircuit<Fr> for MyCircuit {
    fn arity(&self) -> usize { 1 }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        // Circuit logic here
        Ok(vec![z_out])
    }

    fn non_deterministic_advice(&self) -> Vec<Fr> {
        vec![]
    }
}

// 2. Setup public parameters
let pp = PublicParams::<E1>::setup(&circuit, &*S1::ck_floor(), &*S2::ck_floor());

// 3. Generate recursive proof
let mut recursive_snark = RecursiveSNARK::<E1>::new(&pp, &circuit, &z0)?;
for _ in 0..num_steps {
    recursive_snark.prove_step(&pp, &circuit, ic)?;
    ic = recursive_snark.increment_commitment(&pp, &circuit);
}

// 4. Compress for on-chain verification
let (compressed_pk, compressed_vk) = CompressedSNARK::setup(&pp, &mut rng, z0.len())?;
let proof = CompressedSNARK::prove(&pp, &compressed_pk, &recursive_snark, &mut rng)?;

// 5. Generate Solidity verifier
let nova_cyclefold_vk = NovaCycleFoldVerifierKey::from((
    compressed_vk.pp_hash,
    SolidityGroth16VerifierKey::from(compressed_vk.groth16_vk),
    SolidityKZGVerifierKey::from((compressed_vk.kzg_vk, Vec::new())),
    z0.len(),
));

let solidity_code = get_decider_template_for_cyclefold_decider(nova_cyclefold_vk);

// 6. Prepare calldata
let function_selector = get_function_selector_for_nova_cyclefold_verifier(z0.len() * 2 + 1);
let calldata = prepare_calldata(function_selector, &proof)?;
```

## Verifier Template System

### Templates Location

`/home/hshadab/arc-verifier/arecibo/templates/`

1. **nova_cyclefold_decider.askama.sol**
   - Main Nova verifier template
   - Combines Groth16 + KZG10 verifiers
   - Uses Askama templating with placeholders:
     - `{{ pp_hash }}` - Public parameters hash
     - `{{ z_len }}` - Circuit state length
     - `{{ num_limbs }}` - Non-native field limbs
     - `{{ groth16_verifier }}` - Groth16 verifier code
     - `{{ kzg10_verifier }}` - KZG10 verifier code

2. **groth16_verifier.askama.sol**
   - Standard Groth16 SNARK verifier
   - From snarkJS templates
   - Placeholders:
     - `{{ vkey_alpha_g1 }}`, `{{ vkey_beta_g2 }}`, etc.
     - `{{ gamma_abc_g1 }}` - Verification key IC elements

3. **kzg10_verifier.askama.sol**
   - KZG polynomial commitment verifier
   - Verifies evaluation proofs
   - Uses BN254 pairing

### Generated Contract Structure

```solidity
// NovaDecider.sol (generated)
contract NovaDecider is Groth16Verifier, KZG10Verifier {
    function verifyNovaProof(
        uint256[z_len * 2 + 1] calldata i_z0_zi,
        uint256[4] calldata U_i_cmW_U_i_cmE,
        uint256[2] calldata u_i_cmW,
        uint256[3] calldata cmT_r,
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[4] calldata challenge_W_challenge_E_kzg_evals,
        uint256[2][2] calldata kzg_proof
    ) public view returns (bool) {
        // Verifies:
        // 1. KZG commitments for witness and error
        // 2. Groth16 proof of correct folding
        return true if all checks pass;
    }
}
```

## Integration Path for Our Circuits

### Current Status

✅ **Phase 1 Complete: Circuits Implemented**
- Position Limit Circuit: 3/3 tests passing
- Liquidity Reserve Circuit: 4/4 tests passing
- Whitelist Circuit: 2/2 tests passing
- Range Proofs: 8/8 tests passing
- **Total: 18/18 tests (100%)**

### Challenge: Curve Compatibility

Our circuits use **Pasta curves** (Pallas/Vesta with `Fp` field):
```rust
use pasta_curves::Fp;
use bellpepper_core::Circuit;

pub struct PositionLimitCircuit<F: PrimeField> {
    // Works with Pasta Fp
}
```

Arecibo's on-chain verifiers require **BN254 curves**:
```rust
use halo2curves::bn256::Fr;  // Different field!

impl StepCircuit<Fr> for MyCircuit {
    // Must use BN254 Fr field
}
```

**Why BN254?** Ethereum's `bn256Add`, `bn256Mul`, `bn256Pairing` precompiles are specifically for BN254. This enables efficient on-chain verification.

### Solution Paths

#### Option 1: Port Circuits to BN254 (Recommended for Production)

**Pros:**
- Full on-chain verification
- Trustless, permissionless
- No bridge required

**Cons:**
- Requires rewriting circuits for different field
- Additional testing needed

**Steps:**
1. Rewrite `PositionLimitCircuit`, `LiquidityReserveCircuit`, `WhitelistCircuit` using `halo2curves::bn256::Fr`
2. Implement `StepCircuit` trait instead of `Circuit`
3. Add `non_deterministic_advice()` method
4. Update range proofs to use BN254 field arithmetic
5. Test with Arecibo's Nova prover
6. Generate Solidity verifiers
7. Deploy to Arc testnet

**Example conversion:**
```rust
// Before (Pasta)
use pasta_curves::Fp;
use bellpepper_core::Circuit;

pub struct PositionLimitCircuit<F: PrimeField> {
    max_position_percentage: Option<u64>,
    asset_values: Vec<Option<F>>,
    total_portfolio_value: Option<F>,
}

impl<F: PrimeField> Circuit<F> for PositionLimitCircuit<F> {
    fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // ...
    }
}

// After (BN254)
use halo2curves::bn256::Fr;
use arecibo::{
    frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError},
    nebula::rs::StepCircuit,
};

#[derive(Clone)]
pub struct PositionLimitCircuit {
    max_position_percentage: u64,
    asset_values: Vec<u64>,
    total_portfolio_value: u64,
}

impl StepCircuit<Fr> for PositionLimitCircuit {
    fn arity(&self) -> usize {
        1  // Number of state elements
    }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        // Convert input state
        let compliance_flag = z_in[0].clone();

        // Allocate private inputs
        let max_pct = AllocatedNum::alloc(
            cs.namespace(|| "max_pct"),
            || Ok(Fr::from(self.max_position_percentage))
        )?;

        // Check each asset
        for (i, &asset_val) in self.asset_values.iter().enumerate() {
            let asset = AllocatedNum::alloc(
                cs.namespace(|| format!("asset_{}", i)),
                || Ok(Fr::from(asset_val))
            )?;

            // Percentage check logic...
            // Use range proofs...
        }

        // Output updated state
        Ok(vec![new_compliance_flag])
    }

    fn non_deterministic_advice(&self) -> Vec<Fr> {
        vec![]
    }
}
```

#### Option 2: Two-Layer Approach (Hybrid)

Use Pasta circuits + attestation bridge:

1. Generate proofs using Pasta circuits (current implementation)
2. Verify off-chain or on a Pasta-friendly chain
3. Submit attestation to Arc via bridge
4. Smart contract trusts attestation

**Pros:**
- Reuse existing circuits
- Faster development

**Cons:**
- Introduces trust assumptions
- More complex architecture

#### Option 3: MVP with Mock Verifiers (Current Demo Approach)

Deploy smart contracts with mock verifiers for demonstration:

1. Use existing `TokenizedFundManager.sol` with mock `_verifyPositionLimit()`, etc.
2. Deploy to Arc testnet
3. Test end-to-end flow with mock proofs
4. Demonstrate architecture and workflow
5. Note that production would use Option 1

**This is the fastest path to a working demo on Arc testnet.**

## Current Implementation Status

### Working Components

1. **Circuits (Pasta/Bellpepper)** ✅
   - `/home/hshadab/arc-verifier/circuits/src/position_limit.rs`
   - `/home/hshadab/arc-verifier/circuits/src/liquidity_reserve.rs`
   - `/home/hshadab/arc-verifier/circuits/src/whitelist.rs`
   - `/home/hshadab/arc-verifier/circuits/src/range_proof.rs`
   - All 18/18 tests passing

2. **Smart Contracts (Foundry)** ✅
   - `/home/hshadab/arc-verifier/contracts/src/TokenizedFundManager.sol`
   - `/home/hshadab/arc-verifier/contracts/test/TokenizedFundManager.t.sol`
   - All 8/8 tests passing
   - Ready for deployment with mock verifiers

3. **Arc Testnet Setup** ✅
   - Wallet: `0xc2d88f27DBd6c178AC2638a9940435a9D6726251`
   - Balance: 10 USDC
   - RPC: `https://rpc.testnet.arc.network`
   - Chain ID: 5042002

### Next Steps

**For MVP Demo (Fastest):**
1. Deploy `TokenizedFundManager.sol` to Arc testnet
2. Test with mock proofs
3. Document architecture
4. Note that real verifiers would come from Option 1

**For Production (Option 1):**
1. Port circuits to BN254/StepCircuit
2. Generate real Nova proofs
3. Extract Solidity verifiers
4. Integrate verifiers with TokenizedFundManager
5. Deploy to Arc testnet
6. Full end-to-end testing

## Arecibo API Notes

### Key Types

```rust
// Nova types
type E1 = Bn256EngineKZG;  // Primary curve
type E2 = GrumpkinEngine;  // Secondary curve for CycleFold

// Evaluation engines
type EE1 = arecibo::provider::hyperkzg::EvaluationEngine<Bn256, E1>;
type EE2 = arecibo::provider::ipa_pc::EvaluationEngine<E2>;

// SNARK types
type S1 = arecibo::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
type S2 = arecibo::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;
```

### Important Methods

```rust
// Public parameters
PublicParams::<E1>::setup(&circuit, &*S1::ck_floor(), &*S2::ck_floor())

// Recursive SNARK
RecursiveSNARK::<E1>::new(&pp, &circuit, &z0)
recursive_snark.prove_step(&pp, &circuit, ic)
recursive_snark.verify(&pp, num_steps, &z0, ic)

// Compression
CompressedSNARK::setup(&pp, &mut rng, z0.len())
CompressedSNARK::prove(&pp, &pk, &recursive_snark, &mut rng)
CompressedSNARK::verify(&proof, vk)

// Solidity generation
get_decider_template_for_cyclefold_decider(verifier_key)
```

## Resources

- **Arecibo GitHub**: https://github.com/wyattbenno777/arecibo (wyatt_dev branch)
- **Arecibo Templates**: `/home/hshadab/arc-verifier/arecibo/templates/`
- **Full Flow Test**: `/home/hshadab/arc-verifier/arecibo/src/onchain/test/full_flow.rs`
- **Arc Network Docs**: https://docs.arc.network/
- **Nova Paper**: https://eprint.iacr.org/2021/370

## Conclusion

We have successfully:
1. ✅ Implemented fund compliance circuits with range proofs
2. ✅ All tests passing (18/18)
3. ✅ Explored Arecibo's verifier generation system
4. ✅ Smart contracts ready for deployment

The main gap is porting circuits from Pasta to BN254 curves. For the MVP demo, we'll deploy with mock verifiers and document the production path.
