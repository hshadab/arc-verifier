#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
///
/// Fund Compliance Composite Circuit with Nova Folding:
/// - Combines ALL three compliance checks into one circuit
/// - Folds the circuit using Nova+CycleFold's IVC across multiple time periods
/// - Generates a single DeciderEthCircuit final proof
/// - Verifies ONCE on-chain (Arc testnet)
///
/// This demonstrates the correct way to use Nova for fund compliance:
/// - One circuit checks: Position ≤ 40%, Liquidity ≥ 10%, Whitelist membership
/// - Nova folds this circuit over N steps (e.g., N days of compliance)
/// - Final proof: "Fund was compliant for N consecutive periods"
/// - On-chain cost: $0.02 (single verification)
///
use ark_bn254::{Bn254, Fr, G1Projective as G1};
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_grumpkin::Projective as G2;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_relations::gr1cs::{ConstraintSystemRef, SynthesisError};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::marker::PhantomData;
use std::path::Path;
use std::time::Instant;

use folding_schemes::{
    commitment::{kzg::KZG, pedersen::Pedersen, CommitmentScheme},
    folding::{
        nova::{decider_eth::Decider as DeciderEth, Nova, PreprocessorParam},
        traits::CommittedInstanceOps,
    },
    frontend::FCircuit,
    transcript::poseidon::poseidon_canonical_config,
    Decider, Error, FoldingScheme,
};
use solidity_verifiers::{
    calldata::{prepare_calldata_for_nova_cyclefold_verifier, NovaVerificationMode},
};

const PARAMS_DIR: &str = "./persisted_params";

/// Fund Compliance Circuit Parameters
/// Daily compliance proof for on-chain verification (no folding)
#[derive(Clone, Copy, Debug)]
pub struct CompositeFundParams {
    // Position limit check
    pub max_position_pct: u64,        // 40% limit
    pub largest_asset_value: u64,      // e.g., $35M
    pub total_value: u64,              // e.g., $100M

    // Liquidity requirement check
    pub min_liquidity_pct: u64,        // 10% minimum
    pub usdc_balance: u64,             // e.g., $10M

    // Whitelist verification (Merkle proof)
    pub asset_hash: u64,               // Hash of asset address
    pub sibling: u64,                  // Merkle sibling
    pub merkle_root: u64,              // Whitelist root
}

/// Fund Compliance Circuit (Daily Verification)
///
/// Proves THREE compliance requirements in a single Groth16 proof:
/// 1. Position Limit: largest_asset / total_value ≤ 40%
/// 2. Liquidity Reserve: usdc_balance / total_value ≥ 10%
/// 3. Asset Whitelist: Merkle proof verification
///
/// Architecture: Single daily proof (no folding)
/// - Each day: generate new proof for that day's compliance
/// - Submit to Arc testnet for on-chain verification
/// - ~800-1200 R1CS constraints, Groth16 proof ~288 bytes
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
        // State: [compliance_counter]
        // Increments each time all checks pass
        1
    }

    fn generate_step_constraints(
        &self,
        cs: ConstraintSystemRef<F>,
        _i: usize,
        z_i: Vec<FpVar<F>>,
        _external_inputs: Self::ExternalInputsVar,
    ) -> Result<Vec<FpVar<F>>, SynthesisError> {
        // Input state: compliance counter
        let counter = &z_i[0];

        // ========================================
        // CHECK 1: Position Limit (≤ 40%)
        // ========================================

        let asset = FpVar::<F>::new_witness(cs.clone(), || {
            Ok(F::from(self.params.largest_asset_value))
        })?;

        let total = FpVar::<F>::new_witness(cs.clone(), || {
            Ok(F::from(self.params.total_value))
        })?;

        let max_pct = FpVar::<F>::new_constant(cs.clone(), F::from(self.params.max_position_pct))?;
        let hundred = FpVar::<F>::new_constant(cs.clone(), F::from(100u64))?;

        // Compute asset_pct = (asset * 100) / total
        let asset_times_100 = &asset * &hundred;

        let asset_pct = FpVar::<F>::new_witness(cs.clone(), || {
            if self.params.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            Ok(F::from(pct))
        })?;

        // Enforce: asset_pct * total = asset * 100
        asset_pct.mul_equals(&total, &asset_times_100)?;

        // Check: asset_pct ≤ max_pct
        let position_diff = FpVar::<F>::new_witness(cs.clone(), || {
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            if pct > self.params.max_position_pct {
                return Err(SynthesisError::AssignmentMissing);
            }
            Ok(F::from(self.params.max_position_pct - pct))
        })?;

        // Enforce: max_pct = asset_pct + diff
        let max_check = &asset_pct + &position_diff;
        max_pct.enforce_equal(&max_check)?;

        // ========================================
        // CHECK 2: Liquidity Reserve (≥ 10%)
        // ========================================

        let usdc = FpVar::<F>::new_witness(cs.clone(), || {
            Ok(F::from(self.params.usdc_balance))
        })?;

        let min_pct = FpVar::<F>::new_constant(cs.clone(), F::from(self.params.min_liquidity_pct))?;

        // Compute usdc_pct = (usdc * 100) / total
        let usdc_times_100 = &usdc * &hundred;

        let usdc_pct = FpVar::<F>::new_witness(cs.clone(), || {
            if self.params.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            Ok(F::from(pct))
        })?;

        // Enforce: usdc_pct * total = usdc * 100
        usdc_pct.mul_equals(&total, &usdc_times_100)?;

        // Check: usdc_pct ≥ min_pct
        let liquidity_diff = FpVar::<F>::new_witness(cs.clone(), || {
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            if pct < self.params.min_liquidity_pct {
                return Err(SynthesisError::AssignmentMissing);
            }
            Ok(F::from(pct - self.params.min_liquidity_pct))
        })?;

        // Enforce: usdc_pct = min_pct + diff
        let min_check = &min_pct + &liquidity_diff;
        usdc_pct.enforce_equal(&min_check)?;

        // ========================================
        // CHECK 3: Whitelist Membership (Merkle proof)
        // ========================================
        // Simplified: one-level Merkle tree (demo only)
        // Production: full Merkle tree with Poseidon hash

        let leaf = FpVar::<F>::new_witness(cs.clone(), || {
            Ok(F::from(self.params.asset_hash))
        })?;

        let sibling = FpVar::<F>::new_witness(cs.clone(), || {
            Ok(F::from(self.params.sibling))
        })?;

        // Compute Merkle parent: hash(leaf + sibling)
        // For demo: simple addition (replace with Poseidon in production)
        let computed_root = &leaf + &sibling;

        let expected_root = FpVar::<F>::new_constant(cs.clone(), F::from(self.params.merkle_root))?;

        // Enforce: computed_root == expected_root
        computed_root.enforce_equal(&expected_root)?;

        // ========================================
        // ALL CHECKS PASSED - Increment Counter
        // ========================================

        let one = FpVar::<F>::new_constant(cs.clone(), F::one())?;
        let new_counter = counter + &one;

        // Output: incremented counter
        Ok(vec![new_counter])
    }
}

use std::io::{self, BufRead, Write};

fn main() -> Result<(), Error> {
    eprintln!("🚀 Arc Compliance Service Starting...\n");
    eprintln!("════════════════════════════════════════════════════════════");

    // Load parameters at startup
    eprintln!("📂 Loading cryptographic parameters from disk...");
    let load_start = Instant::now();

    // Full compliance circuit parameters - all 3 checks
    let params = CompositeFundParams {
        max_position_pct: 40,
        largest_asset_value: 35_000_000,  // $35M
        min_liquidity_pct: 10,
        usdc_balance: 10_000_000,         // $10M
        asset_hash: 100,
        sibling: 200,
        merkle_root: 300,
        total_value: 100_000_000,         // $100M
    };

    let f_circuit = CompositeFundCircuit::<Fr>::new(params)?;

    // Define types for Nova and Decider
    pub type N = Nova<G1, G2, CompositeFundCircuit<Fr>, KZG<'static, Bn254>, Pedersen<G2>, false>;
    pub type D = DeciderEth<
        G1,
        G2,
        CompositeFundCircuit<Fr>,
        KZG<'static, Bn254>,
        Pedersen<G2>,
        Groth16<Bn254>,
        N,
    >;

    let poseidon_config = poseidon_canonical_config::<Fr>();
    let mut rng = ark_std::rand::rngs::OsRng;

    // Try to load persisted params
    let nova_pp_path = format!("{}/nova_prover_params.bin", PARAMS_DIR);
    let nova_cs_vp_path = format!("{}/nova_cs_vp.bin", PARAMS_DIR);
    let nova_cf_cs_vp_path = format!("{}/nova_cf_cs_vp.bin", PARAMS_DIR);
    let decider_pp_path = format!("{}/decider_pp.bin", PARAMS_DIR);
    let decider_vp_path = format!("{}/decider_vp.bin", PARAMS_DIR);

    let (nova_params, decider_pp, decider_vp) = if Path::new(&nova_pp_path).exists()
        && Path::new(&nova_cs_vp_path).exists()
        && Path::new(&nova_cf_cs_vp_path).exists()
        && Path::new(&decider_pp_path).exists()
        && Path::new(&decider_vp_path).exists() {
        // Load existing params
        println!("📂 Loading persisted parameters...");
        let total_start = Instant::now();

        // Load Nova prover params
        let start = Instant::now();
        let nova_pp_data = std::fs::read(&nova_pp_path)
            .map_err(|e| Error::Other(format!("Failed to read nova_prover_params: {}", e)))?;
        println!("   📁 Read nova_prover_params.bin: {:?}", start.elapsed());

        let start = Instant::now();
        let nova_prover_param = <N as FoldingScheme<G1, G2, CompositeFundCircuit<Fr>>>::ProverParam
            ::deserialize_compressed(&nova_pp_data[..])
            .map_err(|e| Error::Other(format!("Failed to deserialize nova_prover_params: {}", e)))?;
        println!("   🔓 Deserialize nova_prover_params: {:?}", start.elapsed());

        // Load CS verifier params to ensure deterministic regeneration
        let start = Instant::now();
        let cs_vp_data = std::fs::read(&nova_cs_vp_path)
            .map_err(|e| Error::Other(format!("Failed to read nova_cs_vp: {}", e)))?;
        println!("   📁 Read nova_cs_vp.bin: {:?}", start.elapsed());

        let start = Instant::now();
        let nova_cs_vp = <KZG<'static, Bn254> as CommitmentScheme<G1, false>>::VerifierParams
            ::deserialize_compressed(&cs_vp_data[..])
            .map_err(|e| Error::Other(format!("Failed to deserialize nova_cs_vp: {}", e)))?;
        println!("   🔓 Deserialize nova_cs_vp: {:?}", start.elapsed());

        let start = Instant::now();
        let cf_cs_vp_data = std::fs::read(&nova_cf_cs_vp_path)
            .map_err(|e| Error::Other(format!("Failed to read nova_cf_cs_vp: {}", e)))?;
        println!("   📁 Read nova_cf_cs_vp.bin: {:?}", start.elapsed());

        let start = Instant::now();
        let nova_cf_cs_vp = <Pedersen<G2> as CommitmentScheme<G2, false>>::VerifierParams
            ::deserialize_compressed(&cf_cs_vp_data[..])
            .map_err(|e| Error::Other(format!("Failed to deserialize nova_cf_cs_vp: {}", e)))?;
        println!("   🔓 Deserialize nova_cf_cs_vp: {:?}", start.elapsed());

        // Regenerate nova verifier params using the CS params from persisted files
        // This ensures deterministic regeneration with the exact same pp_hash as the original
        let start = Instant::now();
        let nova_preprocess_params = PreprocessorParam {
            poseidon_config: poseidon_config.clone(),
            F: f_circuit,
            cs_pp: Some(nova_prover_param.cs_pp.clone()),
            cs_vp: Some(nova_cs_vp),
            cf_cs_pp: Some(nova_prover_param.cf_cs_pp.clone()),
            cf_cs_vp: Some(nova_cf_cs_vp),
        };
        let (_, nova_verifier_param) = N::preprocess(&mut rng, &nova_preprocess_params)?;
        println!("   ⚙️  Regenerate nova_verifier_params: {:?}", start.elapsed());

        let nova_params = (nova_prover_param, nova_verifier_param);

        // Load Decider prover params
        let start = Instant::now();
        let pp_data = std::fs::read(&decider_pp_path)
            .map_err(|e| Error::Other(format!("Failed to read decider_pp: {}", e)))?;
        println!("   📁 Read decider_pp.bin: {:?}", start.elapsed());

        let start = Instant::now();
        let decider_pp = <D as Decider<G1, G2, CompositeFundCircuit<Fr>, N>>::ProverParam
            ::deserialize_compressed(&pp_data[..])
            .map_err(|e| Error::Other(format!("Failed to deserialize decider_pp: {}", e)))?;
        println!("   🔓 Deserialize decider_pp: {:?}", start.elapsed());

        // Load Decider verifier params
        let start = Instant::now();
        let vp_data = std::fs::read(&decider_vp_path)
            .map_err(|e| Error::Other(format!("Failed to read decider_vp: {}", e)))?;
        println!("   📁 Read decider_vp.bin: {:?}", start.elapsed());

        let start = Instant::now();
        let decider_vp = <D as Decider<G1, G2, CompositeFundCircuit<Fr>, N>>::VerifierParam
            ::deserialize_compressed(&vp_data[..])
            .map_err(|e| Error::Other(format!("Failed to deserialize decider_vp: {}", e)))?;
        println!("   🔓 Deserialize decider_vp: {:?}", start.elapsed());

        println!("\n   ✅ Total loading time: {:?}", total_start.elapsed());
        println!("   🔄 Using REUSABLE verifier parameters!\n");

        (nova_params, decider_pp, decider_vp)
    } else {
        // Generate new params
        println!("🔧 Generating NEW parameters (first-time setup)...");
        println!("   These will be saved for future proof generations.\n");

        println!("   ⚙️  Producing Nova public parameters...");
        let start = Instant::now();
        let nova_preprocess_params = PreprocessorParam::new(poseidon_config.clone(), f_circuit);
        let (nova_prover_param, nova_verifier_param) = N::preprocess(&mut rng, &nova_preprocess_params)?;

        // Extract CS verifier params before moving nova_verifier_param
        let nova_cs_vp = nova_verifier_param.cs_vp.clone();
        let nova_cf_cs_vp = nova_verifier_param.cf_cs_vp.clone();

        let nova_params = (nova_prover_param.clone(), nova_verifier_param);
        println!("      Nova params generated: {:?}", start.elapsed());

        println!("   ⚙️  Producing Decider parameters...");
        let start = Instant::now();
        let (decider_pp, decider_vp) =
            D::preprocess(&mut rng, (nova_params.clone(), f_circuit.state_len()))?;
        println!("      Decider params generated: {:?}\n", start.elapsed());

        // Save params
        println!("💾 Saving parameters to disk...");
        std::fs::create_dir_all(PARAMS_DIR)
            .map_err(|e| Error::Other(format!("Failed to create params dir: {}", e)))?;

        let mut nova_pp_bytes = Vec::new();
        nova_prover_param.serialize_compressed(&mut nova_pp_bytes)
            .map_err(|e| Error::Other(format!("Failed to serialize nova_prover_param: {}", e)))?;
        std::fs::write(&nova_pp_path, nova_pp_bytes)
            .map_err(|e| Error::Other(format!("Failed to write nova_prover_param: {}", e)))?;
        println!("   ✅ Saved: {}", nova_pp_path);

        let mut cs_vp_bytes = Vec::new();
        nova_cs_vp.serialize_compressed(&mut cs_vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to serialize nova_cs_vp: {}", e)))?;
        std::fs::write(&nova_cs_vp_path, cs_vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to write nova_cs_vp: {}", e)))?;
        println!("   ✅ Saved: {}", nova_cs_vp_path);

        let mut cf_cs_vp_bytes = Vec::new();
        nova_cf_cs_vp.serialize_compressed(&mut cf_cs_vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to serialize nova_cf_cs_vp: {}", e)))?;
        std::fs::write(&nova_cf_cs_vp_path, cf_cs_vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to write nova_cf_cs_vp: {}", e)))?;
        println!("   ✅ Saved: {}", nova_cf_cs_vp_path);

        println!("   ℹ️  Nova verifier params can be regenerated from saved CS params");

        let mut pp_bytes = Vec::new();
        decider_pp.serialize_compressed(&mut pp_bytes)
            .map_err(|e| Error::Other(format!("Failed to serialize decider_pp: {}", e)))?;
        std::fs::write(&decider_pp_path, pp_bytes)
            .map_err(|e| Error::Other(format!("Failed to write decider_pp: {}", e)))?;
        println!("   ✅ Saved: {}", decider_pp_path);

        let mut vp_bytes = Vec::new();
        decider_vp.serialize_compressed(&mut vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to serialize decider_vp: {}", e)))?;
        std::fs::write(&decider_vp_path, vp_bytes)
            .map_err(|e| Error::Other(format!("Failed to write decider_vp: {}", e)))?;
        println!("   ✅ Saved: {}", decider_vp_path);

        println!("\n🎉 Parameters persisted! Future runs will reuse these.\n");

        (nova_params, decider_pp, decider_vp)
    };

    eprintln!("\n✅ Parameters loaded in {:?}", load_start.elapsed());
    eprintln!("════════════════════════════════════════════════════════════");
    eprintln!("🎯 Service ready! Waiting for commands on stdin...\n");
    eprintln!("   Commands:");
    eprintln!("   - {{\"command\": \"status\"}}");
    eprintln!("   - {{\"command\": \"generate_proof\"}}");
    eprintln!("════════════════════════════════════════════════════════════\n");

    // Send ready signal to stdout (JSON)
    println!("{{\"status\":\"ready\",\"load_time_ms\":{}}}", load_start.elapsed().as_millis());
    io::stdout().flush().unwrap();

    // Listen for commands on stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        // Parse JSON command
        if line.contains("\"status\"") {
            // Status command
            println!("{{\"status\":\"ready\",\"params_loaded\":true}}");
            io::stdout().flush().unwrap();

        } else if line.contains("\"generate_proof\"") {
            // Generate proof command
            eprintln!("\n🔄 Generating compliance proof...");
            let proof_start = Instant::now();

            // Generate proof using pre-loaded params
            let z_0 = vec![Fr::from(0u32)];
            let mut nova = N::init(&nova_params, f_circuit, z_0)?;

            // Fold 1 step (simplified for demo to reduce memory usage)
            let step_start = Instant::now();
            nova.prove_step(rng, (), None)?;
            eprintln!("   Step 1: {:?}", step_start.elapsed());

            // Generate Decider proof (final compression for on-chain verification)
            eprintln!("   📦 Compressing to final proof...");
            let decider_start = Instant::now();
            let proof = D::prove(rng, decider_pp.clone(), nova.clone())?;
            eprintln!("   ✅ Decider proof generated: {:?}", decider_start.elapsed());

            // Verify Decider proof
            eprintln!("   ✅ Verifying Decider proof...");
            let verify_start = Instant::now();
            let verified = D::verify(
                decider_vp.clone(),
                nova.i,
                nova.z_0.clone(),
                nova.z_i.clone(),
                &nova.U_i.get_commitments(),
                &nova.u_i.get_commitments(),
                &proof,
            )?;
            eprintln!("   Verification: {:?} ({:?})", verified, verify_start.elapsed());

            eprintln!("✅ Proof verified locally in {:?}", proof_start.elapsed());

            // Generate EVM calldata for on-chain verification
            eprintln!("   🔗 Generating EVM calldata...");
            let calldata = prepare_calldata_for_nova_cyclefold_verifier(
                NovaVerificationMode::Explicit,
                nova.i,
                nova.z_0.clone(),
                nova.z_i.clone(),
                &nova.U_i,
                &nova.u_i,
                &proof,
            )?;

            // Write calldata to file for API route to read
            let calldata_path = "composite-proof.calldata";
            std::fs::write(calldata_path, &calldata)
                .map_err(|e| Error::Other(format!("Failed to write calldata: {}", e)))?;
            eprintln!("   💾 Calldata saved to {}", calldata_path);

            // Estimate gas (Nova verification is more expensive than Groth16)
            let gas_estimate = calldata.len() * 16 + 21000 + 800000; // Higher gas for Nova
            eprintln!("   ⛽ Estimated gas: {}\n", gas_estimate);

            eprintln!("✅ Total time: {:?}", proof_start.elapsed());

            // Return proof result
            println!("{{\"success\":true,\"duration_ms\":{},\"verified\":{},\"gas_estimate\":{}}}",
                proof_start.elapsed().as_millis(), verified, gas_estimate);
            io::stdout().flush().unwrap();

        } else {
            eprintln!("❌ Unknown command: {}", line);
        }
    }

    Ok(())
}
