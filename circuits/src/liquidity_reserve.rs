//! Liquidity Reserve Circuit
//!
//! Proves that the fund maintains sufficient USDC liquidity reserves
//! without revealing the exact portfolio composition.
//!
//! Public Inputs:
//! - min_liquidity_percentage: Minimum required liquidity percentage (e.g., 10 for 10%)
//!
//! Private Inputs:
//! - usdc_balance: Current USDC balance
//! - total_portfolio_value: Total value of all holdings
//!
//! Constraints:
//! - (usdc_balance * 100) / total_portfolio_value >= min_liquidity_percentage

use bellpepper_core::{Circuit, ConstraintSystem, SynthesisError};
use ff::PrimeField;
use serde::{Deserialize, Serialize};
use crate::range_proof;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidityReserveCircuit<F: PrimeField> {
    /// Minimum required liquidity percentage (public input)
    pub min_liquidity_percentage: Option<u64>,

    /// Current USDC balance (private witness)
    pub usdc_balance: Option<F>,

    /// Total portfolio value (private witness)
    pub total_portfolio_value: Option<F>,
}

impl<F: PrimeField> LiquidityReserveCircuit<F> {
    /// Create a new liquidity reserve circuit
    pub fn new(min_liquidity_percentage: u64, usdc_balance: F, total_portfolio_value: F) -> Self {
        Self {
            min_liquidity_percentage: Some(min_liquidity_percentage),
            usdc_balance: Some(usdc_balance),
            total_portfolio_value: Some(total_portfolio_value),
        }
    }

    /// Create an empty circuit for setup (no witness data)
    pub fn empty() -> Self {
        Self {
            min_liquidity_percentage: None,
            usdc_balance: None,
            total_portfolio_value: None,
        }
    }
}

impl<F: PrimeField> Circuit<F> for LiquidityReserveCircuit<F> {
    fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // Allocate public input: min_liquidity_percentage
        let min_liquidity_var = cs.alloc_input(
            || "min_liquidity_percentage",
            || {
                self.min_liquidity_percentage
                    .map(F::from)
                    .ok_or(SynthesisError::AssignmentMissing)
            },
        )?;

        // Allocate private inputs
        let usdc_balance_var = cs.alloc(
            || "usdc_balance",
            || self.usdc_balance.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let total_value_var = cs.alloc(
            || "total_portfolio_value",
            || {
                self.total_portfolio_value
                    .ok_or(SynthesisError::AssignmentMissing)
            },
        )?;

        // Compute actual liquidity percentage: (usdc_balance * 100) / total_value
        let actual_liquidity_percentage = cs.alloc(
            || "actual_liquidity_percentage",
            || {
                let usdc = self.usdc_balance.ok_or(SynthesisError::AssignmentMissing)?;
                let total = self
                    .total_portfolio_value
                    .ok_or(SynthesisError::AssignmentMissing)?;

                let total_inv: Option<F> = total.invert().into();
                let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                let percentage = usdc * total_inv * F::from(100u64);
                Ok(percentage)
            },
        )?;

        // Enforce: actual_liquidity_percentage * total_value = usdc_balance * 100
        cs.enforce(
            || "liquidity_percentage_constraint",
            |lc| lc + actual_liquidity_percentage,
            |lc| lc + total_value_var,
            |lc| lc + (F::from(100u64), usdc_balance_var),
        );

        // Enforce: actual_liquidity_percentage >= min_liquidity_percentage
        // This is equivalent to: actual_liquidity_percentage - min_liquidity_percentage >= 0
        let surplus = cs.alloc(
            || "liquidity_surplus",
            || {
                let actual = {
                    let usdc = self.usdc_balance.ok_or(SynthesisError::AssignmentMissing)?;
                    let total = self
                        .total_portfolio_value
                        .ok_or(SynthesisError::AssignmentMissing)?;
                    let total_inv: Option<F> = total.invert().into();
                    let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                    usdc * total_inv * F::from(100u64)
                };

                let min_liq = self
                    .min_liquidity_percentage
                    .map(F::from)
                    .ok_or(SynthesisError::AssignmentMissing)?;

                let surplus = actual - min_liq;
                Ok(surplus)
            },
        )?;

        // Enforce: surplus = actual_liquidity_percentage - min_liquidity_percentage
        cs.enforce(
            || "surplus_constraint",
            |lc| lc + CS::one(),
            |lc| lc + surplus,
            |lc| lc + actual_liquidity_percentage - min_liquidity_var,
        );

        // Enforce surplus >= 0 using range proof (bit decomposition)
        // This ensures actual_liquidity_percentage >= min_liquidity_percentage
        let surplus_val = match (self.usdc_balance, self.total_portfolio_value, self.min_liquidity_percentage) {
            (Some(usdc), Some(total), Some(min_liq)) => {
                let total_inv: Option<F> = total.invert().into();
                let total_inv = total_inv.ok_or(SynthesisError::DivisionByZero)?;
                let actual = usdc * total_inv * F::from(100u64);
                Some(actual - F::from(min_liq))
            }
            _ => None,
        };

        range_proof::decompose_allocated_value(
            cs.namespace(|| "range_proof_surplus"),
            surplus,
            surplus_val,
            32, // 32 bits is enough for percentage differences
            "surplus_bits",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_liquidity_reserve_sufficient() {
        // Portfolio with sufficient liquidity
        // Total: 100M, USDC: 15M (15% > 10% requirement)
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let usdc = Fp::from(15_000_000u64); // 15%
        let min_liquidity = 10u64; // 10% requirement

        let circuit = LiquidityReserveCircuit::new(min_liquidity, usdc, total);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
        println!("Num constraints: {}", cs.num_constraints());
    }

    #[test]
    fn test_liquidity_reserve_at_minimum() {
        // Portfolio with exactly minimum liquidity
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let usdc = Fp::from(10_000_000u64); // Exactly 10%
        let min_liquidity = 10u64;

        let circuit = LiquidityReserveCircuit::new(min_liquidity, usdc, total);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_liquidity_reserve_insufficient() {
        // Portfolio with insufficient liquidity
        // Total: 100M, USDC: 5M (5% < 10% requirement)
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let usdc = Fp::from(5_000_000u64); // Only 5%
        let min_liquidity = 10u64;

        let circuit = LiquidityReserveCircuit::new(min_liquidity, usdc, total);

        let result = circuit.synthesize(&mut cs);
        if result.is_ok() {
            assert!(
                !cs.is_satisfied(),
                "Circuit should not be satisfied with insufficient liquidity"
            );
        }
    }

    #[test]
    fn test_liquidity_reserve_high_percentage() {
        // Portfolio with very high liquidity
        let mut cs = TestConstraintSystem::<Fp>::new();

        let total = Fp::from(100_000_000u64);
        let usdc = Fp::from(50_000_000u64); // 50% liquidity
        let min_liquidity = 10u64;

        let circuit = LiquidityReserveCircuit::new(min_liquidity, usdc, total);

        circuit.synthesize(&mut cs).unwrap();
        assert!(cs.is_satisfied());
    }
}
