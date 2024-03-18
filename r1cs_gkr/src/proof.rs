use crate::circuit::program_circuit;
use crate::constraint::Term;
use crate::program::{R1CSProgram, SymbolTable};
use ark_ff::PrimeField;
use gkr::circuit::Circuit;
use gkr::protocol::{prove as GKRProve, verify as GKRVerify, Proof as GKRProof};
use std::collections::HashMap;

type Operands<F> = (Term<F>, Term<F>);

/// Generate a GKR proof for a given R1CSProgram
pub fn prove_circom_gkr<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
) -> Result<GKRProof<F>, &'static str> {
    let (circuit, constrained_witness) = compile_program_and_constrain_witness(program, witness)?;
    let evaluations = circuit.evaluate(constrained_witness).unwrap();
    GKRProve(circuit, evaluations)
}

/// Verify a GKR proof for a given R1CSProgram
pub fn verify_circom_gkr<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
    proof: GKRProof<F>,
) -> Result<bool, &'static str> {
    let (circuit, constrained_witness) = compile_program_and_constrain_witness(program, witness)?;
    if proof.sumcheck_proofs[0].sum != F::zero() {
        return Ok(false);
    }
    GKRVerify(circuit, constrained_witness, proof)
}

/// Compile the R1CSProgram into a circuit and add circuit dependent constants / intermediate values
/// to the witness vector
fn compile_program_and_constrain_witness<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
) -> Result<(Circuit, Vec<F>), &'static str> {
    let (circuit, constant_map, symbol_table) = program_circuit(program);

    let expected_witness_len = symbol_table.last_variable_index - symbol_table.variable_map.len();
    if witness.len() != expected_witness_len {
        return Err("invalid witness length");
    }

    let witness = generate_intermediate_witness_values(witness, symbol_table);

    let constrained_witness = constrain_witness(witness, constant_map)?;
    Ok((circuit, constrained_witness))
}

/// Generate the actual witness values for intermediate variables created
/// during the simplification step i.e Constraints -> Reduced Constraints
fn generate_intermediate_witness_values<F: PrimeField>(
    mut witness: Vec<F>,
    symbol_table: SymbolTable<F>,
) -> Vec<F> {
    // sort the variable mapping by index
    // this basically sorts them by order of creation
    // and since each variable must use two variable that already exists
    // this order allows for all needed values to be present every time we want to perform
    // a variable value computation
    let mut variable_map_tuples: Vec<(Operands<F>, usize)> =
        symbol_table.variable_map.into_iter().collect();
    variable_map_tuples.sort_by(|a, b| a.1.cmp(&b.1));

    // insert a 1 at index 0 of the witness
    // some of the intermediate variables might reference index 0
    witness.insert(0, F::one());

    // compute c which is equal to a op b
    for ((term_a, term_b), result_index) in variable_map_tuples {
        let term_a_value = witness[term_a.0] * term_a.1;
        let term_b_value = witness[term_b.0] * term_b.1;
        let c = term_a_value + term_b_value;

        if result_index != witness.len() {
            panic!("expect the witness len to grow with the variable map index");
        }

        witness.push(c)
    }

    witness
}

/// Add the circuit specific constant values to the witness array
/// structure: [1, ...witness..., ...remaining_constants...]
/// expected input structure is [1, ...witness...]
/// hence we just need to add ...remaining_constants...
fn constrain_witness<F: PrimeField>(
    witness: Vec<F>,
    constant_map: HashMap<F, usize>,
) -> Result<Vec<F>, &'static str> {
    let mut witness_with_constants = vec![];
    witness_with_constants.extend(witness);

    let mut hash_map_tuples: Vec<(F, usize)> = constant_map.into_iter().collect();
    hash_map_tuples.sort_by(|a, b| a.1.cmp(&b.1));

    // we skip the first element in the sorted constant list
    // because we have already inserted the constant 1 at the start
    let remaining_constants = hash_map_tuples.into_iter().skip(1).map(|(val, _)| val);

    witness_with_constants.extend(remaining_constants);

    Ok(witness_with_constants)
}

#[cfg(test)]
mod tests {
    use crate::circuit::tests::x_cube;
    use crate::constraint::{Constraint, Term};
    use crate::program::test::eq_3a_plus_5b;
    use crate::program::R1CSProgram;
    use crate::proof::{prove_circom_gkr, verify_circom_gkr};
    use ark_bls12_381::Fr;

    fn x_square() -> R1CSProgram<Fr> {
        // x * x = a
        R1CSProgram::new(vec![Constraint::new(
            vec![Term(1, Fr::from(1))],
            vec![Term(1, Fr::from(1))],
            vec![Term(2, Fr::from(1))],
        )])
    }

    #[test]
    fn test_prove_verify_single_constraint() {
        // program: x * x = a
        // valid witness
        // x = 2
        // a = 4
        // input structure [x, a]
        let witness = vec![Fr::from(2), Fr::from(4)];
        let proof = prove_circom_gkr(x_square(), witness.clone()).unwrap();

        assert_eq!(verify_circom_gkr(x_square(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_single_constraint_invalid_witness() {
        // program: x * x = a
        // invalid witness
        // x = 3
        // a = 4
        // input structure [x, a]
        let witness = vec![Fr::from(3), Fr::from(4)];
        let proof = prove_circom_gkr(x_square(), witness.clone()).unwrap();
        assert_eq!(
            verify_circom_gkr(x_square(), witness, proof).unwrap(),
            false
        );
    }

    #[test]
    fn test_prove_verify_x_cube() {
        // program:
        //  x * x = a
        //  a * x = b
        // valid witness
        //  x = 3
        //  a = 9
        //  b = 27
        // input structure [x, a, b]
        let witness = vec![Fr::from(3), Fr::from(9), Fr::from(27)];
        let proof = prove_circom_gkr(x_cube(), witness.clone()).unwrap();
        assert_eq!(verify_circom_gkr(x_cube(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_3a_plus_5b_plus_10() {
        // program
        // 3a = threea
        // 5b = fiveb
        // 10 - c = s1
        // -fiveb - threeb = s1

        // witness structure
        // [a, b, c, threea, fiveb]
        // valid witness
        // a = 2
        // b = 3
        // threea = 6
        // fiveb = 15
        // c = 31
        let witness = vec![
            Fr::from(2),
            Fr::from(3),
            Fr::from(31),
            Fr::from(6),
            Fr::from(15),
        ];
        let proof = prove_circom_gkr(eq_3a_plus_5b(), witness.clone()).unwrap();
        assert_eq!(
            verify_circom_gkr(eq_3a_plus_5b(), witness, proof).unwrap(),
            true
        );
    }
}
