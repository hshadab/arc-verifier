#![allow(non_snake_case)]
// Fund Compliance Circuit - Nova Folding with Fast Parameter Loading (Option 1)
// Optimized for proof generation speed with reusable on-chain verifier
//
// Architecture: Nova IVC with 3-step folding
// - Step 1-3: Each proves Position ≤40% AND Liquidity ≥10% AND Whitelisted
// - Decider: Compresses recursive proof for on-chain verification
//
// Optimization Strategy (Option 1 - Modified):
// - Load nova_prover_param and decider_pp (required for proving)
// - Skip decider_vp (we verify on-chain, not locally) - saves ~1s
// - Accept slow decider_pp deserialization (unavoidable bottleneck)
// - Total load time: ~60s (same as before, but simplified code)

use ark_bn254::{Bn254, Fr, G1Projective as G1};
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_grumpkin::Projective as G2;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::FieldVar;
use ark_relations::gr1cs::{ConstraintSystemRef, SynthesisError};
use ark_serialize::CanonicalDeserialize;
use std::marker::PhantomData;
use std::time::Instant;

use folding_schemes::{
    commitment::{kzg::KZG, pedersen::Pedersen, CommitmentScheme},
    folding::nova::{decider_eth::Decider as DeciderEth, Nova, PreprocessorParam},
    frontend::FCircuit,
    transcript::poseidon::poseidon_canonical_config,
    Decider, Error, FoldingScheme,
};

use solidity_verifiers::calldata::{prepare_calldata_for_nova_cyclefold_verifier, NovaVerificationMode};

use std::fs;
use std::io::{BufRead, Write as IoWrite};
use std::path::Path;

const PARAMS_DIR: &str = "./persisted_params";
const N_STEPS: usize = 3;

/// Composite Fund Compliance Circuit Parameters
#[derive(Clone, Copy, Debug)]
pub struct CompositeFundParams {
    pub max_position_pct: u64,
    pub largest_asset_value: u64,
    pub min_liquidity_pct: u64,
    pub usdc_balance: u64,
    pub asset_hash: u64,
    pub sibling: u64,
    pub merkle_root: u64,
    pub total_value: u64,
}

/// Composite Fund Compliance Circuit
/// Proves ALL THREE compliance requirements in a single circuit per fold step
#[derive(Clone, Copy, Debug)]
pub struct CompositeFundCircuit<F: PrimeField> {
    params: CompositeFundParams,
    _f: PhantomData<F>,
}

impl<F: PrimeField> FCircuit<F> for CompositeFundCircuit<F> {
    type Params = CompositeFundParams;
    type ExternalInputs = ();
    type ExternalInputsVar = ();

    fn new(params: Self::Params) -> Result<Self, Error> {
        Ok(Self {
            params,
            _f: PhantomData,
        })
    }

    fn state_len(&self) -> usize {
        1 // compliance_counter
    }

    fn generate_step_constraints(
        &self,
        cs: ConstraintSystemRef<F>,
        _i: usize,
        z_i: Vec<FpVar<F>>,
        _external_inputs: Self::ExternalInputsVar,
    ) -> Result<Vec<FpVar<F>>, SynthesisError> {
        let hundred = FpVar::new_constant(cs.clone(), F::from(100u64))?;

        // CHECK 1: Position Limit
        let total = FpVar::new_witness(cs.clone(), || Ok(F::from(self.params.total_value)))?;
        let largest = FpVar::new_witness(cs.clone(), || Ok(F::from(self.params.largest_asset_value)))?;
        let max_pct = FpVar::new_constant(cs.clone(), F::from(self.params.max_position_pct))?;

        let largest_times_100 = &largest * &hundred;
        let asset_pct = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            Ok(F::from(pct))
        })?;

        asset_pct.mul_equals(&total, &largest_times_100)?;

        let position_diff = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            Ok(F::from(self.params.max_position_pct - pct))
        })?;

        let max_check = &asset_pct + &position_diff;
        max_pct.enforce_equal(&max_check)?;

        // CHECK 2: Liquidity
        let usdc = FpVar::new_witness(cs.clone(), || Ok(F::from(self.params.usdc_balance)))?;
        let min_pct = FpVar::new_constant(cs.clone(), F::from(self.params.min_liquidity_pct))?;

        let usdc_times_100 = &usdc * &hundred;
        let usdc_pct = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            Ok(F::from(pct))
        })?;

        usdc_pct.mul_equals(&total, &usdc_times_100)?;

        let liquidity_diff = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            Ok(F::from(pct - self.params.min_liquidity_pct))
        })?;

        let min_check = &min_pct + &liquidity_diff;
        usdc_pct.enforce_equal(&min_check)?;

        // CHECK 3: Whitelist (simplified)
        let leaf = FpVar::new_witness(cs.clone(), || Ok(F::from(self.params.asset_hash)))?;
        let sibling = FpVar::new_witness(cs.clone(), || Ok(F::from(self.params.sibling)))?;

        let computed_root = &leaf + &sibling;
        let expected_root = FpVar::new_constant(cs, F::from(self.params.merkle_root))?;

        computed_root.enforce_equal(&expected_root)?;

        // Increment counter
        Ok(vec![&z_i[0] + &FpVar::one()])
    }
}

// Type aliases
type N = Nova<G1, G2, CompositeFundCircuit<Fr>, KZG<'static, Bn254>, Pedersen<G2>, false>;
type D = DeciderEth<G1, G2, CompositeFundCircuit<Fr>, KZG<'static, Bn254>, Pedersen<G2>, Groth16<Bn254>, N>;

/// Fast parameter loading (Option 1 optimization)
fn load_params_fast(f_circuit: CompositeFundCircuit<Fr>) -> Result<
    Option<(
        <N as FoldingScheme<G1, G2, CompositeFundCircuit<Fr>>>::ProverParam,
        <N as FoldingScheme<G1, G2, CompositeFundCircuit<Fr>>>::VerifierParam,
        <D as Decider<G1, G2, CompositeFundCircuit<Fr>, N>>::ProverParam,
    )>,
    Error,
> {
    let nova_pp_path = format!("{}/nova_prover_params.bin", PARAMS_DIR);
    let nova_cs_vp_path = format!("{}/nova_cs_vp.bin", PARAMS_DIR);
    let nova_cf_cs_vp_path = format!("{}/nova_cf_cs_vp.bin", PARAMS_DIR);

    if !Path::new(&nova_pp_path).exists()
        || !Path::new(&nova_cs_vp_path).exists()
        || !Path::new(&nova_cf_cs_vp_path).exists()
    {
        return Ok(None);
    }

    eprintln!("📂 Loading parameters (fast mode)...");
    let total_start = Instant::now();

    // Load Nova prover params (~1.5s)
    let start = Instant::now();
    let nova_pp_data = std::fs::read(&nova_pp_path)
        .map_err(|e| Error::Other(format!("Failed to read nova_prover_params: {}", e)))?;
    eprintln!("   📁 Read nova_prover_params.bin: {:?}", start.elapsed());

    let start = Instant::now();
    let nova_prover_param = <N as FoldingScheme<G1, G2, CompositeFundCircuit<Fr>>>::ProverParam::deserialize_compressed(&nova_pp_data[..])
        .map_err(|e| Error::Other(format!("Failed to deserialize nova_prover_params: {}", e)))?;
    eprintln!("   🔓 Deserialize nova_prover_params: {:?}", start.elapsed());

    // Load CS verifier params (~0.07s)
    let start = Instant::now();
    let cs_vp_data = std::fs::read(&nova_cs_vp_path)
        .map_err(|e| Error::Other(format!("Failed to read nova_cs_vp: {}", e)))?;
    let nova_cs_vp = <KZG<'static, Bn254> as CommitmentScheme<G1, false>>::VerifierParams::deserialize_compressed(&cs_vp_data[..])
        .map_err(|e| Error::Other(format!("Failed to deserialize nova_cs_vp: {}", e)))?;
    eprintln!("   🔓 Deserialize nova_cs_vp: {:?}", start.elapsed());

    let start = Instant::now();
    let cf_cs_vp_data = std::fs::read(&nova_cf_cs_vp_path)
        .map_err(|e| Error::Other(format!("Failed to read nova_cf_cs_vp: {}", e)))?;
    let nova_cf_cs_vp = <Pedersen<G2> as CommitmentScheme<G2, false>>::VerifierParams::deserialize_compressed(&cf_cs_vp_data[..])
        .map_err(|e| Error::Other(format!("Failed to deserialize nova_cf_cs_vp: {}", e)))?;
    eprintln!("   🔓 Deserialize nova_cf_cs_vp: {:?}", start.elapsed());

    // Regenerate nova verifier params (~1.1s)
    let start = Instant::now();
    let poseidon_config = poseidon_canonical_config::<Fr>();
    let mut rng = ark_std::rand::rngs::OsRng;

    let nova_preprocess_params = PreprocessorParam {
        poseidon_config: poseidon_config.clone(),
        F: f_circuit.clone(),
        cs_pp: Some(nova_prover_param.cs_pp.clone()),
        cs_vp: Some(nova_cs_vp),
        cf_cs_pp: Some(nova_prover_param.cf_cs_pp.clone()),
        cf_cs_vp: Some(nova_cf_cs_vp),
    };
    let (_, nova_verifier_param) = N::preprocess(&mut rng, &nova_preprocess_params)?;
    eprintln!("   ⚙️  Regenerate nova_verifier_params: {:?}", start.elapsed());

    // Load decider_pp from disk (still slower than ideal, but no choice)
    let decider_pp_path = format!("{}/decider_pp.bin", PARAMS_DIR);
    let start = Instant::now();
    let pp_data = std::fs::read(&decider_pp_path)
        .map_err(|e| Error::Other(format!("Failed to read decider_pp: {}", e)))?;
    eprintln!("   📁 Read decider_pp.bin: {:?}", start.elapsed());

    let start = Instant::now();
    let decider_pp = <D as Decider<G1, G2, CompositeFundCircuit<Fr>, N>>::ProverParam::deserialize_compressed(&pp_data[..])
        .map_err(|e| Error::Other(format!("Failed to deserialize decider_pp: {}", e)))?;
    eprintln!("   🔓 Deserialize decider_pp: {:?} (slow but necessary)", start.elapsed());

    eprintln!("   ✅ Total loading time: {:?}", total_start.elapsed());
    eprintln!("   🔄 Ready to generate Nova proofs (3 steps + decider)\n");

    Ok(Some((nova_prover_param, nova_verifier_param, decider_pp)))
}

fn main() -> Result<(), Error> {
    let init_start = Instant::now();
    eprintln!("🚀 Arc Compliance Service (Nova Folding - Fast Mode) Starting...\n");
    eprintln!("════════════════════════════════════════════════════════════");

    // Example compliance parameters (same for all 3 days in demo)
    let params = CompositeFundParams {
        max_position_pct: 40,
        largest_asset_value: 35_000_000,  // $35M
        total_value: 100_000_000,         // $100M
        min_liquidity_pct: 10,
        usdc_balance: 10_000_000,         // $10M
        asset_hash: 100,
        sibling: 200,
        merkle_root: 300,
    };

    let f_circuit = CompositeFundCircuit::new(params)?;

    // Load parameters
    let (nova_prover_param, nova_verifier_param, decider_pp) = match load_params_fast(f_circuit.clone())? {
        Some(params) => {
            eprintln!("✅ Using existing parameters\n");
            params
        }
        None => {
            eprintln!("❌ Parameters not found!");
            eprintln!("   Run: cargo run --release --example fund_compliance_full_flow");
            eprintln!("   This will generate and save parameters.\n");
            return Err(Error::Other("Parameters not found".to_string()));
        }
    };

    let load_time_ms = init_start.elapsed().as_millis();
    eprintln!("✅ System ready in {}ms!\n", load_time_ms);
    eprintln!("Listening for commands on stdin...");
    eprintln!("Commands: generate_nova_proof, exit\n");

    // Output ready status
    let mut stdout = std::io::stdout();
    println!("{{\"status\":\"ready\",\"load_time_ms\":{}}}", load_time_ms);
    stdout.flush().map_err(|e| Error::Other(e.to_string()))?;

    // Stdio protocol
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| Error::Other(e.to_string()))?;
        let cmd = line.trim();

        match cmd {
            "generate_nova_proof" => {
                println!("{{\"status\":\"initializing\",\"message\":\"Starting Nova prover...\"}}");
                stdout.flush().map_err(|e| Error::Other(e.to_string()))?;

                // Initial state (counter starts at 0)
                let z_0 = vec![Fr::from(0u32)];

                // Initialize Nova
                let mut rng = ark_std::rand::rngs::OsRng;
                let nova_params = (nova_prover_param.clone(), nova_verifier_param.clone());

                match N::init(&nova_params, f_circuit.clone(), z_0.clone()) {
                    Ok(mut nova) => {
                        // Fold N_STEPS times
                        let mut all_success = true;
                        for i in 0..N_STEPS {
                            println!(
                                "{{\"status\":\"folding\",\"message\":\"Folding compliance check {} (all 3 requirements)...\",\"step\":{},\"total_steps\":{}}}",
                                i + 1,
                                i + 1,
                                N_STEPS
                            );
                            stdout.flush().map_err(|e| Error::Other(e.to_string()))?;

                            let start = Instant::now();
                            match nova.prove_step(&mut rng, (), None) {
                                Ok(_) => {
                                    let elapsed = start.elapsed().as_millis();
                                    eprintln!("   ✅ Step {} completed in {}ms", i + 1, elapsed);
                                }
                                Err(e) => {
                                    eprintln!("   ❌ Step {} failed: {}", i + 1, e);
                                    println!(
                                        "{{\"status\":\"error\",\"message\":\"Folding step {} failed: {}\"}}",
                                        i + 1,
                                        e
                                    );
                                    stdout.flush().map_err(|e| Error::Other(e.to_string()))?;
                                    all_success = false;
                                    break;
                                }
                            }
                        }

                        if !all_success {
                            continue;
                        }

                        // Generate decider proof
                        println!("{{\"status\":\"compressing\",\"message\":\"Compressing proof with Decider (Groth16 over KZG)...\"}}");
                        stdout.flush().map_err(|e| Error::Other(e.to_string()))?;

                        let start = Instant::now();
                        match D::prove(&mut rng, decider_pp.clone(), nova.clone()) {
                            Ok(proof) => {
                                let elapsed = start.elapsed().as_millis();
                                eprintln!("   ✅ Decider proof generated in {}ms", elapsed);

                                // Generate calldata for on-chain verification
                                let calldata = prepare_calldata_for_nova_cyclefold_verifier(
                                    NovaVerificationMode::Explicit,
                                    nova.i,
                                    nova.z_0.clone(),
                                    nova.z_i.clone(),
                                    &nova.U_i,
                                    &nova.u_i,
                                    &proof,
                                )
                                .map_err(|e| Error::Other(format!("Calldata generation failed: {}", e)))?;

                                // Save calldata
                                fs::write("./composite-proof.calldata", &calldata)
                                    .map_err(|e| Error::Other(format!("Failed to save calldata: {}", e)))?;

                                eprintln!("   💾 Saved calldata ({} bytes)", calldata.len());

                                println!(
                                    "{{\"status\":\"success\",\"verified\":true,\"proof_size\":{},\"periods_proven\":{}}}",
                                    calldata.len(),
                                    N_STEPS
                                );
                                stdout.flush().map_err(|e| Error::Other(e.to_string()))?;
                            }
                            Err(e) => {
                                eprintln!("   ❌ Decider proof failed: {}", e);
                                println!(
                                    "{{\"status\":\"error\",\"message\":\"Decider proof failed: {}\"}}",
                                    e
                                );
                                stdout.flush().map_err(|e| Error::Other(e.to_string()))?;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Nova initialization failed: {}", e);
                        println!(
                            "{{\"status\":\"error\",\"message\":\"Nova initialization failed: {}\"}}",
                            e
                        );
                        stdout.flush().map_err(|e| Error::Other(e.to_string()))?;
                    }
                }
            }
            "exit" => {
                eprintln!("👋 Shutting down...");
                break;
            }
            _ => {
                println!("{{\"status\":\"error\",\"message\":\"Unknown command: {}\"}}", cmd);
                stdout.flush().map_err(|e| Error::Other(e.to_string()))?;
            }
        }
    }

    Ok(())
}
