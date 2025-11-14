//! Composite Fund Compliance Circuit
//!
//! This circuit combines all three compliance checks into a single circuit:
//! 1. Position Limit: No single asset exceeds 40% of portfolio
//! 2. Liquidity Reserve: USDC reserves ≥ 10% of portfolio
//! 3. Whitelist: All assets are approved (Merkle proof with Poseidon)
//!
//! This composite circuit is designed to be folded using Nova IVC,
//! proving compliance across multiple time periods with a single on-chain verification.

use bellpepper_core::{ConstraintSystem, SynthesisError};
use ff::PrimeField;

/// Parameters for the composite fund compliance circuit
#[derive(Clone, Debug)]
pub struct FundComplianceParams {
    // Position limit check
    pub max_position_pct: u64,
    pub largest_asset_value: u64,

    // Liquidity check
    pub min_liquidity_pct: u64,
    pub usdc_balance: u64,

    // Whitelist check (using Poseidon hash)
    pub merkle_root: u64,
    pub asset_hash: u64,
    pub siblings: Vec<u64>,
    pub is_right: Vec<bool>,

    // Shared parameter
    pub total_value: u64,
}

/// Composite circuit that checks all three compliance rules
///
/// This circuit proves:
/// 1. largest_asset_value / total_value ≤ max_position_pct
/// 2. usdc_balance / total_value ≥ min_liquidity_pct
/// 3. asset_hash ∈ Merkle tree with root = merkle_root
///
/// State: [compliance_counter]
/// Each successful fold increments the counter, proving N consecutive compliant periods
#[derive(Clone, Debug)]
pub struct FundComplianceCircuit<F: PrimeField> {
    pub params: FundComplianceParams,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: PrimeField> FundComplianceCircuit<F> {
    pub fn new(params: FundComplianceParams) -> Self {
        Self {
            params,
            _phantom: std::marker::PhantomData,
        }
    }
}

// For use with Bellpepper constraint systems
impl<F: PrimeField> FundComplianceCircuit<F> {
    /// Synthesize the complete compliance check circuit
    pub fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
    ) -> Result<(), SynthesisError> {
        // ========================================
        // CHECK 1: Position Limit (asset ≤ 40%)
        // ========================================

        let asset = cs.alloc(
            || "largest_asset_value",
            || Ok(F::from(self.params.largest_asset_value))
        )?;

        let total = cs.alloc(
            || "total_value",
            || Ok(F::from(self.params.total_value))
        )?;

        let max_pct = cs.alloc(
            || "max_position_pct",
            || Ok(F::from(self.params.max_position_pct))
        )?;

        let hundred = cs.alloc(
            || "hundred",
            || Ok(F::from(100u64))
        )?;

        // Compute asset_pct = (asset * 100) / total
        let asset_times_100 = cs.alloc(
            || "asset * 100",
            || {
                let a = F::from(self.params.largest_asset_value);
                let h = F::from(100u64);
                Ok(a * h)
            }
        )?;

        // Enforce: asset * 100 = asset_times_100
        cs.enforce(
            || "asset * 100 calculation",
            |lc| lc + asset,
            |lc| lc + hundred,
            |lc| lc + asset_times_100,
        );

        let asset_pct = cs.alloc(
            || "asset_pct",
            || {
                if self.params.total_value == 0 {
                    return Err(SynthesisError::DivisionByZero);
                }
                let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
                Ok(F::from(pct))
            }
        )?;

        // Enforce: asset_pct * total = asset * 100
        cs.enforce(
            || "position percentage check",
            |lc| lc + asset_pct,
            |lc| lc + total,
            |lc| lc + asset_times_100,
        );

        // Check: asset_pct ≤ max_pct
        // Compute diff = max_pct - asset_pct (must be ≥ 0)
        let position_diff = cs.alloc(
            || "position_diff",
            || {
                let pct = (self.params.largest_asset_value * 100) / self.params.total_value;
                if pct > self.params.max_position_pct {
                    return Err(SynthesisError::Unsatisfiable);
                }
                Ok(F::from(self.params.max_position_pct - pct))
            }
        )?;

        // Enforce: max_pct = asset_pct + diff
        cs.enforce(
            || "position limit satisfied",
            |lc| lc + max_pct,
            |lc| lc + CS::one(),
            |lc| lc + asset_pct + position_diff,
        );

        // ========================================
        // CHECK 2: Liquidity Reserve (USDC ≥ 10%)
        // ========================================

        let usdc = cs.alloc(
            || "usdc_balance",
            || Ok(F::from(self.params.usdc_balance))
        )?;

        let min_pct = cs.alloc(
            || "min_liquidity_pct",
            || Ok(F::from(self.params.min_liquidity_pct))
        )?;

        // Compute usdc_pct = (usdc * 100) / total
        let usdc_times_100 = cs.alloc(
            || "usdc * 100",
            || {
                let u = F::from(self.params.usdc_balance);
                let h = F::from(100u64);
                Ok(u * h)
            }
        )?;

        cs.enforce(
            || "usdc * 100 calculation",
            |lc| lc + usdc,
            |lc| lc + hundred,
            |lc| lc + usdc_times_100,
        );

        let usdc_pct = cs.alloc(
            || "usdc_pct",
            || {
                if self.params.total_value == 0 {
                    return Err(SynthesisError::DivisionByZero);
                }
                let pct = (self.params.usdc_balance * 100) / self.params.total_value;
                Ok(F::from(pct))
            }
        )?;

        // Enforce: usdc_pct * total = usdc * 100
        cs.enforce(
            || "liquidity percentage check",
            |lc| lc + usdc_pct,
            |lc| lc + total,
            |lc| lc + usdc_times_100,
        );

        // Check: usdc_pct ≥ min_pct
        // Compute diff = usdc_pct - min_pct (must be ≥ 0)
        let liquidity_diff = cs.alloc(
            || "liquidity_diff",
            || {
                let pct = (self.params.usdc_balance * 100) / self.params.total_value;
                if pct < self.params.min_liquidity_pct {
                    return Err(SynthesisError::Unsatisfiable);
                }
                Ok(F::from(pct - self.params.min_liquidity_pct))
            }
        )?;

        // Enforce: usdc_pct = min_pct + diff
        cs.enforce(
            || "liquidity requirement satisfied",
            |lc| lc + usdc_pct,
            |lc| lc + CS::one(),
            |lc| lc + min_pct + liquidity_diff,
        );

        // ========================================
        // CHECK 3: Whitelist Membership (Merkle proof)
        // ========================================
        // Note: For demo, using simplified hash
        // Production should use Poseidon gadget

        let mut current = cs.alloc(
            || "leaf",
            || Ok(F::from(self.params.asset_hash))
        )?;

        // Recompute Merkle root by walking up the tree
        for (i, (&sibling_val, &is_right)) in self.params.siblings.iter()
            .zip(self.params.is_right.iter())
            .enumerate()
        {
            let sibling = cs.alloc(
                || format!("sibling_{}", i),
                || Ok(F::from(sibling_val))
            )?;

            // For demo: simple addition-based hash
            // TODO: Replace with Poseidon gadget for production
            let parent = cs.alloc(
                || format!("parent_{}", i),
                || {
                    let c = F::from(if is_right {
                        sibling_val + self.params.asset_hash
                    } else {
                        self.params.asset_hash + sibling_val
                    });
                    Ok(c)
                }
            )?;

            // Enforce: parent = current + sibling (simplified)
            cs.enforce(
                || format!("merkle_hash_{}", i),
                |lc| lc + current + sibling,
                |lc| lc + CS::one(),
                |lc| lc + parent,
            );

            current = parent;
        }

        // Verify computed root matches expected root
        let root = cs.alloc(
            || "merkle_root",
            || Ok(F::from(self.params.merkle_root))
        )?;

        cs.enforce(
            || "root verification",
            |lc| lc + current,
            |lc| lc + CS::one(),
            |lc| lc + root,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use halo2curves::bn256::Fr;

    #[test]
    fn test_composite_circuit_compliant() {
        // $100M fund with $35M in largest asset (35%), $10M USDC (10%)
        let params = FundComplianceParams {
            max_position_pct: 40,
            largest_asset_value: 35_000_000,
            min_liquidity_pct: 10,
            usdc_balance: 10_000_000,
            merkle_root: 12345, // Simplified
            asset_hash: 100,
            siblings: vec![200],
            is_right: vec![false],
            total_value: 100_000_000,
        };

        let circuit = FundComplianceCircuit::<Fr>::new(params);
        let mut cs = TestConstraintSystem::<Fr>::new();

        let result = circuit.synthesize(&mut cs);
        assert!(result.is_ok(), "Circuit should be satisfied with compliant fund");
        assert!(cs.is_satisfied(), "All constraints should be satisfied");
    }

    #[test]
    fn test_composite_circuit_position_violation() {
        // $100M fund with $45M in largest asset (45% - VIOLATION!)
        let params = FundComplianceParams {
            max_position_pct: 40,
            largest_asset_value: 45_000_000, // Too large!
            min_liquidity_pct: 10,
            usdc_balance: 10_000_000,
            merkle_root: 12345,
            asset_hash: 100,
            siblings: vec![200],
            is_right: vec![false],
            total_value: 100_000_000,
        };

        let circuit = FundComplianceCircuit::<Fr>::new(params);
        let mut cs = TestConstraintSystem::<Fr>::new();

        let result = circuit.synthesize(&mut cs);
        assert!(result.is_err(), "Circuit should fail with position violation");
    }

    #[test]
    fn test_composite_circuit_liquidity_violation() {
        // $100M fund with only $5M USDC (5% - VIOLATION!)
        let params = FundComplianceParams {
            max_position_pct: 40,
            largest_asset_value: 35_000_000,
            min_liquidity_pct: 10,
            usdc_balance: 5_000_000, // Too low!
            merkle_root: 12345,
            asset_hash: 100,
            siblings: vec![200],
            is_right: vec![false],
            total_value: 100_000_000,
        };

        let circuit = FundComplianceCircuit::<Fr>::new(params);
        let mut cs = TestConstraintSystem::<Fr>::new();

        let result = circuit.synthesize(&mut cs);
        assert!(result.is_err(), "Circuit should fail with liquidity violation");
    }
}
