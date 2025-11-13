//! Position Limit Circuit
//!
//! Proves that no single asset position exceeds the specified percentage limit
//! of the total portfolio without revealing exact position sizes.
//!
//! Public Inputs:
//! - max_position_percentage: Maximum allowed percentage (e.g., 40 for 40%)
//!
//! Private Inputs:
//! - asset_values: Array of asset values in the portfolio
//! - total_portfolio_value: Sum of all asset values
//!
//! Constraints:
//! - For each asset: (asset_value * 100) / total_portfolio_value <= max_position_percentage
//! - Sum of asset_values == total_portfolio_value (integrity check)

use bellpepper_core::{Circuit, ConstraintSystem, SynthesisError};
use ff::PrimeField;
use serde::{Deserialize, Serialize};
use crate::range_proof;

/// Maximum number of assets supported in a single proof
pub const MAX_ASSETS: usize = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionLimitCircuit<F: PrimeField> {
    /// Maximum allowed percentage for a single position (public input)
    pub max_position_percentage: Option<u64>,

    /// Individual asset values (private witness)
    pub asset_values: Vec<Option<F>>,

    /// Total portfolio value (private witness)
    pub total_portfolio_value: Option<F>,
}

impl<F: PrimeField> PositionLimitCircuit<F> {
    /// Create a new position limit circuit
    pub fn new(
        max_position_percentage: u64,
        asset_values: Vec<F>,
        total_portfolio_value: F,
    ) -> Self {
        Self {
            max_position_percentage: Some(max_position_percentage),
            asset_values: asset_values.into_iter().map(Some).collect(),
            total_portfolio_value: Some(total_portfolio_value),
        }
    }

    /// Create an empty circuit for setup (no witness data)
    pub fn empty(num_assets: usize) -> Self {
        Self {
            max_position_percentage: None,
            asset_values: vec![None; num_assets],
            total_portfolio_value: None,
        }
    }
}

impl<F: PrimeField> Circuit<F> for PositionLimitCircuit<F> {
    fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // Allocate public input: max_position_percentage
        let max_percentage_var = cs.alloc_input(
            || "max_position_percentage",
            || {
                self.max_position_percentage
                    .map(|p| F::from(p))
                    .ok_or(SynthesisError::AssignmentMissing)
            },
        )?;

        // Allocate private input: total_portfolio_value
        let total_value_var = cs.alloc(
            || "total_portfolio_value",
            || {
                self.total_portfolio_value
                    .ok_or(SynthesisError::AssignmentMissing)
            },
        )?;

        // Allocate asset values and compute sum
        let mut sum_lc = bellpepper_core::LinearCombination::zero();

        for (i, asset_value) in self.asset_values.iter().enumerate() {
            let asset_var = cs.alloc(
                || format!("asset_value_{}", i),
                || asset_value.ok_or(SynthesisError::AssignmentMissing),
            )?;

            sum_lc = sum_lc + asset_var;

            // Compute percentage for this asset: (asset_value * 100) / total_value
            // We enforce: asset_value * 100 <= max_percentage * total_value
            // Rearranged: asset_value * 100 - max_percentage * total_value <= 0

            let asset_percentage = cs.alloc(
                || format!("asset_{}_percentage", i),
                || {
                    let asset_val = asset_value.ok_or(SynthesisError::AssignmentMissing)?;
                    let total_val = self
                        .total_portfolio_value
                        .ok_or(SynthesisError::AssignmentMissing)?;

                    // Compute: (asset_val / total_val) * 100
                    let total_inv: Option<F> = total_val.invert().into();
                    let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                    let percentage = asset_val * total_inv * F::from(100u64);
                    Ok(percentage)
                },
            )?;

            // Enforce: asset_percentage * total_value = asset_value * 100
            cs.enforce(
                || format!("asset_{}_percentage_constraint", i),
                |lc| lc + asset_percentage,
                |lc| lc + total_value_var,
                |lc| lc + (F::from(100u64), asset_var),
            );

            // Now enforce: asset_percentage <= max_percentage
            // We do this by enforcing: max_percentage - asset_percentage >= 0
            let diff = cs.alloc(
                || format!("diff_{}", i),
                || {
                    let max_p = self
                        .max_position_percentage
                        .map(F::from)
                        .ok_or(SynthesisError::AssignmentMissing)?;
                    let asset_val = asset_value.ok_or(SynthesisError::AssignmentMissing)?;
                    let total_val = self
                        .total_portfolio_value
                        .ok_or(SynthesisError::AssignmentMissing)?;

                    let total_inv: Option<F> = total_val.invert().into();
                    let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                    let asset_p = asset_val * total_inv * F::from(100u64);
                    let diff = max_p - asset_p;
                    Ok(diff)
                },
            )?;

            // Enforce: diff = max_percentage - asset_percentage
            cs.enforce(
                || format!("diff_{}_constraint", i),
                |lc| lc + CS::one(),
                |lc| lc + diff,
                |lc| lc + max_percentage_var - asset_percentage,
            );

            // Enforce diff >= 0 using range proof (bit decomposition)
            // This ensures asset_percentage <= max_percentage
            let diff_val = match (self.max_position_percentage, *asset_value, self.total_portfolio_value) {
                (Some(max_p), Some(asset_val), Some(total_val)) => {
                    let total_inv: Option<F> = total_val.invert().into();
                    let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                    let asset_p = asset_val * total_inv * F::from(100u64);
                    Some(F::from(max_p) - asset_p)
                }
                _ => None,
            };

            range_proof::decompose_allocated_value(
                cs.namespace(|| format!("range_proof_{}", i)),
                diff,
                diff_val,
                32, // 32 bits is enough for percentage differences
                &format!("diff_{}_bits", i),
            )?;
        }

        // Enforce: sum of all asset values equals total_portfolio_value
        cs.enforce(
            || "sum_equals_total",
            |lc| lc + CS::one(),
            |lc| lc + total_value_var,
            |_| sum_lc,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_position_limit_circuit_valid() {
        // Portfolio with 4 assets, max 40% per position
        // Total: 100M
        // Assets: 35M, 25M, 25M, 15M (all under 40%)
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let assets = vec![
            Fp::from(35_000_000u64), // 35%
            Fp::from(25_000_000u64), // 25%
            Fp::from(25_000_000u64), // 25%
            Fp::from(15_000_000u64), // 15%
        ];

        let circuit = PositionLimitCircuit::new(40, assets, total);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
        println!("Num constraints: {}", cs.num_constraints());
    }

    #[test]
    fn test_position_limit_circuit_violation() {
        // Portfolio with position exceeding limit
        // Total: 100M
        // Assets: 45M, 30M, 25M (45% > 40% limit)
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let assets = vec![
            Fp::from(45_000_000u64), // 45% - EXCEEDS LIMIT
            Fp::from(30_000_000u64), // 30%
            Fp::from(25_000_000u64), // 25%
        ];

        let circuit = PositionLimitCircuit::new(40, assets, total);

        // This should fail to synthesize or be unsatisfiable
        let result = circuit.synthesize(&mut cs);
        // Note: Depending on circuit implementation, this might synthesize
        // but cs.is_satisfied() should return false
        if result.is_ok() {
            assert!(!cs.is_satisfied(), "Circuit should not be satisfied when position exceeds limit");
        }
    }

    #[test]
    fn test_position_limit_at_boundary() {
        // Test exactly at the limit
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let assets = vec![
            Fp::from(40_000_000u64), // Exactly 40%
            Fp::from(35_000_000u64), // 35%
            Fp::from(25_000_000u64), // 25%
        ];

        let circuit = PositionLimitCircuit::new(40, assets, total);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
    }
}
