use ark_ff::PrimeField;
use std::ffi::c_long;

#[derive(Clone, Debug, PartialEq)]
/// Represents the constraint operation
enum Operation {
    Add,
    Mul,
}

#[derive(Clone, Debug, PartialEq)]
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

    // TODO: add documentation
    fn simplify(&self) -> Vec<ReducedConstraint<F>> {
        // we first need to know if is simplifiable or not
        // if it is not, we tranform this to a reduced constraint
        if self.can_simplify() {
            todo!()
        } else {
            vec![self.try_into().unwrap()]
        }
    }

    /// Determines if a constraint needs simplification before converting to a ReducedConstraint
    fn can_simplify(&self) -> bool {
        let has_more_than_one_term_in_a_slot = self.a.len() > 1 || self.b.len() > 2 || self.c.len() > 3;
        if self.terms_count() > 3 || has_more_than_one_term_in_a_slot {
            true
        } else {
            false
        }
    }

    /// Total number of terms in the constraint
    fn terms_count(&self) -> usize {
        self.a.len() + self.b.len() + self.c.len()
    }
}

// TODO: do I really need this?
impl<F: PrimeField> TryFrom<&Constraint<F>> for ReducedConstraint<F> {
    type Error = &'static str;

    fn try_from(value: &Constraint<F>) -> Result<Self, Self::Error> {
        if value.a.len() > 1 || value.b.len() > 1 || value.c.len() > 1 {
            return Err("can only convert constraints that have at most 1 value for A, B and C");
        }

        Ok(Self {
            a: value.a.get(0).cloned(),
            b: value.b.get(0).cloned(),
            c: value.c.get(0).cloned(),
            operation: value.operation.clone(),
        })
    }
}

#[derive(Debug, PartialEq)]
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
    use crate::circom_gkr::{Constraint, Operation, ProductArg, ReducedConstraint};
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

    #[test]
    fn test_reduced_constraint_from_constraint() {
        let constraint = Constraint::new(
            vec![ProductArg(0, Fr::from(1))],
            vec![ProductArg(0, Fr::from(1))],
            vec![],
        );
        let reduced_constraint: ReducedConstraint<Fr> = (&constraint).try_into().unwrap();
        assert_eq!(
            reduced_constraint,
            ReducedConstraint {
                a: Some(ProductArg(0, Fr::from(1))),
                b: Some(ProductArg(0, Fr::from(1))),
                c: None,
                operation: Operation::Mul
            }
        );
    }
}
