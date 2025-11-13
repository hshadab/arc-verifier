//! Range Proof Utilities
//!
//! Provides functionality to prove that a value is non-negative by decomposing
//! it into bits and enforcing each bit is boolean.
//!
//! This is critical for enforcing inequalities in zero-knowledge:
//! - To prove a ≤ b, we prove (b - a) ≥ 0
//! - To prove (b - a) ≥ 0, we decompose (b - a) into bits
//! - If the value were negative, it would wrap around to a large number
//! - The bit decomposition will fail if the value is out of range

use bellpepper_core::{ConstraintSystem, LinearCombination, SynthesisError, Variable};
use ff::PrimeField;

/// Number of bits to use for range proofs
/// 64 bits supports values up to ~18 * 10^18 (way more than needed for percentages)
pub const RANGE_PROOF_BITS: usize = 64;

/// Allocate a boolean variable
pub fn alloc_boolean<F, CS>(
    mut cs: CS,
    value: Option<bool>,
    label: &str,
) -> Result<Variable, SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    let var = cs.alloc(
        || label,
        || {
            value
                .map(|b| if b { F::ONE } else { F::ZERO })
                .ok_or(SynthesisError::AssignmentMissing)
        },
    )?;

    // Enforce boolean constraint: bit * (1 - bit) = 0
    cs.enforce(
        || format!("{}_boolean", label),
        |lc| lc + var,
        |lc| lc + CS::one() - var,
        |lc| lc,
    );

    Ok(var)
}

/// Decompose a value into bits and return the bit variables
/// This proves the value is in the range [0, 2^num_bits)
pub fn decompose_into_bits<F, CS>(
    mut cs: CS,
    value: Option<F>,
    num_bits: usize,
    label: &str,
) -> Result<Vec<Variable>, SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    let value_var = cs.alloc(
        || format!("{}_value", label),
        || value.ok_or(SynthesisError::AssignmentMissing),
    )?;

    decompose_allocated_value(cs, value_var, value, num_bits, label)
}

/// Decompose an already-allocated value into bits
pub fn decompose_allocated_value<F, CS>(
    mut cs: CS,
    value_var: Variable,
    value: Option<F>,
    num_bits: usize,
    label: &str,
) -> Result<Vec<Variable>, SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    let mut bits = Vec::with_capacity(num_bits);
    let mut recomposition = LinearCombination::zero();
    let mut coeff = F::ONE;

    // Extract bits from the value
    let value_bits: Option<Vec<bool>> = value.map(|v| {
        let repr = v.to_repr();
        let bytes = repr.as_ref();
        (0..num_bits)
            .map(|i| {
                let byte_index = i / 8;
                let bit_index = i % 8;
                if byte_index < bytes.len() {
                    (bytes[byte_index] >> bit_index) & 1 == 1
                } else {
                    false
                }
            })
            .collect()
    });

    // Allocate and constrain each bit
    for i in 0..num_bits {
        let bit_value = value_bits.as_ref().and_then(|bits| bits.get(i).copied());

        let bit_var = alloc_boolean(
            cs.namespace(|| format!("{}_{}", label, i)),
            bit_value,
            &format!("bit_{}", i),
        )?;

        bits.push(bit_var);

        // Add bit * 2^i to recomposition
        recomposition = recomposition + (coeff, bit_var);

        // Update coefficient for next bit
        if i < num_bits - 1 {
            coeff = coeff.double();
        }
    }

    // Enforce: sum(bit_i * 2^i) = value
    cs.enforce(
        || format!("{}_recomposition", label),
        |lc| lc + CS::one(),
        |lc| lc + value_var,
        |_| recomposition,
    );

    Ok(bits)
}

/// Prove that a ≤ b by proving (b - a) is in valid range
pub fn enforce_less_than_or_equal<F, CS>(
    mut cs: CS,
    a: Variable,
    b: Variable,
    a_val: Option<F>,
    b_val: Option<F>,
    num_bits: usize,
) -> Result<(), SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    // Compute difference: b - a
    let diff_val = match (a_val, b_val) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };

    let diff = cs.alloc(
        || "difference",
        || diff_val.ok_or(SynthesisError::AssignmentMissing),
    )?;

    // Enforce: diff = b - a
    cs.enforce(
        || "diff_equals_b_minus_a",
        |lc| lc + CS::one(),
        |lc| lc + diff,
        |lc| lc + b - a,
    );

    // Decompose diff into bits (proves it's non-negative and in range)
    decompose_allocated_value(
        cs.namespace(|| "range_proof"),
        diff,
        diff_val,
        num_bits,
        "diff_bits",
    )?;

    Ok(())
}

/// Prove that a ≥ b by proving (a - b) is in valid range
pub fn enforce_greater_than_or_equal<F, CS>(
    mut cs: CS,
    a: Variable,
    b: Variable,
    a_val: Option<F>,
    b_val: Option<F>,
    num_bits: usize,
) -> Result<(), SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    // Just swap the arguments
    enforce_less_than_or_equal(cs, b, a, b_val, a_val, num_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_alloc_boolean_true() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let bit = alloc_boolean(cs.namespace(|| "test"), Some(true), "bit").unwrap();
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_alloc_boolean_false() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let bit = alloc_boolean(cs.namespace(|| "test"), Some(false), "bit").unwrap();
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_decompose_into_bits_small() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let value = Fp::from(42u64);

        let bits = decompose_into_bits(
            cs.namespace(|| "decompose"),
            Some(value),
            8,
            "value",
        ).unwrap();

        assert_eq!(bits.len(), 8);
        assert!(cs.is_satisfied());
        println!("Decompose 42 into 8 bits: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_decompose_into_bits_large() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let value = Fp::from(1000000u64);

        let bits = decompose_into_bits(
            cs.namespace(|| "decompose"),
            Some(value),
            32,
            "value",
        ).unwrap();

        assert_eq!(bits.len(), 32);
        assert!(cs.is_satisfied());
        println!("Decompose 1M into 32 bits: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_enforce_less_than_or_equal_valid() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let a = Fp::from(10u64);
        let b = Fp::from(20u64);

        let a_var = cs.alloc(|| "a", || Ok(a)).unwrap();
        let b_var = cs.alloc(|| "b", || Ok(b)).unwrap();

        enforce_less_than_or_equal(
            cs.namespace(|| "a_le_b"),
            a_var,
            b_var,
            Some(a),
            Some(b),
            16,
        ).unwrap();

        assert!(cs.is_satisfied());
        println!("Range proof (10 ≤ 20): {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_enforce_less_than_or_equal_equal() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let a = Fp::from(15u64);
        let b = Fp::from(15u64);

        let a_var = cs.alloc(|| "a", || Ok(a)).unwrap();
        let b_var = cs.alloc(|| "b", || Ok(b)).unwrap();

        enforce_less_than_or_equal(
            cs.namespace(|| "a_le_b"),
            a_var,
            b_var,
            Some(a),
            Some(b),
            16,
        ).unwrap();

        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_enforce_less_than_or_equal_invalid() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        // This should fail: 20 > 10
        let a = Fp::from(20u64);
        let b = Fp::from(10u64);

        let a_var = cs.alloc(|| "a", || Ok(a)).unwrap();
        let b_var = cs.alloc(|| "b", || Ok(b)).unwrap();

        let result = enforce_less_than_or_equal(
            cs.namespace(|| "a_le_b"),
            a_var,
            b_var,
            Some(a),
            Some(b),
            16,
        );

        // The circuit should synthesize but not be satisfied
        // (the difference will be negative, wrapping to large number, bits won't match)
        if result.is_ok() {
            assert!(
                !cs.is_satisfied(),
                "Circuit should not be satisfied when a > b"
            );
        }
    }

    #[test]
    fn test_enforce_greater_than_or_equal_valid() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let a = Fp::from(20u64);
        let b = Fp::from(10u64);

        let a_var = cs.alloc(|| "a", || Ok(a)).unwrap();
        let b_var = cs.alloc(|| "b", || Ok(b)).unwrap();

        enforce_greater_than_or_equal(
            cs.namespace(|| "a_ge_b"),
            a_var,
            b_var,
            Some(a),
            Some(b),
            16,
        ).unwrap();

        assert!(cs.is_satisfied());
    }
}
