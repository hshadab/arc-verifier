//! Utility functions for circuit operations

use bellpepper_core::{ConstraintSystem, SynthesisError, Variable};
use ff::PrimeField;

/// Allocate a variable representing a value that might not be known yet
pub fn alloc_option<F, CS>(
    mut cs: CS,
    value: Option<F>,
    label: &str,
) -> Result<Variable, SynthesisError>
where
    F: PrimeField,
    CS: ConstraintSystem<F>,
{
    let var = cs.alloc(
        || label,
        || value.ok_or(SynthesisError::AssignmentMissing),
    )?;
    Ok(var)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_alloc_option() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let value = Some(Fp::from(42u64));
        let var = alloc_option(cs.namespace(|| "test"), value, "test_var").unwrap();
        assert!(cs.is_satisfied());
    }
}
