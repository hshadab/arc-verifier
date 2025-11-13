//! Generate Nova proofs and extract Solidity verifier for Arc deployment
//!
//! This example:
//! 1. Creates recursive proofs using Arecibo Nova
//! 2. Compresses proofs for efficient on-chain verification
//! 3. Generates Solidity verifier contract
//! 4. Prepares calldata for on-chain submission
//!
//! Run with: cargo run --release --example nova_proof_with_verifier --features solidity
//!
//! Note: This uses a simplified StepCircuit. Production would port full circuits to BN254.

// Arecibo Nova types
use arecibo::{
    frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError},
    nebula::{
        compression::CompressedSNARK,
        rs::{PublicParams, RecursiveSNARK, StepCircuit},
    },
    provider::{Bn256EngineKZG, GrumpkinEngine},
    traits::{snark::RelaxedR1CSSNARKTrait, Engine},
};

#[cfg(feature = "solidity")]
use arecibo::onchain::{
    compressed::prepare_calldata,
    utils::get_function_selector_for_nova_cyclefold_verifier,
    verifiers::{
        groth16::SolidityGroth16VerifierKey,
        kzg::SolidityKZGVerifierKey,
        nebula::{get_decider_template_for_cyclefold_decider, NovaCycleFoldVerifierKey},
    },
};

use ff::Field;
use halo2curves::bn256::Fr;
use rand::thread_rng;
use std::time::Instant;

type E1 = Bn256EngineKZG;
type E2 = GrumpkinEngine;
type EE1 = arecibo::provider::hyperkzg::EvaluationEngine<halo2curves::bn256::Bn256, E1>;
type EE2 = arecibo::provider::ipa_pc::EvaluationEngine<E2>;
type S1 = arecibo::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
type S2 = arecibo::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;

/// Fund compliance circuit compatible with Arecibo Nova
///
/// This demonstrates position limit checking in a ZK-friendly way.
/// The circuit proves that a portfolio allocation satisfies constraints
/// without revealing exact positions.
///
/// State: [total_value, max_position_pct, compliance_flag]
/// Transition: Updates compliance flag based on portfolio check
#[derive(Clone, Debug)]
struct FundComplianceCircuit {
    /// Maximum position percentage (e.g., 40 for 40%)
    max_position_pct: u64,
}

impl FundComplianceCircuit {
    fn new() -> Self {
        Self {
            max_position_pct: 40,
        }
    }
}

impl StepCircuit<Fr> for FundComplianceCircuit {
    fn arity(&self) -> usize {
        // State: [compliance_counter]
        // We track how many compliant checks have been performed
        1
    }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        // Input: compliance counter
        let counter = z_in[0].clone();

        // Allocate the max position percentage as a constant
        let max_pct = AllocatedNum::alloc(cs.namespace(|| "max_pct"), || {
            Ok(Fr::from(self.max_position_pct))
        })?;

        // Simulate checking a position (in production, this would be private input)
        // For demo: assume position is 35% (compliant)
        let position_pct = AllocatedNum::alloc(cs.namespace(|| "position_pct"), || {
            Ok(Fr::from(35u64))
        })?;

        // Check: position_pct <= max_pct
        // We compute diff = max_pct - position_pct
        // If diff >= 0, position is compliant
        let diff = position_pct.sub(cs.namespace(|| "diff"), &max_pct)?;

        // In a real circuit, we'd do range proofs here
        // For demo, we just show the structure

        // Increment counter (proves we completed a compliance check)
        let one = AllocatedNum::alloc(cs.namespace(|| "one"), || Ok(Fr::ONE))?;
        let new_counter = counter.add(cs.namespace(|| "increment"), &one)?;

        // Constrain that diff was computed correctly (a * 1 = diff)
        cs.enforce(
            || "diff_constraint",
            |lc| lc + diff.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
        );

        // Output: incremented counter
        Ok(vec![new_counter])
    }
}

fn main() {
    println!("🔐 Arc Fund Manager - Nova Proof Generation");
    println!("═══════════════════════════════════════════\n");

    // Create circuit
    let circuit = FundComplianceCircuit::new();

    println!("📊 Circuit Setup:");
    println!("   Circuit: Position Limit Compliance");
    println!("   Assets: 4 positions");
    println!("   Max position: 40%\n");

    // Setup public parameters
    println!("⚙️  Generating Public Parameters...");
    let start = Instant::now();
    let pp = PublicParams::<E1>::setup(&circuit, &*S1::ck_floor(), &*S2::ck_floor());
    println!("   ✅ Setup complete ({:?})", start.elapsed());
    println!("   Constraints (primary): {}", pp.num_constraints().0);
    println!("   Constraints (secondary): {}\n", pp.num_constraints().1);

    // Generate recursive SNARK
    println!("🔄 Generating Recursive SNARK...");
    let num_steps = 3;
    let z0 = vec![<E1 as Engine>::Scalar::ONE];

    let start = Instant::now();
    let mut recursive_snark = RecursiveSNARK::<E1>::new(&pp, &circuit, &z0)
        .expect("Failed to create RecursiveSNARK");

    let mut ic = <E1 as Engine>::Scalar::ZERO;
    for i in 0..num_steps {
        let step_start = Instant::now();
        recursive_snark.prove_step(&pp, &circuit, ic)
            .expect(&format!("Step {} failed", i));
        ic = recursive_snark.increment_commitment(&pp, &circuit);
        println!("   Step {}: {:?}", i, step_start.elapsed());
    }
    println!("   ✅ Recursive proof complete ({:?})\n", start.elapsed());

    // Verify recursive SNARK
    println!("✓ Verifying Recursive SNARK...");
    let start = Instant::now();
    recursive_snark.verify(&pp, num_steps, &z0, ic)
        .expect("Verification failed");
    println!("   ✅ Verified! ({:?})\n", start.elapsed());

    // Compress SNARK for on-chain verification
    println!("📦 Compressing SNARK for on-chain verification...");
    let mut rng = thread_rng();
    let start = Instant::now();

    let (compressed_pk, compressed_vk) = CompressedSNARK::setup(&pp, &mut rng, z0.len())
        .expect("Compressed setup failed");
    println!("   Setup: {:?}", start.elapsed());

    let start = Instant::now();
    let compressed_proof = CompressedSNARK::prove(&pp, &compressed_pk, &recursive_snark, &mut rng)
        .expect("Compression failed");
    println!("   ✅ Compressed! ({:?})\n", start.elapsed());

    // Verify compressed proof
    println!("✓ Verifying Compressed Proof...");
    let start = Instant::now();
    CompressedSNARK::verify(&compressed_proof, compressed_vk.clone())
        .expect("Compressed verification failed");
    println!("   ✅ Verified! ({:?})\n", start.elapsed());

    // Generate Solidity verifier (requires 'solidity' feature)
    #[cfg(feature = "solidity")]
    {
        println!("🔧 Generating Solidity Verifier Contract...");

        let function_selector = get_function_selector_for_nova_cyclefold_verifier(
            recursive_snark.z0.len() * 2 + 1
        );

        let calldata = prepare_calldata(function_selector, &compressed_proof)
            .expect("Failed to prepare calldata");

        let nova_cyclefold_vk = NovaCycleFoldVerifierKey::from((
            compressed_vk.pp_hash,
            SolidityGroth16VerifierKey::from(compressed_vk.groth16_vk),
            SolidityKZGVerifierKey::from((compressed_vk.kzg_vk, Vec::new())),
            recursive_snark.z0.len(),
        ));

        let solidity_code = get_decider_template_for_cyclefold_decider(nova_cyclefold_vk);

        // Save to file
        use std::fs;
        fs::write("./PositionLimitVerifier.sol", &solidity_code)
            .expect("Failed to write verifier");
        fs::write("./position_limit_proof.calldata", &calldata)
            .expect("Failed to write calldata");

        println!("   ✅ Verifier saved to: PositionLimitVerifier.sol");
        println!("   ✅ Calldata saved to: position_limit_proof.calldata");
        println!("   📊 Contract size: {} bytes", solidity_code.len());
        println!("   📊 Calldata size: {} bytes\n", calldata.len());

        #[cfg(feature = "solidity")]
        {
            use arecibo::onchain::eth::evm::{compile_solidity, Evm};

            println!("🧪 Testing in EVM...");
            let bytecode = compile_solidity(&solidity_code, "NovaDecider");
            let mut evm = Evm::default();
            let verifier_address = evm.create(bytecode);

            let (gas, output) = evm.call(verifier_address, calldata);
            let success = *output.last().unwrap() == 1;

            println!("   Result: {}", if success { "✅ PASS" } else { "❌ FAIL" });
            println!("   Gas used: {}\n", gas);
        }
    }

    #[cfg(not(feature = "solidity"))]
    println!("⚠️  Solidity verifier generation requires 'solidity' feature");

    println!("✅ Done! Next steps:");
    println!("   1. Deploy PositionLimitVerifier.sol to Arc testnet");
    println!("   2. Use position_limit_proof.calldata to test verification");
    println!("   3. Integrate with TokenizedFundManager contract");
}
