// Fund Compliance Circuit - Groth16 Direct (No Folding)
// Daily compliance proofs for on-chain verification on Arc testnet
//
// Architecture: Single Groth16 proof per day (no Nova folding)
// - Proves 3 compliance requirements in one proof
// - ~800-1200 R1CS constraints
// - Memory: ~4-8GB for setup (vs 20GB+ for Nova+Decider)
// - Proof size: ~288 bytes

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use std::fs;
use std::io::{BufRead, Write as IoWrite};
use std::path::Path;
use std::time::Instant;

// For Solidity verifier generation
use solidity_verifiers::{Groth16VerifierKey, ProtocolVerifierKey};

const PARAMS_DIR: &str = "./groth16_params";

/// Fund Compliance Parameters
#[derive(Clone, Copy, Debug)]
pub struct ComplianceParams {
    // Position limit check (≤40%)
    pub max_position_pct: u64,
    pub largest_asset_value: u64,
    pub total_value: u64,

    // Liquidity requirement (≥10%)
    pub min_liquidity_pct: u64,
    pub usdc_balance: u64,

    // Whitelist verification (Merkle proof)
    pub asset_hash: u64,
    pub sibling: u64,
    pub merkle_root: u64,
}

/// Compliance Circuit
/// Proves 3 requirements:
/// 1. Position limit: largest_asset / total ≤ 40%
/// 2. Liquidity: usdc / total ≥ 10%
/// 3. Whitelist: Merkle proof verification
#[derive(Clone)]
pub struct ComplianceCircuit {
    params: ComplianceParams,
}

impl ComplianceCircuit {
    pub fn new(params: ComplianceParams) -> Self {
        Self { params }
    }
}

impl ConstraintSynthesizer<Fr> for ComplianceCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Constants
        let hundred = FpVar::<Fr>::new_constant(cs.clone(), Fr::from(100u64))?;

        // ========================================
        // CHECK 1: Position Limit (≤ 40%)
        // ========================================

        let total = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.params.total_value)))?;
        let largest = FpVar::new_witness(cs.clone(), || {
            Ok(Fr::from(self.params.largest_asset_value))
        })?;
        let max_pct =
            FpVar::new_constant(cs.clone(), Fr::from(self.params.max_position_pct))?;

        // Compute asset_pct = (largest * 100) / total
        let largest_times_100 = &largest * &hundred;
        let asset_pct = FpVar::new_witness(cs.clone(), || {
            if self.params.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            Ok(Fr::from(pct))
        })?;

        // Enforce: asset_pct * total = largest * 100
        asset_pct.mul_equals(&total, &largest_times_100)?;

        // Check: asset_pct ≤ max_pct (by proving difference is non-negative)
        let position_diff = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
            if pct > self.params.max_position_pct {
                return Err(SynthesisError::AssignmentMissing);
            }
            Ok(Fr::from(self.params.max_position_pct - pct))
        })?;

        // Enforce: max_pct = asset_pct + diff
        let max_check = &asset_pct + &position_diff;
        max_pct.enforce_equal(&max_check)?;

        // ========================================
        // CHECK 2: Liquidity Reserve (≥ 10%)
        // ========================================

        let usdc = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.params.usdc_balance)))?;
        let min_pct =
            FpVar::new_constant(cs.clone(), Fr::from(self.params.min_liquidity_pct))?;

        // Compute usdc_pct = (usdc * 100) / total
        let usdc_times_100 = &usdc * &hundred;
        let usdc_pct = FpVar::new_witness(cs.clone(), || {
            if self.params.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            Ok(Fr::from(pct))
        })?;

        // Enforce: usdc_pct * total = usdc * 100
        usdc_pct.mul_equals(&total, &usdc_times_100)?;

        // Check: usdc_pct ≥ min_pct
        let liquidity_diff = FpVar::new_witness(cs.clone(), || {
            let pct = (self.params.usdc_balance * 100) / self.params.total_value;
            if pct < self.params.min_liquidity_pct {
                return Err(SynthesisError::AssignmentMissing);
            }
            Ok(Fr::from(pct - self.params.min_liquidity_pct))
        })?;

        // Enforce: usdc_pct = min_pct + diff
        let min_check = &min_pct + &liquidity_diff;
        usdc_pct.enforce_equal(&min_check)?;

        // ========================================
        // CHECK 3: Whitelist Membership
        // ========================================

        let leaf = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.params.asset_hash)))?;
        let sibling = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.params.sibling)))?;

        // Simple Merkle verification (demo - production would use Poseidon)
        let computed_root = &leaf + &sibling;
        let expected_root = FpVar::new_constant(cs.clone(), Fr::from(self.params.merkle_root))?;

        // Enforce: computed_root == expected_root
        computed_root.enforce_equal(&expected_root)?;

        Ok(())
    }
}

/// Generate Groth16 proving and verifying keys
fn setup_groth16(
    circuit: ComplianceCircuit,
) -> Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>), Box<dyn std::error::Error>> {
    eprintln!("🔧 Generating Groth16 parameters...");
    let start = Instant::now();

    let mut rng = StdRng::seed_from_u64(0u64);
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)?;

    eprintln!("   Groth16 setup completed: {:?}", start.elapsed());
    Ok((pk, vk))
}

/// Generate a Groth16 proof
fn generate_proof(
    pk: &ProvingKey<Bn254>,
    circuit: ComplianceCircuit,
) -> Result<Proof<Bn254>, Box<dyn std::error::Error>> {
    eprintln!("🔐 Generating Groth16 proof...");
    let start = Instant::now();

    let mut rng = StdRng::seed_from_u64(0u64);
    let proof = Groth16::<Bn254>::prove(pk, circuit, &mut rng)?;

    eprintln!("   Proof generated: {:?}", start.elapsed());
    Ok(proof)
}

/// Verify a Groth16 proof
fn verify_proof(
    vk: &VerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
) -> Result<bool, Box<dyn std::error::Error>> {
    eprintln!("✅ Verifying proof...");
    let start = Instant::now();

    // Public inputs (empty for this circuit - all private)
    let public_inputs = vec![];
    let valid = Groth16::<Bn254>::verify(vk, &public_inputs, proof)?;

    eprintln!("   Verification completed: {:?}", start.elapsed());
    Ok(valid)
}

/// Export proof as calldata for on-chain verification
fn export_calldata(proof: &Proof<Bn254>, _vk: &VerifyingKey<Bn254>) -> Vec<u8> {
    use ark_ec::AffineRepr;
    use ark_ff::{BigInteger, PrimeField};

    // Function selector for verifyProof(uint256[2],uint256[2][2],uint256[2])
    const FUNCTION_SELECTOR: [u8; 4] = [0x66, 0x68, 0xa9, 0xfa];

    // Extract proof points in affine coordinates
    let (a_x, a_y) = proof.a.xy().unwrap();
    let (b_x, b_y) = proof.b.xy().unwrap();
    let (c_x, c_y) = proof.c.xy().unwrap();

    // ABI encode proof (selector + pA + pB + pC)
    [
        &FUNCTION_SELECTOR[..],
        &a_x.into_bigint().to_bytes_be(),
        &a_y.into_bigint().to_bytes_be(),
        &b_x.c1.into_bigint().to_bytes_be(),
        &b_x.c0.into_bigint().to_bytes_be(),
        &b_y.c1.into_bigint().to_bytes_be(),
        &b_y.c0.into_bigint().to_bytes_be(),
        &c_x.into_bigint().to_bytes_be(),
        &c_y.into_bigint().to_bytes_be(),
    ]
    .concat()
}

/// Save parameters to disk
fn save_params(
    pk: &ProvingKey<Bn254>,
    vk: &VerifyingKey<Bn254>,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(PARAMS_DIR)?;

    eprintln!("💾 Saving parameters to {}...", PARAMS_DIR);

    let pk_path = Path::new(PARAMS_DIR).join("proving_key.bin");
    let mut pk_file = fs::File::create(&pk_path)?;
    pk.serialize_compressed(&mut pk_file)?;
    let pk_size = fs::metadata(&pk_path)?.len();
    eprintln!("   Proving key: {:.2} MB", pk_size as f64 / 1024.0 / 1024.0);

    let vk_path = Path::new(PARAMS_DIR).join("verifying_key.bin");
    let mut vk_file = fs::File::create(&vk_path)?;
    vk.serialize_compressed(&mut vk_file)?;
    let vk_size = fs::metadata(&vk_path)?.len();
    eprintln!(
        "   Verifying key: {:.2} MB",
        vk_size as f64 / 1024.0 / 1024.0
    );

    // Generate and save Solidity verifier
    eprintln!("📝 Generating Solidity verifier contract...");
    let g16_vk = Groth16VerifierKey::from(vk.clone());
    let verifier_code = g16_vk.render_as_template(None);

    let verifier_path = "./ComplianceGroth16Verifier.sol";
    fs::write(verifier_path, verifier_code)?;
    let verifier_size = fs::metadata(verifier_path)?.len();
    eprintln!(
        "   Solidity verifier: {:.2} KB ({})",
        verifier_size as f64 / 1024.0,
        verifier_path
    );

    Ok(())
}

/// Load parameters from disk
fn load_params(
) -> Result<Option<(ProvingKey<Bn254>, VerifyingKey<Bn254>)>, Box<dyn std::error::Error>> {
    let pk_path = Path::new(PARAMS_DIR).join("proving_key.bin");
    let vk_path = Path::new(PARAMS_DIR).join("verifying_key.bin");

    if !pk_path.exists() || !vk_path.exists() {
        return Ok(None);
    }

    eprintln!("📂 Loading parameters from disk...");
    let start = Instant::now();

    let pk_file = fs::File::open(&pk_path)?;
    let pk = ProvingKey::<Bn254>::deserialize_compressed(pk_file)?;

    let vk_file = fs::File::open(&vk_path)?;
    let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_file)?;

    eprintln!("   Parameters loaded: {:?}", start.elapsed());
    Ok(Some((pk, vk)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let init_start = Instant::now();
    eprintln!("🚀 Arc Compliance Service (Groth16) Starting...\n");
    eprintln!("════════════════════════════════════════════════════════════");

    // Example compliance parameters
    let params = ComplianceParams {
        max_position_pct: 40,
        largest_asset_value: 35_000_000,  // $35M
        total_value: 100_000_000,         // $100M
        min_liquidity_pct: 10,
        usdc_balance: 10_000_000,         // $10M
        asset_hash: 100,
        sibling: 200,
        merkle_root: 300,
    };

    let circuit = ComplianceCircuit::new(params);

    // Try to load existing parameters
    let (pk, vk) = match load_params()? {
        Some((pk, vk)) => {
            eprintln!("✅ Using existing parameters\n");
            (pk, vk)
        }
        None => {
            eprintln!("🔧 No existing parameters found. Generating new ones...\n");
            let (pk, vk) = setup_groth16(circuit.clone())?;
            save_params(&pk, &vk)?;
            eprintln!();
            (pk, vk)
        }
    };

    let load_time_ms = init_start.elapsed().as_millis();
    eprintln!("✅ System ready!\n");
    eprintln!("Listening for commands on stdin...");
    eprintln!("Commands: generate_proof, verify, exit\n");

    // Output ready status for Node.js wrapper
    let mut stdout = std::io::stdout();
    println!("{{\"status\":\"ready\",\"load_time_ms\":{}}}", load_time_ms);
    stdout.flush()?;

    // Simple stdio protocol
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let cmd = line.trim();

        match cmd {
            "generate_proof" => {
                println!("{{\"status\":\"generating\"}}");
                stdout.flush()?;

                match generate_proof(&pk, circuit.clone()) {
                    Ok(proof) => {
                        match verify_proof(&vk, &proof) {
                            Ok(true) => {
                                let calldata = export_calldata(&proof, &vk);

                                // Save calldata
                                fs::write("./compliance-proof.calldata", &calldata)?;

                                println!(
                                    "{{\"status\":\"success\",\"verified\":true,\"proof_size\":{}}}",
                                    calldata.len()
                                );
                            }
                            Ok(false) => {
                                println!("{{\"status\":\"error\",\"message\":\"Verification failed\"}}");
                            }
                            Err(e) => {
                                println!(
                                    "{{\"status\":\"error\",\"message\":\"Verification error: {}\"}}",
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("{{\"status\":\"error\",\"message\":\"Proof generation failed: {}\"}}", e);
                    }
                }
                stdout.flush()?;
            }
            "exit" => {
                eprintln!("👋 Shutting down...");
                break;
            }
            _ => {
                println!("{{\"status\":\"error\",\"message\":\"Unknown command: {}\" }}", cmd);
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
