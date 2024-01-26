// TODO: implement reduced constraint to circuit
//  this will make use of the constant map

// TODO: implement multi constraint circuit construction

use crate::circom_gkr::constraint::{Operation, ReducedConstraint};
use crate::gkr::circuit::Circuit as GKRCircuit;
use ark_ff::PrimeField;
use std::collections::HashMap;

// TODO: add documentation
fn constraint_circuit<F: PrimeField>(
    constraint: &ReducedConstraint<F>,
    constant_map: HashMap<F, usize>,
) -> GKRCircuit {
    //                        +
    //              /                    \
    //          OP                          x
    //       /      \                   /        \
    //    x            x              x             x
    //  /   \       /     \        /     \       /     \
    // A   a_val   B    b_val     C    c_val    1      -1
    todo!()
}

// TODO: add documentation
fn reduced_constraint_to_circuit_input<F: PrimeField>(
    constraint: &ReducedConstraint<F>,
    constant_map: &HashMap<F, usize>,
) -> [(usize, F); 3] {
    let zero_value = (*constant_map.get(&F::zero()).unwrap(), F::zero());
    let one_value = (*constant_map.get(&F::one()).unwrap(), F::one());
    let default_value = match constraint.operation {
        // for addition gates, terms that are empty can be zero
        Operation::Add => zero_value,
        Operation::Mul => one_value
    };

    let a_value = constraint.a.map(|t| t.into()).unwrap_or(default_value);
    let b_value = constraint.b.map(|t| t.into()).unwrap_or(default_value);
    let c_value = constraint.c.map(|t| t.into()).unwrap_or(zero_value);

    [a_value, b_value, c_value]
}

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

    // insert the constant 0
    last_variable_index += 1;
    constant_map.insert(F::zero(), last_variable_index);

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

        // constant map should contain 4 constants [1, -1, 5]
        // before reduction last known variable index was a 3
        // reduction will create a new variable which will have index 4
        // constant 1 is always 0
        // we are left with constant 0, -1 and 5
        // 0 will take 5, -1 will take 6 and 5 will take 7
        // (note the values they take depend on the order they were seen)
        assert_eq!(constant_map.len(), 4);
        assert_eq!(constant_map.get(&Fr::from(1)).unwrap(), &0);
        assert_eq!(constant_map.get(&Fr::from(0)).unwrap(), &5);
        assert_eq!(constant_map.get(&Fr::from(-1)).unwrap(), &6);
        assert_eq!(constant_map.get(&Fr::from(5)).unwrap(), &7);
    }

    #[test]
    fn test_reduced_constraint_to_circuit_input() {
        // how do I properly test this?
        // the basic idea is to correctly know what to do when different values in the reduced constraint are empty
        // is there a general rule for this?
        // A, B or C can be empty with any combination
        // we also need to make use of the constant map
        // the constant map should have a slot for 0
        todo!()
    }
}
