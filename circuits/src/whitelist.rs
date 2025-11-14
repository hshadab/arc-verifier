//! Asset Whitelist Circuit using Merkle Proofs
//!
//! Proves that an asset address is in the approved whitelist without
//! revealing which specific asset it is.
//!
//! Public Inputs:
//! - merkle_root: Root of the Merkle tree containing approved assets
//!
//! Private Inputs:
//! - asset_hash: Hash of the asset address
//! - merkle_path: Path from leaf to root (sibling hashes)
//! - path_indices: Left/right indicators for path (0 = left, 1 = right)
//!
//! Constraints:
//! - Recompute Merkle root from asset_hash and path
//! - Computed root must equal public merkle_root

use bellpepper_core::{Circuit, ConstraintSystem, SynthesisError};
use ff::PrimeField;
use serde::{Deserialize, Serialize};

/// Maximum depth of Merkle tree (supports up to 2^20 = 1M assets)
pub const MAX_MERKLE_DEPTH: usize = 20;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhitelistCircuit<F: PrimeField> {
    /// Merkle root of approved assets (public input)
    pub merkle_root: Option<F>,

    /// Hash of the asset being checked (private witness)
    pub asset_hash: Option<F>,

    /// Sibling hashes along the path to root (private witness)
    pub merkle_path: Vec<Option<F>>,

    /// Path indices: 0 for left, 1 for right (private witness)
    pub path_indices: Vec<Option<bool>>,
}

impl<F: PrimeField> WhitelistCircuit<F> {
    /// Create a new whitelist circuit
    pub fn new(
        merkle_root: F,
        asset_hash: F,
        merkle_path: Vec<F>,
        path_indices: Vec<bool>,
    ) -> Self {
        assert_eq!(
            merkle_path.len(),
            path_indices.len(),
            "Path and indices must have same length"
        );

        Self {
            merkle_root: Some(merkle_root),
            asset_hash: Some(asset_hash),
            merkle_path: merkle_path.into_iter().map(Some).collect(),
            path_indices: path_indices.into_iter().map(Some).collect(),
        }
    }

    /// Create an empty circuit for setup (no witness data)
    pub fn empty(depth: usize) -> Self {
        Self {
            merkle_root: None,
            asset_hash: None,
            merkle_path: vec![None; depth],
            path_indices: vec![None; depth],
        }
    }
}

// Merkle hash function used by the circuit
// Default (no feature): simple addition-based hash for demo/testing
// Feature `poseidon`: placeholder for Poseidon gadget wiring

#[cfg(not(feature = "poseidon"))]
fn merkle_hash<F: PrimeField, CS: ConstraintSystem<F>>(
    mut cs: CS,
    left: bellpepper_core::Variable,
    right: bellpepper_core::Variable,
    left_val: Option<F>,
    right_val: Option<F>,
) -> Result<bellpepper_core::Variable, SynthesisError> {
    let hash = cs.alloc(
        || "merkle_hash_addition",
        || {
            let l = left_val.ok_or(SynthesisError::AssignmentMissing)?;
            let r = right_val.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(l + r)
        },
    )?;

    cs.enforce(
        || "hash = left + right",
        |lc| lc + CS::one(),
        |lc| lc + hash,
        |lc| lc + left + right,
    );

    Ok(hash)
}

#[cfg(feature = "poseidon")]
fn merkle_hash<F: PrimeField, CS: ConstraintSystem<F>>(
    _cs: CS,
    _left: bellpepper_core::Variable,
    _right: bellpepper_core::Variable,
    _left_val: Option<F>,
    _right_val: Option<F>,
) -> Result<bellpepper_core::Variable, SynthesisError> {
    // Placeholder: Poseidon gadget not wired in this crate yet.
    // Implement using a bellpepper-compatible Poseidon gadget or bridge.
    Err(SynthesisError::Unsatisfiable)
}

impl<F: PrimeField> Circuit<F> for WhitelistCircuit<F> {
    fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        assert_eq!(
            self.merkle_path.len(),
            self.path_indices.len(),
            "Merkle path and indices must have the same length"
        );

        // Allocate public input: merkle_root
        let root_var = cs.alloc_input(
            || "merkle_root",
            || self.merkle_root.ok_or(SynthesisError::AssignmentMissing),
        )?;

        // Allocate private input: asset_hash (leaf)
        let mut current_hash = cs.alloc(
            || "asset_hash",
            || self.asset_hash.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let mut current_val = self.asset_hash;

        // Traverse up the Merkle tree
        for (i, (sibling_opt, index_opt)) in self
            .merkle_path
            .iter()
            .zip(self.path_indices.iter())
            .enumerate()
        {
            // Allocate sibling hash
            let sibling = cs.alloc(
                || format!("sibling_{}", i),
                || sibling_opt.ok_or(SynthesisError::AssignmentMissing),
            )?;

            // Compute left and right directly based on index
            // No complex conditional constraints - just compute both orderings
            let (left_val, right_val, left_var, right_var) =
                match (current_val, *sibling_opt, *index_opt) {
                    (Some(cur), Some(sib), Some(is_right)) => {
                        if is_right {
                            // Current is on the right
                            (Some(sib), Some(cur), sibling, current_hash)
                        } else {
                            // Current is on the left
                            (Some(cur), Some(sib), current_hash, sibling)
                        }
                    }
                    _ => (None, None, current_hash, sibling), // Placeholder
                };

            // Compute parent hash
            current_hash = merkle_hash(
                cs.namespace(|| format!("hash_{}", i)),
                left_var,
                right_var,
                left_val,
                right_val,
            )?;

            current_val = match (left_val, right_val) {
                (Some(l), Some(r)) => Some(l + r),
                _ => None,
            };
        }

        // Enforce: computed root equals public merkle_root
        cs.enforce(
            || "root_constraint",
            |lc| lc + CS::one(),
            |lc| lc + current_hash,
            |lc| lc + root_var,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use ff::Field;
    use pasta_curves::Fp;

    fn compute_merkle_root(leaves: &[Fp], leaf_index: usize) -> (Fp, Vec<Fp>, Vec<bool>) {
        let mut tree = leaves.to_vec();
        let mut path = Vec::new();
        let mut indices = Vec::new();
        let mut current_index = leaf_index;

        // Build tree level by level
        while tree.len() > 1 {
            let mut next_level = Vec::new();
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            // Record sibling
            if sibling_index < tree.len() {
                path.push(tree[sibling_index]);
                indices.push(current_index % 2 == 1);
            } else {
                // No sibling, use zero
                path.push(Fp::ZERO);
                indices.push(false);
            }

            // Compute next level
            for i in (0..tree.len()).step_by(2) {
                let left = tree[i];
                let right = if i + 1 < tree.len() {
                    tree[i + 1]
                } else {
                    Fp::ZERO
                };
                next_level.push(left + right); // Simple hash
            }

            current_index /= 2;
            tree = next_level;
        }

        (tree[0], path, indices)
    }

    #[test]
    fn test_whitelist_circuit_valid() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        // Create a small Merkle tree with 4 assets
        let assets = vec![
            Fp::from(100u64), // Asset 0
            Fp::from(200u64), // Asset 1
            Fp::from(300u64), // Asset 2
            Fp::from(400u64), // Asset 3
        ];

        // Prove that asset 2 is in the tree
        let (root, path, indices) = compute_merkle_root(&assets, 2);

        let circuit = WhitelistCircuit::new(root, assets[2], path, indices);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
        println!("Whitelist circuit - Num constraints: {}", cs.num_constraints());
    }

    #[test]
    fn test_whitelist_circuit_invalid() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        // Create Merkle tree
        let assets = vec![
            Fp::from(100u64),
            Fp::from(200u64),
            Fp::from(300u64),
            Fp::from(400u64),
        ];

        let (root, path, indices) = compute_merkle_root(&assets, 2);

        // Try to prove a different asset is in the tree
        let fake_asset = Fp::from(999u64);
        let circuit = WhitelistCircuit::new(root, fake_asset, path, indices);

        let result = circuit.synthesize(&mut cs);
        if result.is_ok() {
            assert!(
                !cs.is_satisfied(),
                "Circuit should not be satisfied for non-whitelisted asset"
            );
        }
    }
}
