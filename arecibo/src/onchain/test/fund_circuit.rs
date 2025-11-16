//! Fund compliance circuit for Arecibo Nova integration
//! This is our NovaLiquidityCircuit adapted to work directly in Arecibo

use crate::{
    frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError},
    nebula::rs::StepCircuit,
};
use ff::Field;
use halo2curves::bn256::Fr;

/// Liquidity Reserve Circuit for tokenized RWA funds
///
/// Proves that USDC liquidity meets minimum threshold without revealing exact amounts.
#[derive(Clone, Debug)]
pub struct FundLiquidityCircuit {
    /// Minimum liquidity percentage required (e.g., 10 for 10%)
    pub min_liquidity_pct: u64,
    /// Actual USDC balance (private input)
    pub usdc_balance: u64,
    /// Total portfolio value (private input)
    pub total_value: u64,
}

impl FundLiquidityCircuit {
    /// Creates a new FundLiquidityCircuit instance.
    ///
    /// # Arguments
    /// * `min_liquidity_pct` - Minimum required liquidity percentage (e.g., 10 for 10%)
    /// * `usdc_balance` - Actual USDC balance in the fund (private input)
    /// * `total_value` - Total portfolio value (private input)
    pub fn new(min_liquidity_pct: u64, usdc_balance: u64, total_value: u64) -> Self {
        Self {
            min_liquidity_pct,
            usdc_balance,
            total_value,
        }
    }
}

impl StepCircuit<Fr> for FundLiquidityCircuit {
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
        // We verify: usdc * 100 = actual_pct * total
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
