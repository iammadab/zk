// TODO: implement reduced constraint to circuit
//  this will make use of the constant map

// TODO: implement multi constraint circuit construction

use crate::circom_gkr::constraint::ReducedConstraint;
use ark_ff::PrimeField;
use std::collections::HashMap;

/// Generate a mapping from a constant value to variable index
/// witness array is of the form
/// [1, ...intermediate_variables..., ...constants...]
/// hence 1 always has variable index 0
/// the other constants take (number_of_variable + 1)...
fn generate_constant_map<F: PrimeField>(
    reduced_constraints: &[ReducedConstraint<F>],
    mut last_variable_index: usize,
) -> HashMap<F, usize> {
    let mut constant_map: HashMap<F, usize> = [(F::one(), 0)].into();

    for constraint in reduced_constraints {
        constraint.a.map(|term| {
            if !constant_map.contains_key(&term.1) {
                last_variable_index += 1;
                constant_map.insert(term.1, last_variable_index);
            }
        });

        constraint.b.map(|term| {
            if !constant_map.contains_key(&term.1) {
                last_variable_index += 1;
                constant_map.insert(term.1, last_variable_index);
            }
        });

        constraint.c.map(|term| {
            if !constant_map.contains_key(&term.1) {
                last_variable_index += 1;
                constant_map.insert(term.1, last_variable_index);
            }
        });
    }

    constant_map
}

#[cfg(test)]
mod tests {
    use crate::circom_gkr::circuit::generate_constant_map;
    use crate::circom_gkr::constraint::{Constraint, Term};
    use crate::circom_gkr::program::R1CSProgram;
    use ark_bls12_381::Fr;

    #[test]
    fn test_generate_constant_map() {
        let mut program = R1CSProgram::new(vec![Constraint::new(
            vec![
                Term(0, Fr::from(1)),
                Term(1, Fr::from(-1)),
                Term(2, Fr::from(-5)),
                Term(3, Fr::from(1)),
            ],
            vec![],
            vec![],
        )]);

        let (reduced_constraints, symbol_table) = program.compile();
        assert_eq!(reduced_constraints.len(), 2);
        assert_eq!(symbol_table.variable_map.len(), 1);

        let constant_map = generate_constant_map(
            reduced_constraints.as_slice(),
            symbol_table.last_variable_index,
        );

        // constant map should contain 3 constants [1, -1, 5]
        // before reduction last known variable index was a 3
        // reduction will create a new variable which will have index 4
        // constant 1 is always 0
        // we are left with constant -1 and 5
        // -1 will take 5 and 5 will take 6
        // (note the values they take depend on the order they were seen)
        assert_eq!(constant_map.len(), 3);
        assert_eq!(constant_map.get(&Fr::from(1)).unwrap(), &0);
        assert_eq!(constant_map.get(&Fr::from(-1)).unwrap(), &5);
        assert_eq!(constant_map.get(&Fr::from(5)).unwrap(), &6);
    }
}
