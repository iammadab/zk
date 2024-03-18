use crate::program::SymbolTable;
use ark_ff::PrimeField;

#[derive(Debug, PartialEq)]
/// Simplified constraint that contains at most 3 operations
/// and at most 1 operation type
/// Constraints are compiled into a set of ReducedConstraints.
pub struct ReducedConstraint<F: PrimeField> {
    pub(crate) a: Option<Term<F>>,
    pub(crate) b: Option<Term<F>>,
    pub(crate) c: Option<Term<F>>,
    pub(crate) operation: Operation,
}

#[derive(Clone, Debug, PartialEq)]
/// Represents a single R1CS constraint
/// As . Bs = Cs
/// where s contains the witness and constants
pub struct Constraint<F: PrimeField> {
    pub a: Slot<F>,
    pub b: Slot<F>,
    pub c: Slot<F>,
    operation: Operation,
}

impl<F: PrimeField> Constraint<F> {
    /// Create a new constraint, does automatic operation detection
    /// if both a and b are present then operation = mul, else operation = add
    /// see: new_with_operation to remove automatic operation detection
    pub fn new(a: Vec<Term<F>>, b: Vec<Term<F>>, c: Vec<Term<F>>) -> Self {
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

    /// Reduce a constraint into one or more reduced constraints.
    /// First it tries to move extra terms to empty slots, then it attempts merging terms into
    /// new constraints
    pub fn reduce(mut self, symbol_table: &mut SymbolTable<F>) -> Vec<ReducedConstraint<F>> {
        if self.can_simplify() {
            self.rearrange_terms();
            self.reduce_into_multiple_constraints(symbol_table)
        } else {
            vec![(&self).try_into().unwrap()]
        }
    }

    /// Converts a constraint into one or more reduced constraint, such that all constraints have at
    /// most 3 terms and one operation.
    /// See in code comment for more details.
    fn reduce_into_multiple_constraints(
        mut self,
        symbol_table: &mut SymbolTable<F>,
    ) -> Vec<ReducedConstraint<F>> {
        let mut reduced_constraints = vec![];
        while self.can_simplify() {
            // for simplification, we get two terms that can be merged
            // create a new variable to represent their output
            // replace them in the original constraint with the output term
            // and create a new reduced constraint for them
            // e.g. s1 + s2 + s3 = s4
            // we can merge s1 and s2 to become s5
            // s5 = s1 + s2 <--- new reduced constraint
            // s5 + s3 = s4 <--- updated original constraint
            let (mergeable_terms, slot) = self
                .get_mergeable_terms()
                .expect("can_simplify check makes it safe to unwrap");

            let variable_index =
                symbol_table.get_variable_index(mergeable_terms[1], mergeable_terms[0]);

            let output_term = Term(variable_index, F::one());

            reduced_constraints.push(ReducedConstraint {
                a: Some(mergeable_terms[1]),
                b: Some(mergeable_terms[0]),
                c: Some(output_term),
                operation: Operation::Add,
            });

            slot.push(output_term);
        }
        reduced_constraints.push((&self).try_into().unwrap());
        reduced_constraints
    }

    /// Determines if a constraint needs simplification before converting to a ReducedConstraint
    fn can_simplify(&self) -> bool {
        let has_more_than_one_term_in_a_slot =
            self.a.len() > 1 || self.b.len() > 1 || self.c.len() > 1;
        self.terms_count() > 3 || has_more_than_one_term_in_a_slot
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

    /// Returns a mutable reference to the empty slot in the constraint equation
    /// e.g. if A = [s1] B = [] and C = [s3] returns a mutable reference to B
    /// also returns the size of the equation the slot belongs
    /// returns None if no empty slot
    fn get_empty_slot(&mut self) -> (Option<&mut Slot<F>>, EquationDirection) {
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

    /// Searches for slots that have more than 1 term, removes two terms from that slot.
    /// returns them with a reference to the slot
    fn get_mergeable_terms(&mut self) -> Option<([Term<F>; 2], &mut Slot<F>)> {
        // safe to unwrap when popping, confirmed more than 1 item exists
        if self.a.len() > 1 {
            Some(([self.a.pop().unwrap(), self.a.pop().unwrap()], &mut self.a))
        } else if self.b.len() > 1 {
            Some(([self.b.pop().unwrap(), self.b.pop().unwrap()], &mut self.b))
        } else if self.c.len() > 1 {
            Some(([self.c.pop().unwrap(), self.c.pop().unwrap()], &mut self.c))
        } else {
            None
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

    /// Returns the maximum index assigned to a variable
    pub fn max_index(&self) -> usize {
        let max_index_a = self.a.iter().map(|term| term.0).max().unwrap_or(0);
        let max_index_b = self.b.iter().map(|term| term.0).max().unwrap_or(0);
        let max_index_c = self.c.iter().map(|term| term.0).max().unwrap_or(0);
        max_index_a.max(max_index_b).max(max_index_c)
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

impl<F: PrimeField> TryFrom<&Constraint<F>> for ReducedConstraint<F> {
    type Error = &'static str;

    fn try_from(value: &Constraint<F>) -> Result<Self, Self::Error> {
        if value.a.len() > 1 || value.b.len() > 1 || value.c.len() > 1 {
            return Err("can only convert constraints that have at most 1 value for A, B and C");
        }

        Ok(Self {
            a: value.a.first().cloned(),
            b: value.b.first().cloned(),
            c: value.c.first().cloned(),
            operation: value.operation.clone(),
        })
    }
}

/// Represents either As, Bs or Cs in the constraint
pub type Slot<F> = Vec<Term<F>>;

/// Signifies if a value is to the left or right of the equal sign
/// As . Bs (left) = Cs (right)
#[derive(Debug, PartialEq)]
enum EquationDirection {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
/// Represents the constraint operation
pub enum Operation {
    Add,
    Mul,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
/// Contains a pointer to a variable and field element to mul the
/// variable's value with.
/// e.g. let [s1, s2, s3] be the set of variables
/// Term(1, -1)
///     will get the value stored in s2 (e.g. 5) and mul that by -1
///     = -5
/// This is the building block for representing R1Cs constraints
pub struct Term<F: PrimeField>(pub usize, pub F);

impl<F: PrimeField> Term<F> {
    /// Create a new term with negative value
    /// this is useful for when we rearrange terms in an equation
    /// and terms have to move over the equal (=) sign
    fn negate(&self) -> Self {
        Term(self.0, self.1 * F::one().neg())
    }
}

impl<F: PrimeField> From<Term<F>> for (usize, F) {
    fn from(value: Term<F>) -> Self {
        (value.0, value.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::constraint::{
        move_term_to_slot, Constraint, EquationDirection, Operation, ReducedConstraint, Term,
    };
    use crate::program::SymbolTable;
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
        let (empty_slot, _) = constraint.get_empty_slot();
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

        let (movable_item, _) = constraint.get_movable_term();
        assert_eq!(movable_item.is_some(), false);
    }

    #[test]
    fn test_get_mergeable_items() {
        // should be able to extract mergeable items from the first slot twice (without replacement)
        // then once from the last slot
        // expecting 3 extractions before None
        let mut constraint = Constraint::new(
            vec![
                Term(0, Fr::from(2)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(1)),
                Term(3, Fr::from(1)),
            ],
            vec![],
            vec![Term(2, Fr::from(1)), Term(3, Fr::from(1))],
        );

        let (extra_terms, slot) = constraint.get_mergeable_terms().unwrap();
        assert_eq!(extra_terms[0], Term(3, Fr::from(1)));
        assert_eq!(extra_terms[1], Term(2, Fr::from(1)));
        assert_eq!(slot.len(), 2);

        let (extra_terms, slot) = constraint.get_mergeable_terms().unwrap();
        assert_eq!(extra_terms[0], Term(1, Fr::from(1)));
        assert_eq!(extra_terms[1], Term(0, Fr::from(2)));
        assert_eq!(slot.len(), 0);

        let (extra_terms, slot) = constraint.get_mergeable_terms().unwrap();
        assert_eq!(extra_terms[0], Term(3, Fr::from(1)));
        assert_eq!(extra_terms[1], Term(2, Fr::from(1)));
        assert_eq!(slot.len(), 0);

        assert_eq!(constraint.get_mergeable_terms().is_none(), true);
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
            Constraint {
                a: vec![Term(0, Fr::from(1)), Term(1, Fr::from(1))],
                b: vec![Term(3, Fr::from(1))],
                c: vec![Term(2, Fr::from(5))],
                operation: Operation::Add
            }
        )
    }

    #[test]
    fn test_reduce_to_multiple_constraint() {
        // Equation
        // -s3 * (s1 + s2) = 5 - out + s1 + s2
        // Reduction -> s4 = s1 + s2
        // -s3 * s4 = 5 - out + s1 + s2
        // Reduction -> s4 = s1 + s2 (same as previous constraint TODO: duplicated constraint)
        // -s3 * s4 = 5 - out + s4
        // Reduction -> s5 = -out + s4
        // -s3 * s4 = 5 + s5
        // Reduction -> s6 = 5 + s5
        // -s3 * s4 = s6

        // Variables
        // constant - 0
        // s1 = 1
        // s2 = 2
        // s3 = 3
        // out = 4
        // s4 = 5
        // s5 = 6
        // s6 = 7

        let constraint = Constraint::new(
            // -s3
            vec![Term(3, Fr::from(-1))],
            // (s1 + s2)
            vec![Term(1, Fr::from(1)), Term(2, Fr::from(1))],
            // 5 - out + s1 + s2
            vec![
                Term(0, Fr::from(5)),
                Term(4, Fr::from(-1)),
                Term(1, Fr::from(1)),
                Term(2, Fr::from(1)),
            ],
        );
        assert_eq!(constraint.max_index(), 4);

        // last known variable before reduction is out
        let mut symbol_table = SymbolTable::<Fr>::new(4);
        let reduced_constraints = constraint.reduce(&mut symbol_table);

        // we should have 5 reduced constraints
        // 4 new constraints + the original constraint
        assert_eq!(reduced_constraints.len(), 5);

        // assert constraints
        // first reduction s4 = s1 + s2
        assert_eq!(
            reduced_constraints[0],
            ReducedConstraint {
                // s1
                a: Some(Term(1, Fr::from(1))),
                // s2
                b: Some(Term(2, Fr::from(1))),
                // s4
                c: Some(Term(5, Fr::from(1))),
                operation: Operation::Add
            }
        );
        // next reduction s4 = s1 + s2
        assert_eq!(
            reduced_constraints[1],
            ReducedConstraint {
                // s1
                a: Some(Term(1, Fr::from(1))),
                // s2
                b: Some(Term(2, Fr::from(1))),
                // s4
                c: Some(Term(5, Fr::from(1))),
                operation: Operation::Add
            }
        );
        // next reduction s5 = -out + s4
        assert_eq!(
            reduced_constraints[2],
            ReducedConstraint {
                // -out
                a: Some(Term(4, Fr::from(-1))),
                // s4
                b: Some(Term(5, Fr::from(1))),
                // s5
                c: Some(Term(6, Fr::from(1))),
                operation: Operation::Add
            }
        );
        // next reduction s6 = 5 + s5
        assert_eq!(
            reduced_constraints[3],
            ReducedConstraint {
                // 5
                a: Some(Term(0, Fr::from(5))),
                // s5
                b: Some(Term(6, Fr::from(1))),
                // s6
                c: Some(Term(7, Fr::from(1))),
                operation: Operation::Add
            }
        );
        // original statement reduced -s3 * s4 = s6
        assert_eq!(
            reduced_constraints[4],
            ReducedConstraint {
                // -s3
                a: Some(Term(3, Fr::from(-1))),
                // s4
                b: Some(Term(5, Fr::from(1))),
                // s7
                c: Some(Term(7, Fr::from(1))),
                operation: Operation::Mul
            }
        );
    }
}
