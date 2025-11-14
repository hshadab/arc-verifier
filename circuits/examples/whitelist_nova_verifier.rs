//! Generate a Nova whitelist verifier (Poseidon Merkle) and calldata
//!
//! Run with: cargo run --release --example whitelist_nova_verifier --features solidity

use arecibo::{
    frontend::ConstraintSystem,
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

use arc_fund_circuits::nova_circuits::NovaWhitelistCircuit;
use halo2curves::bn256::Fr;
use rand::thread_rng;
use std::time::Instant;

type E1 = Bn256EngineKZG;
type E2 = GrumpkinEngine;
type EE1 = arecibo::provider::hyperkzg::EvaluationEngine<halo2curves::bn256::Bn256, E1>;
type EE2 = arecibo::provider::ipa_pc::EvaluationEngine<E2>;
type S1 = arecibo::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
type S2 = arecibo::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;

fn main() {
    println!("🔐 Nova Whitelist Verifier Generation");

    // Simple 3-level Merkle example with Poseidon hashing inside the circuit
    // Note: values here are toy u64 placeholders mapped into Fr
    let merkle_root = 123456789u64;
    let asset_hash = 42u64;
    let siblings = vec![111u64, 222u64, 333u64];
    let is_right = vec![false, true, false];
    let circuit = NovaWhitelistCircuit::new(merkle_root, asset_hash, siblings, is_right);

    // Setup public params for Nova
    let start = Instant::now();
    let pp = PublicParams::<E1>::setup(&circuit, &*S1::ck_floor(), &*S2::ck_floor());
    println!("Public params setup: {:?}", start.elapsed());
    println!("Constraints: {:?}", pp.num_constraints());

    // Create recursive SNARK with a few steps
    let num_steps = 2usize;
    let z0 = vec![Fr::ZERO];

    let start = Instant::now();
    let mut recursive_snark = RecursiveSNARK::<E1>::new(&pp, &circuit, &z0).expect("new recursive snark");
    let mut ic = <E1 as Engine>::Scalar::ZERO;
    for i in 0..num_steps {
        recursive_snark.prove_step(&pp, &circuit, ic).expect("prove step");
        ic = recursive_snark.increment_commitment(&pp, &circuit);
        println!("  step {} complete", i);
    }
    println!("Recursive proof: {:?}", start.elapsed());

    // Verify recursive
    recursive_snark.verify(&pp, num_steps, &z0, ic).expect("verify recursive");

    // Compress
    let mut rng = thread_rng();
    let (compressed_pk, compressed_vk) = CompressedSNARK::setup(&pp, &mut rng, z0.len()).expect("setup compressed");
    let compressed_proof = CompressedSNARK::prove(&pp, &compressed_pk, &recursive_snark, &mut rng).expect("compress");
    CompressedSNARK::verify(&compressed_proof, compressed_vk.clone()).expect("verify compressed");

    // Solidity artifacts (requires 'solidity' feature)
    #[cfg(feature = "solidity")]
    {
        let function_selector = get_function_selector_for_nova_cyclefold_verifier(recursive_snark.z0.len() * 2 + 1);
        let calldata = prepare_calldata(function_selector, &compressed_proof).expect("calldata");

        let nova_cyclefold_vk = NovaCycleFoldVerifierKey::from((
            compressed_vk.pp_hash,
            SolidityGroth16VerifierKey::from(compressed_vk.groth16_vk),
            SolidityKZGVerifierKey::from((compressed_vk.kzg_vk, Vec::new())),
            recursive_snark.z0.len(),
        ));

        let solidity_code = get_decider_template_for_cyclefold_decider(nova_cyclefold_vk);

        std::fs::write("./WhitelistVerifier.sol", &solidity_code).expect("write verifier");
        std::fs::write("./whitelist_proof.calldata", &calldata).expect("write calldata");
        println!("✅ Wrote WhitelistVerifier.sol and whitelist_proof.calldata");
    }
    #[cfg(not(feature = "solidity"))]
    println!("⚠️  Re-run with --features solidity to emit Solidity verifier and calldata");
}

