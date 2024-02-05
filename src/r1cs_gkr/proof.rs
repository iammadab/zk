use crate::gkr::circuit::Circuit;
use crate::gkr::gkr::{GKRProof, GKRProve, GKRVerify};
use crate::r1cs_gkr::circuit::program_circuit;
use crate::r1cs_gkr::program::R1CSProgram;
use ark_ff::PrimeField;
use std::collections::HashMap;

/// Generate a GKR proof for a given R1CSProgram
fn prove<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
) -> Result<GKRProof<F>, &'static str> {
    let (circuit, constrained_witness) = compile_program_and_constrain_witness(program, witness)?;
    let evaluations = circuit.evaluate(constrained_witness).unwrap();
    GKRProve(circuit, evaluations)
}

/// Verify a GKR proof for a given R1CSProgram
fn verify<F: PrimeField>(
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
    let expected_witness_len = symbol_table.last_variable_index;
    let constrained_witness = constrain_witness(witness, constant_map, expected_witness_len)?;
    Ok((circuit, constrained_witness))
}

/// Add the circuit specific constant values to the witness array
/// structure: [1, ...witness..., ...remaining_constants...]
fn constrain_witness<F: PrimeField>(
    witness: Vec<F>,
    constant_map: HashMap<F, usize>,
    expected_witness_len: usize,
) -> Result<Vec<F>, &'static str> {
    if witness.len() != expected_witness_len {
        return Err("invalid witness length");
    }

    let mut witness_with_constants = vec![F::one()];
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
    use crate::r1cs_gkr::circuit::program_circuit;
    use crate::r1cs_gkr::circuit::tests::x_cube;
    use crate::r1cs_gkr::constraint::{Constraint, Term};
    use crate::r1cs_gkr::program::test::eq_3a_plus_5b;
    use crate::r1cs_gkr::program::R1CSProgram;
    use crate::r1cs_gkr::proof::{prove, verify};
    use ark_bls12_381::Fr;
    use ark_ff::{One, Zero};
    use std::ops::Neg;

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
        let proof = prove(x_square(), witness.clone()).unwrap();

        assert_eq!(verify(x_square(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_single_constraint_invalid_witness() {
        // program: x * x = a
        // invalid witness
        // x = 3
        // a = 4
        // input structure [x, a]
        let witness = vec![Fr::from(3), Fr::from(4)];
        let proof = prove(x_square(), witness.clone()).unwrap();
        assert_eq!(verify(x_square(), witness, proof).unwrap(), false);
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
        let proof = prove(x_cube(), witness.clone()).unwrap();
        assert_eq!(verify(x_cube(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_3a_plus_5b_plus_10() {
        // program
        // 3a = threea
        // 5b = fiveb
        // 10 - c = s1
        // -fiveb - threeb = s1

        // witness structure
        // [c, a, b, threea, fiveb]
        // valid witness
        // a = 2
        // b = 3
        // threea = 6
        // fiveb = 15
        // c = 21
        let witness = vec![
            Fr::from(21),
            Fr::from(2),
            Fr::from(3),
            Fr::from(6),
            Fr::from(15),
        ];
        let proof = prove(eq_3a_plus_5b(), witness.clone()).unwrap();
        assert_eq!(verify(eq_3a_plus_5b(), witness, proof).unwrap(), true);
    }
}
