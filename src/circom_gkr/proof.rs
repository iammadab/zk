use crate::circom_gkr::circuit::program_circuit;
use crate::circom_gkr::program::R1CSProgram;
use crate::gkr::gkr::{GKRProof, GKRProve, GKRVerify};
use ark_ff::PrimeField;

// TODO: add documentation
// prover takes the witness and generates a gkr proof given some program
// TODO: witness value should be generated by the prover
//  consider making a prove with witness function
fn prove<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
) -> Result<GKRProof<F>, &'static str> {
    let circuit = program_circuit(program);
    let evaluations = circuit.evaluate(witness).unwrap();
    GKRProve(circuit, evaluations)
}

// TODO: add documentation
fn verify<F: PrimeField>(
    program: R1CSProgram<F>,
    witness: Vec<F>,
    proof: GKRProof<F>,
) -> Result<bool, &'static str> {
    let circuit = program_circuit(program);
    // TODO: is this a sufficient check
    if proof.sumcheck_proofs[0].sum != F::zero() {
        return Ok(false);
    }
    GKRVerify(circuit, witness, proof)
}

// TODO: write test that has correct witness structure but doesn't satisfy all constraints
//   should be able to generate proof but fail to verify because of the sum section

// TODO: write test that has correct witness structure + satisfies all constraints
//   all checks should pass

// TODO: figure out how to force the witness values for the constants

#[cfg(test)]
mod tests {
    use crate::circom_gkr::circuit::program_circuit;
    use crate::circom_gkr::circuit::tests::x_cube;
    use crate::circom_gkr::constraint::{Constraint, Term};
    use crate::circom_gkr::program::R1CSProgram;
    use crate::circom_gkr::proof::{prove, verify};
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
        // input structure [1, x, a, 0, -1]
        let witness = vec![
            Fr::one(),
            Fr::from(2),
            Fr::from(4),
            Fr::zero(),
            Fr::one().neg(),
        ];
        let proof = prove(x_square(), witness.clone()).unwrap();

        assert_eq!(verify(x_square(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_single_constraint_invalid_witness() {
        // program: x * x = a
        // invalid witness
        // x = 3
        // a = 4
        // input structure [1, x, a, 0, -1]
        let witness = vec![
            Fr::one(),
            Fr::from(3),
            Fr::from(4),
            Fr::zero(),
            Fr::one().neg(),
        ];
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
        // input structure [1, x, a, b, 0, -1]
        let witness = vec![
            Fr::one(),
            Fr::from(3),
            Fr::from(9),
            Fr::from(27),
            Fr::zero(),
            Fr::one().neg(),
        ];
        let proof = prove(x_cube(), witness.clone()).unwrap();
        assert_eq!(verify(x_cube(), witness, proof).unwrap(), true);
    }

    #[test]
    fn test_prove_verify_x_cube_bad_constant_values() {
        // TODO: add program comment
        // TODO: fix this test
        let witness = vec![
            Fr::from(0),
            Fr::from(2),
            Fr::from(4),
            Fr::from(8),
            Fr::from(0),
            Fr::from(0)
        ];
        let proof = prove(x_cube(), witness.clone()).unwrap();
        assert_eq!(verify(x_cube(), witness, proof).unwrap(), false);
    }
}
