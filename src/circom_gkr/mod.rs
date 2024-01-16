use ark_ff::PrimeField;
use std::ffi::c_long;

#[derive(Debug, PartialEq)]
/// Represents the constraint operation
enum Operation {
    Add,
    Mul,
}

#[derive(Clone)]
/// Contains a pointer to a variable and field element to mul the
/// variable's value with.
/// e.g. let [s1, s2, s3] be the set of variables
/// ProductArg(1, -1)
///     will get the value stored in s2 (e.g. 5) and mul that by -1
///     = -5
/// This is the building block for representing R1Cs constraints
struct ProductArg<F: PrimeField>(usize, F);

/// Represents a single R1CS constraint
/// As . Bs = Cs
/// where s contains the witness and constants
struct Constraint<F: PrimeField> {
    a: Vec<ProductArg<F>>,
    b: Vec<ProductArg<F>>,
    c: Vec<ProductArg<F>>,
    operation: Operation,
}

impl<F: PrimeField> Constraint<F> {
    fn new(a: Vec<ProductArg<F>>, b: Vec<ProductArg<F>>, c: Vec<ProductArg<F>>) -> Self {
        // R1CS is of the form <As> . <Bs> = <Cs>
        // where As, Bs and Cs are inner products
        // if either As or Bs is not present then the multiplication
        // operand is never invoked.
        let operation = if a.is_empty() || b.is_empty() {
            Operation::Add
        } else {
            Operation::Mul
        };

        Self { a, b, c, operation }
    }
}

// TODO: do you really need this?
impl<F: PrimeField> TryFrom<Constraint<F>> for ReducedConstraint<F> {
    type Error = &'static str;

    fn try_from(value: Constraint<F>) -> Result<Self, Self::Error> {
        if value.a.len() > 1 || value.b.len() > 1 || value.c.len() > 1 {
            return Err("can only convert constraints that have at most 1 value for A, B and C");
        }

        Ok(Self {
            a: value.a.get(0).cloned(),
            b: value.b.get(0).cloned(),
            c: value.c.get(0).cloned(),
            operation: value.operation,
        })
    }
}

/// Simplified constraint that contains at most 3 operations
/// and at most 1 operation type
struct ReducedConstraint<F: PrimeField> {
    a: Option<ProductArg<F>>,
    b: Option<ProductArg<F>>,
    c: Option<ProductArg<F>>,
    operation: Operation,
}

#[cfg(test)]
mod tests {
    use crate::circom_gkr::{Constraint, Operation, ProductArg};
    use ark_bls12_381::Fr;

    #[test]
    fn test_constraint_correct_operation_type() {
        // s1 * s2 = s3
        // expected operation type = mul
        let constraint = Constraint::new(
            vec![ProductArg(0, Fr::from(1))],
            vec![ProductArg(1, Fr::from(1))],
            vec![ProductArg(2, Fr::from(1))],
        );
        assert_eq!(constraint.operation, Operation::Mul);

        // s1 + s2 = s3
        // expected operation type = add
        let constraint = Constraint::new(
            vec![ProductArg(0, Fr::from(1)), ProductArg(1, Fr::from(1))],
            vec![],
            vec![ProductArg(2, Fr::from(1))],
        );
        assert_eq!(constraint.operation, Operation::Add);

        // s1 = -s2
        let constraint = Constraint::new(
            vec![],
            vec![ProductArg(0, Fr::from(1))],
            vec![ProductArg(1, Fr::from(-1))],
        );
        assert_eq!(constraint.operation, Operation::Add);

        // s1 * s2 = 0
        let constraint = Constraint::new(
            vec![ProductArg(0, Fr::from(1))],
            vec![ProductArg(1, Fr::from(1))],
            vec![],
        );
        assert_eq!(constraint.operation, Operation::Mul);
    }

    // #[test]
    // fn test_reduced_constraint_from_constraint() {
    //     let constraint = Constraint::new(
    //         vec![ProductArg(0, Fr::from(1))],
    //         vec![ProductArg(0, Fr::from(1))],
    //         vec![ProductArg(0, Fr::from(1))],
    //     );
    // }
}
