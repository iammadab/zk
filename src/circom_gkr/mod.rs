use ark_ff::PrimeField;
use std::ffi::c_long;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
enum EquationDirection {
    Left,
    Right,
}

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
/// Term(1, -1)
///     will get the value stored in s2 (e.g. 5) and mul that by -1
///     = -5
/// This is the building block for representing R1Cs constraints
struct Term<F: PrimeField>(usize, F);

impl<F: PrimeField> Term<F> {
    /// Create a new term with negative value
    /// this is useful for when we rearrange terms in an equation
    /// and terms have to move over the equal (=) sign
    fn negate(&self) -> Self {
        Term(self.0, self.1 * F::one().neg())
    }
}

#[derive(Debug, PartialEq)]
/// Represents a single R1CS constraint
/// As . Bs = Cs
/// where s contains the witness and constants
struct Constraint<F: PrimeField> {
    a: Vec<Term<F>>,
    b: Vec<Term<F>>,
    c: Vec<Term<F>>,
    operation: Operation,
}

impl<F: PrimeField> Constraint<F> {
    /// Create a new constraint, does automatic operation detection
    /// if both a and b are present then operation = mul, else operation = add
    /// see: new_with_operation to remove automatic operation detection
    fn new(a: Vec<Term<F>>, b: Vec<Term<F>>, c: Vec<Term<F>>) -> Self {
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

    /// Create new constraint with automatic operation detection
    fn new_with_operation(
        a: Vec<Term<F>>,
        b: Vec<Term<F>>,
        c: Vec<Term<F>>,
        operation: Operation,
    ) -> Self {
        Self { a, b, c, operation }
    }

    // TODO: should this be mut
    // TODO: add documentation
    fn simplify(&mut self) -> Vec<ReducedConstraint<F>> {
        // we first need to know if is simplifiable or not
        // if it is not, we tranform this to a reduced constraint
        if self.can_simplify() {
            self.rearrange_terms();
            todo!()
        } else {
            // vec![*self.try_into().unwrap()]
            todo!()
        }
    }

    /// Determines if a constraint needs simplification before converting to a ReducedConstraint
    fn can_simplify(&self) -> bool {
        let has_more_than_one_term_in_a_slot =
            self.a.len() > 1 || self.b.len() > 2 || self.c.len() > 3;
        if self.terms_count() > 3 || has_more_than_one_term_in_a_slot {
            true
        } else {
            false
        }
    }

    /// Simplifies constraint by moving extra terms to empty slots (as many as possible)
    fn rearrange_terms(&mut self) {
        while self.should_rearrange_terms() {
            let (term, term_location) = self.get_movable_term();
            let (empty_slot, slot_location) = self.get_empty_slot();

            // we can safely unwrap
            // the should_rearrange_terms check already verifies they exist
            move_term_to_slot(
                term.unwrap(),
                empty_slot.unwrap(),
                slot_location == term_location,
            );
        }
    }

    // TODO: consider giving this type Slot
    /// Returns a mutable reference to the empty slot in the constraint equation
    /// e.g. if A = [s1] B = [] and C = [s3] returns a mutable reference to B
    /// also returns the size of the equation the slot belongs
    /// returns None if no empty slot
    fn get_empty_slot(&mut self) -> (Option<&mut Vec<Term<F>>>, EquationDirection) {
        if self.a.is_empty() {
            (Some(&mut self.a), EquationDirection::Left)
        } else if self.b.is_empty() {
            (Some(&mut self.b), EquationDirection::Left)
        } else if self.c.is_empty() {
            (Some(&mut self.c), EquationDirection::Right)
        } else {
            (None, EquationDirection::Right)
        }
    }

    /// Searches for slots that have more than 1 term, removes a term from there and returns it
    /// also returns the equation side the term was taken from
    fn get_movable_term(&mut self) -> (Option<Term<F>>, EquationDirection) {
        if self.a.len() > 1 {
            (self.a.pop(), EquationDirection::Left)
        } else if self.b.len() > 1 {
            (self.b.pop(), EquationDirection::Left)
        } else if self.c.len() > 1 {
            (self.c.pop(), EquationDirection::Right)
        } else {
            (None, EquationDirection::Right)
        }
    }

    /// Determines if there is an empty slot to move double terms to
    /// e.g. A = [s1. s2], B = [s3], C = [] and operation = Add
    /// above is s1 + s2 + s3 = 0
    /// rearrangement will move either s1 or s2 to c
    /// A = [s1], B = [s3], C = [-s2] resulting in s1 + s3 = -s2
    fn should_rearrange_terms(&self) -> bool {
        // when the operation is multiplication, terms cannot be rearranged
        if self.operation == Operation::Mul {
            return false;
        }

        // TODO: write tests to solidify this constraints

        // no need to rearrange if all slots already have a maximum of one term
        if self.a.len() <= 1 && self.b.len() <= 1 && self.c.len() <= 1 {
            return false;
        }

        // otherwise, we need to ensure there is an empty slot for a term to move to
        self.a.is_empty() || self.b.is_empty() || self.c.is_empty()
    }

    /// Total number of terms in the constraint
    fn terms_count(&self) -> usize {
        self.a.len() + self.b.len() + self.c.len()
    }
}

/// Move a term to a given slot
/// if the slot is not in the same side of the equation as the term
/// then negate the term (this simulates moving a term over the equal (=) sign)
fn move_term_to_slot<F: PrimeField>(
    mut term: Term<F>,
    slot: &mut Vec<Term<F>>,
    same_direction: bool,
) {
    // if the equation direction changes, then we are moving over the equal sign
    // hence we need to negate the term
    if !same_direction {
        term = term.negate();
    }

    slot.push(term);
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
    a: Option<Term<F>>,
    b: Option<Term<F>>,
    c: Option<Term<F>>,
    operation: Operation,
}

#[cfg(test)]
mod tests {
    use crate::circom_gkr::{
        move_term_to_slot, Constraint, EquationDirection, Operation, ReducedConstraint, Term,
    };
    use ark_bls12_381::Fr;

    #[test]
    fn test_term_negation() {
        let p1 = Term(0, Fr::from(2));
        assert_eq!(p1.negate(), Term(0, Fr::from(-2)));

        let p2 = Term(0, Fr::from(-2));
        assert_eq!(p2.negate(), Term(0, Fr::from(2)));
    }

    #[test]
    fn test_constraint_correct_operation_type() {
        // s1 * s2 = s3
        // expected operation type = mul
        let constraint = Constraint::new(
            vec![Term(0, Fr::from(1))],
            vec![Term(1, Fr::from(1))],
            vec![Term(2, Fr::from(1))],
        );
        assert_eq!(constraint.operation, Operation::Mul);

        // s1 + s2 = s3
        // expected operation type = add
        let constraint = Constraint::new(
            vec![Term(0, Fr::from(1)), Term(1, Fr::from(1))],
            vec![],
            vec![Term(2, Fr::from(1))],
        );
        assert_eq!(constraint.operation, Operation::Add);

        // s1 = -s2
        let constraint = Constraint::new(
            vec![],
            vec![Term(0, Fr::from(1))],
            vec![Term(1, Fr::from(-1))],
        );
        assert_eq!(constraint.operation, Operation::Add);

        // s1 * s2 = 0
        let constraint = Constraint::new(
            vec![Term(0, Fr::from(1))],
            vec![Term(1, Fr::from(1))],
            vec![],
        );
        assert_eq!(constraint.operation, Operation::Mul);
    }

    #[test]
    fn test_reduced_constraint_from_constraint() {
        let constraint = Constraint::new(
            vec![Term(0, Fr::from(1))],
            vec![Term(0, Fr::from(1))],
            vec![],
        );
        assert_eq!(constraint.can_simplify(), false);
        let reduced_constraint: ReducedConstraint<Fr> = (&constraint).try_into().unwrap();
        assert_eq!(
            reduced_constraint,
            ReducedConstraint {
                a: Some(Term(0, Fr::from(1))),
                b: Some(Term(0, Fr::from(1))),
                c: None,
                operation: Operation::Mul
            }
        );
    }

    #[test]
    fn test_can_rearrange() {
        // already simplified, but can be rearranged
        let constraint = Constraint::new(vec![Term(0, Fr::from(1))], vec![], vec![]);
        assert_eq!(constraint.can_simplify(), false);
        assert_eq!(constraint.should_rearrange_terms(), false);

        // can simplify and can be rearranged
        let constraint = Constraint::new(
            vec![
                Term(0, Fr::from(1)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(1)),
                Term(3, Fr::from(1)),
            ],
            vec![],
            vec![],
        );
        assert_eq!(constraint.can_simplify(), true);
        assert_eq!(constraint.should_rearrange_terms(), true);

        // can simply but cannot rearrange
        let constraint = Constraint::new(
            vec![Term(0, Fr::from(1)), Term(2, Fr::from(1))],
            vec![Term(1, Fr::from(1))],
            vec![Term(3, Fr::from(1))],
        );
        assert_eq!(constraint.can_simplify(), true);
        assert_eq!(constraint.should_rearrange_terms(), false);

        // cannot rearrange multiplication constraints
        let constraint = Constraint::new(
            vec![
                Term(0, Fr::from(1)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(1)),
                Term(3, Fr::from(1)),
            ],
            vec![Term(0, Fr::from(1))],
            vec![],
        );
        assert_eq!(constraint.can_simplify(), true);
        assert_eq!(constraint.should_rearrange_terms(), false);
    }

    #[test]
    fn test_get_empty_slot() {
        let mut constraint = Constraint::new(vec![Term(0, Fr::from(1))], vec![], vec![]);
        let (empty_slot, slot_location) = constraint.get_empty_slot();
        assert_eq!(empty_slot.is_some(), true);
        assert_eq!(empty_slot.unwrap().len(), 0);
        assert_eq!(slot_location, EquationDirection::Left);

        let mut constraint = Constraint::new(
            vec![Term(0, Fr::from(1))],
            vec![Term(0, Fr::from(1))],
            vec![],
        );
        let (empty_slot, slot_location) = constraint.get_empty_slot();
        assert_eq!(empty_slot.is_some(), true);
        assert_eq!(empty_slot.unwrap().len(), 0);
        assert_eq!(slot_location, EquationDirection::Right);

        let mut constraint = Constraint::new(
            vec![Term(0, Fr::from(1))],
            vec![Term(0, Fr::from(1))],
            vec![Term(0, Fr::from(2))],
        );
        let (empty_slot, slot_location) = constraint.get_empty_slot();
        assert_eq!(empty_slot.is_none(), true);
    }

    #[test]
    fn test_get_movable_term() {
        // should be able to move 4 terms from this (without replacement)
        // 3 from A and 1 from C
        let mut constraint = Constraint::new(
            vec![
                Term(0, Fr::from(1)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(1)),
                Term(3, Fr::from(1)),
            ],
            vec![],
            vec![Term(2, Fr::from(1)), Term(3, Fr::from(1))],
        );

        let (movable_item, slot_location) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), true);
        assert_eq!(movable_item.unwrap(), Term(3, Fr::from(1)));
        assert_eq!(slot_location, EquationDirection::Left);

        let (movable_item, slot_location) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), true);
        assert_eq!(movable_item.unwrap(), Term(2, Fr::from(1)));
        assert_eq!(slot_location, EquationDirection::Left);

        let (movable_item, slot_location) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), true);
        assert_eq!(movable_item.unwrap(), Term(1, Fr::from(1)));
        assert_eq!(slot_location, EquationDirection::Left);

        let (movable_item, slot_location) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), true);
        assert_eq!(movable_item.unwrap(), Term(3, Fr::from(1)));
        assert_eq!(slot_location, EquationDirection::Right);

        let (movable_item, slot_location) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), false);
    }

    #[test]
    fn test_move_to_slot() {
        let mut slot = vec![];
        let term = Term(0, Fr::from(1));
        move_term_to_slot(term, &mut slot, true);
        assert_eq!(slot.len(), 1);
        assert_eq!(slot[0], Term(0, Fr::from(1)));

        let mut slot = vec![];
        let term = Term(0, Fr::from(2));
        move_term_to_slot(term, &mut slot, false);
        assert_eq!(slot.len(), 1);
        assert_eq!(slot[0], Term(0, Fr::from(-2)));
    }

    #[test]
    fn test_rearrange_constraint() {
        let mut constraint = Constraint::new(
            vec![
                Term(0, Fr::from(1)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(-5)),
                Term(3, Fr::from(1)),
            ],
            vec![],
            vec![],
        );

        constraint.rearrange_terms();

        assert_eq!(
            constraint,
            Constraint::new_with_operation(
                vec![Term(0, Fr::from(1)), Term(1, Fr::from(1))],
                vec![Term(3, Fr::from(1))],
                vec![Term(2, Fr::from(5))],
                Operation::Add
            )
        )
    }
}
