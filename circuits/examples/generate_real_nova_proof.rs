//! Generate real Nova proofs using Arecibo and extract Solidity verifier
//!
//! This example demonstrates the complete workflow:
//! 1. Setup public parameters
//! 2. Generate recursive SNARK proofs
//! 3. Compress proofs for on-chain verification
//! 4. Generate Solidity verifier contract
//! 5. Prepare calldata for Arc deployment
//!
//! Run with: cargo run --release --example generate_real_nova_proof --features solidity

use arc_fund_circuits::NovaLiquidityCircuit;
use arecibo::{
    nebula::rs::{PublicParams, RecursiveSNARK},
    onchain::compressed::CompressedSNARK,
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

use halo2curves::bn256::Fr;
use rand::thread_rng;
use std::time::Instant;
use ff::Field;

type E1 = Bn256EngineKZG;
type E2 = GrumpkinEngine;
type EE1 = arecibo::provider::hyperkzg::EvaluationEngine<halo2curves::bn256::Bn256, E1>;
type EE2 = arecibo::provider::ipa_pc::EvaluationEngine<E2>;
type S1 = arecibo::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
type S2 = arecibo::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;

fn main() {
    println!("🚀 Arc Fund Manager - Real Nova Proof Generation");
    println!("═══════════════════════════════════════════════════\n");

    // Example: $100M fund with $10M USDC (10% liquidity)
    let circuit = NovaLiquidityCircuit::new(
        10,           // min 10% required
        10_000_000,   // $10M USDC
        100_000_000,  // $100M total
    );

    println!("📊 Fund State:");
    println!("   Total Portfolio: $100M");
    println!("   USDC Balance: $10M");
    println!("   Liquidity: 10%");
    println!("   Requirement: ≥10%\n");

    // Setup public parameters
    println!("⚙️  Setting up Public Parameters...");
    let start = Instant::now();

    println!("   Creating commitment keys...");
    let s1_ck = S1::ck_floor();
    let s2_ck = S2::ck_floor();

    println!("   Generating public parameters...");
    let pp = PublicParams::<E1>::setup(&circuit, &*s1_ck, &*s2_ck);

    println!("   ✅ Setup complete ({:?})", start.elapsed());
    println!("   Primary constraints: {}", pp.num_constraints().0);
    println!("   Secondary constraints: {}\n", pp.num_constraints().1);

    // Generate recursive SNARK
    println!("🔄 Generating Recursive SNARK (Nova IVC)...");
    let num_steps = 3; // Prove 3 consecutive compliance checks
    let z0 = vec![Fr::zero()]; // Initial state: counter = 0

    let start = Instant::now();

    println!("   Creating initial RecursiveSNARK...");
    let mut recursive_snark = RecursiveSNARK::<E1>::new(&pp, &circuit, &z0)
        .expect("Failed to create RecursiveSNARK");

    let mut ic = <E1 as Engine>::Scalar::zero();

    for i in 0..num_steps {
        let step_start = Instant::now();

        recursive_snark.prove_step(&pp, &circuit, ic)
            .expect(&format!("Step {} failed", i));

        ic = recursive_snark.increment_commitment(&pp, &circuit);

        println!("   Step {}/{}: {:?}", i + 1, num_steps, step_start.elapsed());
    }

    println!("   ✅ Recursive proof complete ({:?})\n", start.elapsed());

    // Verify recursive SNARK
    println!("✓ Verifying Recursive SNARK...");
    let start = Instant::now();

    recursive_snark.verify(&pp, num_steps, &z0[..], ic)
        .expect("Verification failed");

    println!("   ✅ Verified! ({:?})\n", start.elapsed());

    // Compress SNARK for on-chain verification
    println!("📦 Compressing SNARK...");
    let mut rng = thread_rng();

    let start = Instant::now();
    println!("   Running compression setup...");

    let (compressed_pk, compressed_vk) = CompressedSNARK::setup(&pp, &mut rng, z0.len())
        .expect("Compression setup failed");

    println!("   Setup: {:?}", start.elapsed());

    let start = Instant::now();
    println!("   Compressing proof...");

    let compressed_proof = CompressedSNARK::prove(
        &pp,
        &compressed_pk,
        &recursive_snark,
        &mut rng,
    ).expect("Compression failed");

    println!("   ✅ Compressed! ({:?})\n", start.elapsed());

    // Verify compressed proof
    println!("✓ Verifying Compressed Proof...");
    let start = Instant::now();

    CompressedSNARK::verify(&compressed_proof, compressed_vk.clone())
        .expect("Compressed verification failed");

    println!("   ✅ Verified! ({:?})\n", start.elapsed());

    println!("✅ Success! Nova proof generation complete.\n");

    // Generate Solidity verifier (requires 'solidity' feature)
    #[cfg(feature = "solidity")]
    {
        println!("🔧 Generating Solidity Verifier Contract...");

        let function_selector = get_function_selector_for_nova_cyclefold_verifier(
            recursive_snark.z0.len() * 2 + 1
        );

        println!("   Preparing calldata...");
        let calldata = prepare_calldata(function_selector, &compressed_proof)
            .expect("Failed to prepare calldata");

        println!("   Creating verifier key...");
        let nova_cyclefold_vk = NovaCycleFoldVerifierKey::from((
            compressed_vk.pp_hash,
            SolidityGroth16VerifierKey::from(compressed_vk.groth16_vk),
            SolidityKZGVerifierKey::from((compressed_vk.kzg_vk, Vec::new())),
            recursive_snark.z0.len(),
        ));

        println!("   Generating Solidity code...");
        let solidity_code = get_decider_template_for_cyclefold_decider(nova_cyclefold_vk);

        // Save to files
        use std::fs;

        println!("   Writing files...");
        fs::write("./LiquidityVerifier.sol", &solidity_code)
            .expect("Failed to write verifier");
        fs::write("./liquidity_proof.calldata", &calldata)
            .expect("Failed to write calldata");

        println!("   ✅ Verifier saved to: LiquidityVerifier.sol");
        println!("   ✅ Calldata saved to: liquidity_proof.calldata");
        println!("   📊 Contract size: {} bytes", solidity_code.len());
        println!("   📊 Calldata size: {} bytes\n", calldata.len());

        println!("📋 Next Steps:");
        println!("   1. Deploy LiquidityVerifier.sol to Arc testnet");
        println!("   2. Call verifyNovaProof() with liquidity_proof.calldata");
        println!("   3. Integrate with TokenizedFundManager contract\n");
    }

    #[cfg(not(feature = "solidity"))]
    {
        println!("⚠️  To generate Solidity verifier, run with --features solidity");
        println!("   cargo run --release --example generate_real_nova_proof --features solidity\n");
    }

    println!("🎉 Demo Complete!");
    println!("\n═══════════════════════════════════════════════════");
    println!("Summary:");
    println!("  ✅ Public parameters generated");
    println!("  ✅ {} recursive steps proved", num_steps);
    println!("  ✅ Proof compressed for on-chain verification");
    println!("  ✅ Proof verified off-chain");
    #[cfg(feature = "solidity")]
    println!("  ✅ Solidity verifier generated");
    println!("═══════════════════════════════════════════════════");
}
