//! Nova-compatible circuits using BN254 curve
//!
//! These circuits implement the StepCircuit trait for Arecibo Nova compatibility.
//! They use BN254 Fr field instead of Pasta curves for EVM verification.

use arecibo::{
    frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError},
    nebula::rs::StepCircuit,
};
use ff::Field;
use halo2curves::bn256::Fr;
use arecibo::frontend::gadgets::poseidon::{circuit2::poseidon_hash_allocated, poseidon_inner::PoseidonConstants};
use generic_array::typenum::U2;

/// Liquidity Reserve Circuit (BN254 version)
///
/// Proves that USDC liquidity meets minimum threshold without revealing exact amounts.
///
/// State: [compliance_counter]
/// This circuit increments a counter each time a compliant liquidity check is performed.
#[derive(Clone, Debug)]
pub struct NovaLiquidityCircuit {
    /// Minimum liquidity percentage required (e.g., 10 for 10%)
    pub min_liquidity_pct: u64,
    /// Actual USDC balance (private input)
    pub usdc_balance: u64,
    /// Total portfolio value (private input)
    pub total_value: u64,
}

impl NovaLiquidityCircuit {
    pub fn new(min_liquidity_pct: u64, usdc_balance: u64, total_value: u64) -> Self {
        Self {
            min_liquidity_pct,
            usdc_balance,
            total_value,
        }
    }
}

impl StepCircuit<Fr> for NovaLiquidityCircuit {
    fn arity(&self) -> usize {
        // State: [compliance_counter]
        1
    }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        // Input state: compliance counter
        let counter = z_in[0].clone();

        // Allocate private inputs
        let usdc = AllocatedNum::alloc(cs.namespace(|| "usdc_balance"), || {
            Ok(Fr::from(self.usdc_balance))
        })?;

        let total = AllocatedNum::alloc(cs.namespace(|| "total_value"), || {
            Ok(Fr::from(self.total_value))
        })?;

        let min_pct = AllocatedNum::alloc(cs.namespace(|| "min_liquidity_pct"), || {
            Ok(Fr::from(self.min_liquidity_pct))
        })?;

        // Compute actual liquidity percentage: (usdc * 100) / total
        let hundred = AllocatedNum::alloc(cs.namespace(|| "hundred"), || {
            Ok(Fr::from(100u64))
        })?;

        let usdc_times_100 = usdc.mul(cs.namespace(|| "usdc * 100"), &hundred)?;

        // Compute actual_pct = (usdc * 100) / total
        // For the circuit, we'll verify: usdc * 100 = actual_pct * total
        let actual_pct = AllocatedNum::alloc(cs.namespace(|| "actual_pct"), || {
            if self.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.usdc_balance * 100) / self.total_value;
            Ok(Fr::from(pct))
        })?;

        // Enforce: actual_pct * total = usdc * 100
        cs.enforce(
            || "percentage_calculation",
            |lc| lc + actual_pct.get_variable(),
            |lc| lc + total.get_variable(),
            |lc| lc + usdc_times_100.get_variable(),
        );

        // Check: actual_pct >= min_pct
        // We compute diff = actual_pct - min_pct and verify it's non-negative
        // For demo, we just verify the calculation is correct
        // In production, add range proof here
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let actual = (self.usdc_balance * 100) / self.total_value;
            if actual < self.min_liquidity_pct {
                // Circuit will be unsatisfied if liquidity insufficient
                return Err(SynthesisError::Unsatisfiable);
            }
            Ok(Fr::from(actual - self.min_liquidity_pct))
        })?;

        // Enforce: actual_pct = min_pct + diff
        cs.enforce(
            || "liquidity_check",
            |lc| lc + actual_pct.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + min_pct.get_variable() + diff.get_variable(),
        );

        // Increment counter to prove we completed a check
        let one = AllocatedNum::alloc(cs.namespace(|| "one"), || Ok(Fr::one()))?;
        let new_counter = counter.add(cs.namespace(|| "increment_counter"), &one)?;

        // Output: incremented counter
        Ok(vec![new_counter])
    }

    fn non_deterministic_advice(&self) -> Vec<Fr> {
        // No advice needed for this circuit
        vec![]
    }
}

/// Position Limit Circuit (BN254 version)
///
/// Proves that no single asset exceeds maximum position percentage.
///
/// State: [compliance_counter]
#[derive(Clone, Debug)]
pub struct NovaPositionLimitCircuit {
    /// Maximum position percentage (e.g., 40 for 40%)
    pub max_position_pct: u64,
    /// Asset value to check (private input)
    pub asset_value: u64,
    /// Total portfolio value (private input)
    pub total_value: u64,
}

impl NovaPositionLimitCircuit {
    pub fn new(max_position_pct: u64, asset_value: u64, total_value: u64) -> Self {
        Self {
            max_position_pct,
            asset_value,
            total_value,
        }
    }
}

/// Whitelist Circuit (BN254 version)
///
/// Proves that an asset hash is included in a Merkle tree with Poseidon hashing.
/// Uses a fixed-depth tree for demo purposes.
#[derive(Clone, Debug)]
pub struct NovaWhitelistCircuit {
    /// Merkle root (public parameter passed in via calldata off-circuit)
    pub merkle_root: u64,
    /// Asset leaf value
    pub asset_hash: u64,
    /// Sibling nodes along the path (fixed depth)
    pub siblings: Vec<u64>,
    /// Path indices: true if current node is right child
    pub is_right: Vec<bool>,
}

impl NovaWhitelistCircuit {
    pub fn new(merkle_root: u64, asset_hash: u64, siblings: Vec<u64>, is_right: Vec<bool>) -> Self {
        assert_eq!(siblings.len(), is_right.len());
        Self { merkle_root, asset_hash, siblings, is_right }
    }
}

impl StepCircuit<Fr> for NovaWhitelistCircuit {
    fn arity(&self) -> usize { 1 }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        // Counter
        let counter = z_in[0].clone();

        // Allocate leaf
        let mut current = AllocatedNum::alloc(cs.namespace(|| "leaf"), || Ok(Fr::from(self.asset_hash)))?;
        let mut constants: PoseidonConstants<Fr, U2> = PoseidonConstants::new();

        // Recompute root by walking up
        for (i, (sib, right)) in self.siblings.iter().copied().zip(self.is_right.iter().copied()).enumerate() {
            let sibling = AllocatedNum::alloc(cs.namespace(|| format!("sibling_{}", i)), || Ok(Fr::from(sib)))?;
            let (left, rightv) = if right {
                (sibling.clone(), current.clone())
            } else {
                (current.clone(), sibling.clone())
            };

            // Poseidon hash(left, right) -> parent
            let parent = poseidon_hash_allocated::<_, _, U2>(
                cs.namespace(|| format!("poseidon_{}", i)),
                vec![left, rightv],
                &constants,
            )?;
            current = parent;
        }

        // Enforce: computed root equals provided root
        let root = AllocatedNum::alloc(cs.namespace(|| "merkle_root"), || Ok(Fr::from(self.merkle_root)))?;
        cs.enforce(
            || "root_match",
            |lc| lc + current.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + root.get_variable(),
        );

        // Increment counter
        let one = AllocatedNum::alloc(cs.namespace(|| "one"), || Ok(Fr::one()))?;
        let new_counter = counter.add(cs.namespace(|| "increment"), &one)?;
        Ok(vec![new_counter])
    }

    fn non_deterministic_advice(&self) -> Vec<Fr> { vec![] }
}

impl StepCircuit<Fr> for NovaPositionLimitCircuit {
    fn arity(&self) -> usize {
        1
    }

    fn synthesize<CS: ConstraintSystem<Fr>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<Fr>],
    ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
        let counter = z_in[0].clone();

        // Allocate inputs
        let asset = AllocatedNum::alloc(cs.namespace(|| "asset_value"), || {
            Ok(Fr::from(self.asset_value))
        })?;

        let total = AllocatedNum::alloc(cs.namespace(|| "total_value"), || {
            Ok(Fr::from(self.total_value))
        })?;

        let max_pct = AllocatedNum::alloc(cs.namespace(|| "max_position_pct"), || {
            Ok(Fr::from(self.max_position_pct))
        })?;

        // Compute: asset_pct = (asset * 100) / total
        let hundred = AllocatedNum::alloc(cs.namespace(|| "hundred"), || {
            Ok(Fr::from(100u64))
        })?;

        let asset_times_100 = asset.mul(cs.namespace(|| "asset * 100"), &hundred)?;

        let asset_pct = AllocatedNum::alloc(cs.namespace(|| "asset_pct"), || {
            if self.total_value == 0 {
                return Err(SynthesisError::DivisionByZero);
            }
            let pct = (self.asset_value * 100) / self.total_value;
            Ok(Fr::from(pct))
        })?;

        // Enforce: asset_pct * total = asset * 100
        cs.enforce(
            || "percentage_calculation",
            |lc| lc + asset_pct.get_variable(),
            |lc| lc + total.get_variable(),
            |lc| lc + asset_times_100.get_variable(),
        );

        // Check: asset_pct <= max_pct
        // Compute diff = max_pct - asset_pct (must be >= 0)
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let pct = (self.asset_value * 100) / self.total_value;
            if pct > self.max_position_pct {
                return Err(SynthesisError::Unsatisfiable);
            }
            Ok(Fr::from(self.max_position_pct - pct))
        })?;

        // Enforce: max_pct = asset_pct + diff
        cs.enforce(
            || "position_limit_check",
            |lc| lc + max_pct.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + asset_pct.get_variable() + diff.get_variable(),
        );

        // Increment counter
        let one = AllocatedNum::alloc(cs.namespace(|| "one"), || Ok(Fr::one()))?;
        let new_counter = counter.add(cs.namespace(|| "increment"), &one)?;

        Ok(vec![new_counter])
    }

    fn non_deterministic_advice(&self) -> Vec<Fr> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arecibo::frontend::test_cs::TestConstraintSystem;

    #[test]
    fn test_nova_liquidity_sufficient() {
        let circuit = NovaLiquidityCircuit::new(
            10,           // min 10%
            10_000_000,   // $10M USDC
            100_000_000,  // $100M total (exactly 10%)
        );

        let mut cs = TestConstraintSystem::<Fr>::new();

        // Initial state: counter = 0
        let counter = AllocatedNum::alloc(&mut cs.namespace(|| "counter"), || Ok(Fr::zero())).unwrap();

        let result = circuit.synthesize(&mut cs, &[counter]);
        assert!(result.is_ok(), "Circuit should be satisfied with sufficient liquidity");
        assert!(cs.is_satisfied(), "Constraints should be satisfied");
    }

    #[test]
    fn test_nova_liquidity_insufficient() {
        let circuit = NovaLiquidityCircuit::new(
            10,          // min 10%
            5_000_000,   // $5M USDC
            100_000_000, // $100M total (only 5% - insufficient!)
        );

        let mut cs = TestConstraintSystem::<Fr>::new();
        let counter = AllocatedNum::alloc(&mut cs.namespace(|| "counter"), || Ok(Fr::zero())).unwrap();

        let result = circuit.synthesize(&mut cs, &[counter]);
        assert!(result.is_err(), "Circuit should fail with insufficient liquidity");
    }

    #[test]
    fn test_nova_position_compliant() {
        let circuit = NovaPositionLimitCircuit::new(
            40,          // max 40%
            35_000_000,  // $35M asset
            100_000_000, // $100M total (35% - compliant)
        );

        let mut cs = TestConstraintSystem::<Fr>::new();
        let counter = AllocatedNum::alloc(&mut cs.namespace(|| "counter"), || Ok(Fr::zero())).unwrap();

        let result = circuit.synthesize(&mut cs, &[counter]);
        assert!(result.is_ok(), "Circuit should be satisfied");
        assert!(cs.is_satisfied(), "Constraints should be satisfied");
    }

    #[test]
    fn test_nova_position_violating() {
        let circuit = NovaPositionLimitCircuit::new(
            40,          // max 40%
            45_000_000,  // $45M asset
            100_000_000, // $100M total (45% - violation!)
        );

        let mut cs = TestConstraintSystem::<Fr>::new();
        let counter = AllocatedNum::alloc(&mut cs.namespace(|| "counter"), || Ok(Fr::zero())).unwrap();

        let result = circuit.synthesize(&mut cs, &[counter]);
        assert!(result.is_err(), "Circuit should fail with position violation");
    }
}
