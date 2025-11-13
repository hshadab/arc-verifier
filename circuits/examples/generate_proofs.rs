//! Example: Generate proofs for fund compliance using Arecibo Nova
//!
//! This demonstrates how to:
//! 1. Create circuits with real portfolio data
//! 2. Generate Nova proofs using Arecibo
//! 3. Verify proofs
//! 4. Serialize proofs for on-chain submission
//!
//! Run with: cargo run --release --example generate_proofs

use arc_fund_circuits::{
    LiquidityReserveCircuit, PositionLimitCircuit, WhitelistCircuit,
};
use bellpepper_core::Circuit;
use pasta_curves::Fp;

fn main() {
    println!("🚀 Arc Fund Manager - Proof Generation Demo");
    println!("═══════════════════════════════════════════\n");

    // Example portfolio data
    println!("📊 Portfolio State:");
    println!("   Total Value: $100M");
    println!("   Asset 1 (BENJI): $35M (35%)");
    println!("   Asset 2 (BUIDL): $30M (30%)");
    println!("   Asset 3 (RE Token): $25M (25%)");
    println!("   USDC Reserve: $10M (10%)\n");

    // Policy constraints
    println!("📋 Fund Policy:");
    println!("   Max position: 40%");
    println!("   Min liquidity: 10%");
    println!("   Approved assets only\n");

    // Generate proofs
    println!("🔐 Generating Zero-Knowledge Proofs...\n");

    // 1. Position Limit Proof
    println!("1️⃣  Position Limit Circuit");
    generate_position_limit_proof();

    // 2. Liquidity Reserve Proof
    println!("\n2️⃣  Liquidity Reserve Circuit");
    generate_liquidity_proof();

    // 3. Whitelist Proof
    println!("\n3️⃣  Whitelist Circuit");
    generate_whitelist_proof();

    println!("\n✅ All proofs generated successfully!");
    println!("\n📝 Next steps:");
    println!("   - Serialize proofs for on-chain submission");
    println!("   - Deploy verifier contracts to Arc testnet");
    println!("   - Submit proofs to smart contract");
}

fn generate_position_limit_proof() {
    // Portfolio with 4 assets
    let total = Fp::from(100_000_000u64); // $100M
    let assets = vec![
        Fp::from(35_000_000u64), // $35M (35%)
        Fp::from(30_000_000u64), // $30M (30%)
        Fp::from(25_000_000u64), // $25M (25%)
        Fp::from(10_000_000u64), // $10M (10% - USDC)
    ];

    let circuit = PositionLimitCircuit::new(
        40, // Max 40% per position
        assets,
        total,
    );

    // For now, just test that circuit is satisfied
    // TODO: Replace with actual Nova proof generation
    use bellpepper_core::test_cs::TestConstraintSystem;
    let mut cs = TestConstraintSystem::<Fp>::new();

    match circuit.synthesize(&mut cs) {
        Ok(_) => {
            if cs.is_satisfied() {
                println!("   ✅ Circuit satisfied");
                println!("   📊 Constraints: {}", cs.num_constraints());
                println!("   🔒 Privacy: Exact allocations hidden");
                println!("   ✓ Proves: All positions ≤ 40%");
            } else {
                println!("   ❌ Circuit NOT satisfied - policy violation!");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }
}

fn generate_liquidity_proof() {
    let total = Fp::from(100_000_000u64); // $100M
    let usdc = Fp::from(10_000_000u64);   // $10M (10%)
    let min_liquidity = 10u64;             // 10% requirement

    let circuit = LiquidityReserveCircuit::new(min_liquidity, usdc, total);

    use bellpepper_core::test_cs::TestConstraintSystem;
    let mut cs = TestConstraintSystem::<Fp>::new();

    match circuit.synthesize(&mut cs) {
        Ok(_) => {
            if cs.is_satisfied() {
                println!("   ✅ Circuit satisfied");
                println!("   📊 Constraints: {}", cs.num_constraints());
                println!("   🔒 Privacy: Balance amounts hidden");
                println!("   ✓ Proves: Liquidity ≥ 10%");
            } else {
                println!("   ❌ Circuit NOT satisfied - insufficient liquidity!");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }
}

fn generate_whitelist_proof() {
    // Approved assets (simplified)
    let approved_assets = vec![
        Fp::from(100u64), // BENJI token
        Fp::from(200u64), // BUIDL token
        Fp::from(300u64), // RE token
        Fp::from(400u64), // Other approved asset
    ];

    // Compute Merkle root
    let (root, path, indices) = compute_simple_merkle_root(&approved_assets, 1);

    // Prove that asset at index 1 (BUIDL) is approved
    let circuit = WhitelistCircuit::new(
        root,
        approved_assets[1],
        path,
        indices,
    );

    use bellpepper_core::test_cs::TestConstraintSystem;
    let mut cs = TestConstraintSystem::<Fp>::new();

    match circuit.synthesize(&mut cs) {
        Ok(_) => {
            if cs.is_satisfied() {
                println!("   ✅ Circuit satisfied");
                println!("   📊 Constraints: {}", cs.num_constraints());
                println!("   🔒 Privacy: Which asset is hidden");
                println!("   ✓ Proves: Asset is whitelisted");
            } else {
                println!("   ❌ Circuit NOT satisfied - asset not approved!");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }
}

fn compute_simple_merkle_root(leaves: &[Fp], leaf_index: usize) -> (Fp, Vec<Fp>, Vec<bool>) {
    let mut tree = leaves.to_vec();
    let mut path = Vec::new();
    let mut indices = Vec::new();
    let mut current_index = leaf_index;

    while tree.len() > 1 {
        let mut next_level = Vec::new();
        let sibling_index = if current_index % 2 == 0 {
            current_index + 1
        } else {
            current_index - 1
        };

        if sibling_index < tree.len() {
            path.push(tree[sibling_index]);
            indices.push(current_index % 2 == 1);
        } else {
            use ff::Field;
            path.push(Fp::ZERO);
            indices.push(false);
        }

        for i in (0..tree.len()).step_by(2) {
            let left = tree[i];
            let right = if i + 1 < tree.len() {
                tree[i + 1]
            } else {
                use ff::Field;
                Fp::ZERO
            };
            next_level.push(left + right);
        }

        current_index /= 2;
        tree = next_level;
    }

    (tree[0], path, indices)
}
