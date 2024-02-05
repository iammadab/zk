use crate::circom_gkr::constraint::{Constraint, ReducedConstraint, Term};
use ark_ff::PrimeField;
use std::collections::HashMap;

#[derive(Clone)]
/// Represents an R1CSProgram as a collection of constraints
pub struct R1CSProgram<F: PrimeField> {
    constraints: Vec<Constraint<F>>,
}

impl<F: PrimeField> R1CSProgram<F> {
    /// Create a new R1CS Program from constraint set
    pub fn new(constraints: Vec<Constraint<F>>) -> Self {
        Self { constraints }
    }

    /// Get the highest index assigned to a variable
    fn get_last_variable_index(&self) -> usize {
        self.constraints
            .iter()
            .map(|constraint| constraint.max_index())
            .max()
            .unwrap_or(0)
    }

    // TODO: you might need to return more than this
    //  potentially will need to return the symbol table also
    /// Compiles a list of Constraint into a list of ReducedConstraint
    pub fn compile(mut self) -> (Vec<ReducedConstraint<F>>, SymbolTable<F>) {
        let mut symbol_table = SymbolTable::<F>::new(self.get_last_variable_index());
        let mut reduced_constraints = vec![];

        for constraint in self.constraints {
            reduced_constraints.extend(constraint.reduce(&mut symbol_table))
        }

        (reduced_constraints, symbol_table)
    }
}

/// Keeps track of variable data
pub struct SymbolTable<F: PrimeField> {
    pub variable_map: HashMap<(Term<F>, Term<F>), usize>,
    pub last_variable_index: usize,
}

impl<F: PrimeField> SymbolTable<F> {
    pub fn new(last_variable_index: usize) -> Self {
        Self {
            variable_map: HashMap::new(),
            last_variable_index,
        }
    }

    /// Check if we have merge (a, b) or (b, a) before,
    /// if we have, it returns the previously assigned variable
    /// if not, it assigns a new variable, stores that and returns it
    pub fn get_variable_index(&mut self, a: Term<F>, b: Term<F>) -> usize {
        if let Some(index) = self.variable_map.get(&(a, b)) {
            *index
        } else if let Some(index) = self.variable_map.get(&(b, a)) {
            *index
        } else {
            // no previous occurence of the variable pair
            // add them to the map and return that index
            let index = self.last_variable_index + 1;
            self.variable_map.insert((a, b), index);
            self.last_variable_index = index;
            index
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::circom_gkr::constraint::{Constraint, Term};
    use crate::circom_gkr::program::R1CSProgram;
    use ark_bls12_381::Fr;

    fn quadratic_checker_circuit() -> R1CSProgram<Fr> {
        // constraints
        // -x * x = -x_squared
        // -a * x_squared = -first_term
        // -b * x = -second_term
        // 0 = first_term + second_term - partial_sum
        // 0 = c + partial_sum - in[0]
        // 0 = res - in[1]
        // 0 = -out + equal_out
        // 0 = -in[0] + in[1] - isz_in
        // 0 = -equal_out + isz_out
        // isz_in . isz_inv = 1 - isz_out
        // isz_in . isz_out = 0

        // symbol index values
        // out = 1
        // x = 2
        // a = 3
        // b = 4
        // c = 5
        // res = 6
        // x_squared = 7
        // first_term = 8
        // second_term = 9
        // partial_sum = 10
        // equal_out = 11
        // in[0] = 12
        // in[1] = 13
        // isz_out = 14
        // isz_in = 15
        // isz_inv = 16

        let constraints = vec![
            Constraint::new(
                vec![Term(2, Fr::from(-1))],
                vec![Term(2, Fr::from(1))],
                vec![Term(7, Fr::from(-1))],
            ),
            Constraint::new(
                vec![Term(3, Fr::from(-1))],
                vec![Term(7, Fr::from(1))],
                vec![Term(8, Fr::from(-1))],
            ),
            Constraint::new(
                vec![Term(4, Fr::from(-1))],
                vec![Term(2, Fr::from(1))],
                vec![Term(9, Fr::from(-1))],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![
                    Term(8, Fr::from(1)),
                    Term(9, Fr::from(1)),
                    Term(10, Fr::from(-1)),
                ],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![
                    Term(5, Fr::from(1)),
                    Term(10, Fr::from(1)),
                    Term(12, Fr::from(-1)),
                ],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![Term(6, Fr::from(1)), Term(13, Fr::from(-1))],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![Term(1, Fr::from(-1)), Term(11, Fr::from(1))],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![
                    Term(12, Fr::from(-1)),
                    Term(13, Fr::from(1)),
                    Term(15, Fr::from(-1)),
                ],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![Term(11, Fr::from(-1)), Term(14, Fr::from(1))],
            ),
            Constraint::new(
                vec![Term(15, Fr::from(1))],
                vec![Term(16, Fr::from(1))],
                vec![Term(0, Fr::from(1)), Term(14, Fr::from(-1))],
            ),
            Constraint::new(
                vec![Term(15, Fr::from(1))],
                vec![Term(14, Fr::from(1))],
                vec![],
            ),
        ];

        R1CSProgram::new(constraints)
    }

    pub fn eq_3a_plus_5b() -> R1CSProgram<Fr> {
        // constraints
        // 0 = 3a - threea
        // 0 = 5b - fiveb
        // 0 = -c + threea + fiveb

        // symbol index values
        // c = 1
        // a = 2
        // b = 3
        // threea = 4
        // fiveb = 5

        let constraints = vec![
            Constraint::new(
                vec![],
                vec![],
                vec![Term(2, Fr::from(3)), Term(4, Fr::from(-1))],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![Term(3, Fr::from(5)), Term(5, Fr::from(-1))],
            ),
            Constraint::new(
                vec![],
                vec![],
                vec![
                    Term(1, Fr::from(-1)),
                    Term(4, Fr::from(1)),
                    Term(5, Fr::from(1)),
                ],
            ),
        ];

        R1CSProgram::new(constraints)
    }

    #[test]
    fn test_get_last_variable_index() {
        let quadratic_program = quadratic_checker_circuit();
        assert_eq!(quadratic_program.get_last_variable_index(), 16);
    }

    #[test]
    fn test_compile_program() {
        let quadratic_program = quadratic_checker_circuit();
        let (compiled_program, _) = quadratic_program.compile();

        // most constraints in the quadratic program only required moving data around
        // expect 1, which will require a reduction into a new constraint
        // therefore we are expecting only 1 constraint
        // 11 + 1 = 12
        assert_eq!(compiled_program.len(), 12);

        let eq_3a_plus_5b = eq_3a_plus_5b();
        let (compiled_program, _) = eq_3a_plus_5b.compile();
        assert_eq!(compiled_program.len(), 3);
    }
}
