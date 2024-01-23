use crate::circom_gkr::constraint::{Constraint, ReducedConstraint};
use ark_ff::PrimeField;

/// Represents an R1CSProgram as a collection of constraints
struct R1CSProgram<F: PrimeField> {
    constraints: Vec<Constraint<F>>,
}

impl<F: PrimeField> R1CSProgram<F> {
    /// Create a new R1CS Program from constraint set
    fn new(constraints: Vec<Constraint<F>>) -> Self {
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
    fn compile(mut self) -> Vec<ReducedConstraint<F>> {
        // do I need to return the symbol table or is that only useful for
        // co-ordinating the other compilation steps?
        todo!()
    }
}

#[cfg(test)]
mod test {
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

    #[test]
    fn test_get_last_variable_index() {
        let quadratic_program = quadratic_checker_circuit();
        assert_eq!(quadratic_program.get_last_variable_index(), 16);
    }

    #[test]
    fn test_compile_program() {}
}
