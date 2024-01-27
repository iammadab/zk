// TODO: implement circuit index
// TODO: implement multi constraint circuit construction

use crate::circom_gkr::constraint::{Operation, ReducedConstraint, Term};
use crate::gkr::circuit::Circuit as GKRCircuit;
use crate::gkr::gate::Gate;
use crate::gkr::layer::Layer;
use ark_ff::PrimeField;
use std::collections::HashMap;

/// Build a gkr circuit that checks the relation:
/// A op B = C, where op is either add or mul
/// i.e A + B = c or A * B = C
fn constraint_circuit<F: PrimeField>(
    constraint: &ReducedConstraint<F>,
    constant_map: HashMap<F, usize>,
    constraint_index: usize,
) -> GKRCircuit {
    //                        +                           // output layer
    //              /                    \
    //          OP                          x             // compute layer
    //       /      \                   /        \
    //    x            x              x             x     // sign layer
    //  /   \       /     \        /     \       /     \
    // A   a_val   B    b_val     C    c_val    1      -1

    let output_layer_offset = constraint_index;
    let compute_layer_offset = output_layer_offset * 2;
    let sign_layer_offset = compute_layer_offset * 2;

    let circuit_input = reduced_constraint_to_circuit_input(constraint, &constant_map);
    let one_index = constant_map.get(&F::one()).unwrap();
    let minus_one_index = constant_map.get(&F::one().neg()).unwrap();

    // TODO: add better documentation and come up with better names
    // sign layer
    let a_mul_gate = Gate::new(
        0 + sign_layer_offset,
        circuit_input[0].0,
        circuit_input[0].1,
    );
    let b_mul_gate = Gate::new(
        1 + sign_layer_offset,
        circuit_input[1].0,
        circuit_input[1].1,
    );
    let c_mul_gate = Gate::new(
        2 + sign_layer_offset,
        circuit_input[2].0,
        circuit_input[2].1,
    );
    let minus_1_gate = Gate::new(3 + sign_layer_offset, *one_index, *minus_one_index);
    let sign_layer = Layer::new(
        vec![],
        vec![a_mul_gate, b_mul_gate, c_mul_gate, minus_1_gate],
    );

    // compute layer
    // computes A op B where op is either + or *
    // and computes -c
    let a_op_b_gate = Gate::new(
        0 + compute_layer_offset,
        0 + sign_layer_offset,
        1 + sign_layer_offset,
    );
    let c_mul_minus_1 = Gate::new(
        1 + compute_layer_offset,
        2 + sign_layer_offset,
        3 + sign_layer_offset,
    );
    let compute_layer = match constraint.operation {
        Operation::Add => Layer::new(vec![a_op_b_gate], vec![c_mul_minus_1]),
        Operation::Mul => Layer::new(vec![], vec![a_op_b_gate, c_mul_minus_1]),
    };

    // output layer
    let output_gate = Gate::new(
        output_layer_offset,
        0 + compute_layer_offset,
        1 + compute_layer_offset,
    );
    let output_layer = Layer::new(vec![output_gate], vec![]);

    GKRCircuit::new(vec![output_layer, compute_layer, sign_layer])
}

/// Replace the optional values in the reduced constraint with concrete values
/// to feed into the gkr circuit builder
fn reduced_constraint_to_circuit_input<F: PrimeField>(
    constraint: &ReducedConstraint<F>,
    constant_map: &HashMap<F, usize>,
) -> [(usize, usize); 3] {
    // Reduced constraints have optional values for a and b
    // our gkr circuit for single constraint satisfaction doesn't account for optional values
    // hence we need to convert those optional values to concrete values while preserving equation meaning
    // equations are either:
    // a * b = c (mul gate)
    // a + b = c (add gate)
    // in both cases, if c is empty it can be converted to 0, without losing meaning
    // if a or b is empty then we return the gate identity i.e mul -> 1 and add -> 0
    // examples
    // 2 + ? = 2 -> replace ? with 0 -> 2 + 0 = 2
    // 4 * ? = 4 -> replace ? with 1 -> 4 * 1 = 4

    let term_to_index = |t: Term<F>| -> (usize, usize) { (t.0, *constant_map.get(&t.1).unwrap()) };

    let zero_index = *constant_map.get(&F::zero()).unwrap();
    let zero_value = (zero_index, zero_index);

    let one_index = *constant_map.get(&F::one()).unwrap();
    let one_value = (one_index, one_index);

    let default_value = match constraint.operation {
        // for addition gates, terms that are empty can be zero
        Operation::Add => zero_value,
        Operation::Mul => one_value,
    };

    let a_value = constraint.a.map(term_to_index).unwrap_or(default_value);
    let b_value = constraint.b.map(term_to_index).unwrap_or(default_value);
    let c_value = constraint.c.map(term_to_index).unwrap_or(zero_value);

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
    use crate::circom_gkr::circuit::{
        constraint_circuit, generate_constant_map, reduced_constraint_to_circuit_input,
    };
    use crate::circom_gkr::constraint::{Constraint, Operation, ReducedConstraint, Term};
    use crate::circom_gkr::program::R1CSProgram;
    use crate::gkr::gkr::{GKRProve, GKRVerify};
    use ark_bls12_381::Fr;
    use ark_ff::{One, Zero};
    use std::collections::HashMap;

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
        let constant_map: HashMap<Fr, usize> =
            [(Fr::from(0), 0), (Fr::from(1), 1), (Fr::from(2), 2)].into();

        // ? + b = c
        // expected 0 + b = c
        let r1 = ReducedConstraint {
            a: None,
            b: Some(Term(2, Fr::from(1))),
            c: Some(Term(3, Fr::from(2))),
            operation: Operation::Add,
        };
        assert_eq!(
            reduced_constraint_to_circuit_input(&r1, &constant_map),
            [(0, 0), (2, 1), (3, 2)]
        );

        // ? * b = c
        let r2 = ReducedConstraint {
            a: None,
            b: Some(Term(2, Fr::from(1))),
            c: Some(Term(3, Fr::from(2))),
            operation: Operation::Mul,
        };
        assert_eq!(
            reduced_constraint_to_circuit_input(&r2, &constant_map),
            [(1, 1), (2, 1), (3, 2)]
        );

        // a * b = ?
        let r3 = ReducedConstraint {
            a: Some(Term(2, Fr::from(1))),
            b: Some(Term(3, Fr::from(2))),
            c: None,
            operation: Operation::Mul,
        };
        assert_eq!(
            reduced_constraint_to_circuit_input(&r3, &constant_map),
            [(2, 1), (3, 2), (0, 0)]
        );

        // a + b = ?
        let r4 = ReducedConstraint {
            a: Some(Term(2, Fr::from(1))),
            b: Some(Term(3, Fr::from(2))),
            c: None,
            operation: Operation::Add,
        };
        assert_eq!(
            reduced_constraint_to_circuit_input(&r4, &constant_map),
            [(2, 1), (3, 2), (0, 0)]
        );
    }

    #[test]
    fn test_constraint_circuit_evaluation() {
        // a * b = c
        // 3 variables
        // input = [1, a, b, c, 0, -1]

        let constant_map = [(Fr::one(), 0), (Fr::from(0), 4), (Fr::from(-1), 5)].into();
        let constraint = ReducedConstraint {
            a: Some(Term(1, Fr::one())),
            b: Some(Term(2, Fr::one())),
            c: Some(Term(3, Fr::one())),
            operation: Operation::Mul,
        };

        let circuit = constraint_circuit(&constraint, constant_map, 0);

        // example
        // 2 * 3 = 6
        // correct input = [1, 2, 3, 6, 0, -1]
        // bad input = [1, 3, 3, 6, 0, -1]

        // evaluate with bad input
        // output should not be 0
        let bad_evaluation_result = circuit
            .evaluate(vec![
                Fr::one(),
                Fr::from(3),
                Fr::from(3),
                Fr::from(6),
                Fr::zero(),
                Fr::from(-1),
            ])
            .unwrap();
        let mut result_iter = bad_evaluation_result.iter().rev();
        // skip input layer
        result_iter.next();
        assert_eq!(
            result_iter.next().unwrap(),
            &vec![Fr::from(3), Fr::from(3), Fr::from(6), Fr::from(-1)]
        );
        assert_eq!(
            result_iter.next().unwrap(),
            &vec![Fr::from(9), Fr::from(-6)]
        );
        assert_eq!(result_iter.next().unwrap(), &vec![Fr::from(3)]);

        // evaluate with valid input
        let correct_evaluation_result = circuit
            .evaluate(vec![
                Fr::one(),
                Fr::from(2),
                Fr::from(3),
                Fr::from(6),
                Fr::zero(),
                Fr::from(-1),
            ])
            .unwrap();
        let mut result_iter = correct_evaluation_result.iter().rev();
        // skip input layer
        result_iter.next();
        assert_eq!(
            result_iter.next().unwrap(),
            &vec![Fr::from(2), Fr::from(3), Fr::from(6), Fr::from(-1)]
        );
        assert_eq!(
            result_iter.next().unwrap(),
            &vec![Fr::from(6), Fr::from(-6)]
        );
        assert_eq!(result_iter.next().unwrap(), &vec![Fr::from(0)]);
    }
}
