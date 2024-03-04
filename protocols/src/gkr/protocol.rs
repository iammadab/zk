use crate::gkr::circuit::Circuit;
use crate::gkr::gate_eval_extension::GateEvalExtension;
use crate::gkr::util::{evaluate_l_function, l, q};
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::polynomial::Polynomial;
use crate::sumcheck::{PartialSumcheckProof, Sumcheck};
use crate::transcript::Transcript;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Proof<F: PrimeField> {
    // TODO: seems it might be better to return the output points directly i.e. Vec<F>
    //  feels like it will constrain the prover better. Don't make this change if you haven't
    //  figured out a way to break it!!!!
    output_mle: MultiLinearPolynomial<F>,
    pub sumcheck_proofs: Vec<PartialSumcheckProof<F>>,
    q_functions: Vec<UnivariatePolynomial<F>>,
}

/// Prove correct circuit evaluation using the GKR protocol
pub fn prove<F: PrimeField>(
    circuit: Circuit,
    evaluations: Vec<Vec<F>>,
) -> Result<Proof<F>, &'static str> {
    // TODO: do I need to add the circuit and the input to the transcript
    let mut transcript = Transcript::new();
    let mut sumcheck_proofs = vec![];
    let mut q_functions = vec![];

    // get the mle of the output evaluation layer and add to transcript
    let w_0 = Circuit::w(evaluations.as_slice(), 0)?;
    transcript.append(w_0.to_bytes().as_slice());

    // sample k random field elements to make r
    let mut r = transcript.sample_n_field_elements::<F>(w_0.n_vars());

    // evaluate w_0(r) to get m
    let mut m = w_0.evaluate(r.as_slice())?;

    // f(b, c) = add(r, b, c)(w_i(b) + w_i(c)) + mul(r, b, c)(w_i(b) * w_i(c))
    // each gkr round show that m = sum of f(b, c) over the boolean hypercube
    for layer_index in 1..evaluations.len() {
        let [add_mle, mul_mle] = circuit.add_mul_mle(layer_index - 1)?;
        let w_i = Circuit::w(evaluations.as_slice(), layer_index)?;
        let f_b_c = GateEvalExtension::new(r.clone(), add_mle, mul_mle, w_i.clone())?;

        let (partial_sumcheck_proof, challenges) = Sumcheck::prove_partial(f_b_c, m);
        transcript.append(partial_sumcheck_proof.to_bytes().as_slice());
        sumcheck_proofs.push(partial_sumcheck_proof);

        // since the verifier doesn't have access the w_i
        // we need to create a new polynomial q, which restricts w_i to a straight line l
        // i.e q(x) = w(l(x))
        // where l(0) = b and l(1) = c
        let (b, c) = challenges.split_at(challenges.len() / 2);
        let l_function = l(b, c)?;
        let q_function = q(l_function.as_slice(), w_i.clone())?;

        transcript.append(q_function.to_bytes().as_slice());
        q_functions.push(q_function);

        let r_star = transcript.sample_field_element();
        r = evaluate_l_function(l_function.as_slice(), &r_star);
        m = w_i.evaluate(r.as_slice())?;
    }

    Ok(Proof {
        output_mle: w_0,
        sumcheck_proofs,
        q_functions,
    })
}

/// Verify a GKR proof
pub fn verify<F: PrimeField>(
    circuit: Circuit,
    input: Vec<F>,
    proof: Proof<F>,
) -> Result<bool, &'static str> {
    if proof.sumcheck_proofs.len() != proof.q_functions.len() {
        return Err("invalid gkr proof");
    }

    let mut transcript = Transcript::new();
    transcript.append(proof.output_mle.to_bytes().as_slice());

    let mut r = transcript.sample_n_field_elements(proof.output_mle.n_vars());
    let mut m = proof.output_mle.evaluate(r.as_slice())?;

    let sumcheck_and_q_functions = proof
        .sumcheck_proofs
        .clone()
        .into_iter()
        .zip(proof.q_functions);

    // Verify each sumcheck proof and update next round parameters
    for (layer_index, (partial_sumcheck_proof, q_function)) in sumcheck_and_q_functions.enumerate()
    {
        // here we ensure that the sumcheck proof proves the correct sum
        if partial_sumcheck_proof.sum != m {
            return Err("invalid sumcheck proof");
        }

        transcript.append(partial_sumcheck_proof.to_bytes().as_slice());
        transcript.append(q_function.to_bytes().as_slice());

        let subclaim = Sumcheck::verify_partial(partial_sumcheck_proof)
            .ok_or("failed to verify partial sumcheck proof")?;

        // we need to perform the last check ourselves
        // basically evaluate f(b, c) at the challenge points
        // recall f(b, c) = add(r, b, c)(w(b) + w(c)) + mul(r, b, c)(w(b) * w(c))
        // w(b) = q(0) and w(c) = q(1)
        // f(b, c) = add(r, b, c)(q(0) + q(1)) + mul(r, b, c)(q(0) * q(1))
        // verify that the above is equal to the sum in the subclaim

        if subclaim.challenges.len() % 2 != 0 {
            return Err("challenges b and c should be the same length");
        }

        let (b, c) = subclaim.challenges.split_at(subclaim.challenges.len() / 2);
        let [add_mle, mul_mle] = circuit.add_mul_mle(layer_index)?;
        let mut rbc = r.clone();
        rbc.extend(&subclaim.challenges);

        let w_b = q_function.evaluate(&F::zero());
        let w_c = q_function.evaluate(&F::one());
        let add_result = add_mle.evaluate(rbc.as_slice())? * (w_b + w_c);
        let mul_result = mul_mle.evaluate(rbc.as_slice())? * (w_b * w_c);
        let f_b_c_eval = add_result + mul_result;

        // final sumcheck verifier check
        if f_b_c_eval != subclaim.sum {
            return Ok(false);
        }

        let l_function = l(b, c)?;
        let r_star = transcript.sample_field_element();

        r = evaluate_l_function(l_function.as_slice(), &r_star);
        m = q_function.evaluate(&r_star);
    }

    // since the verifier has access to the input layer
    // the verifier can check for the correctness of last m itself
    // by evaluating the input_mle at r and comparing that to the claimed m
    let input_mle = MultiLinearPolynomial::<F>::interpolate(input.as_slice());
    let actual_m = input_mle.evaluate(r.as_slice())?;
    Ok(actual_m == m)
}

#[cfg(test)]
mod test {
    use crate::gkr::circuit::tests::test_circuit;
    use crate::gkr::circuit::Circuit;
    use crate::gkr::gate::Gate;
    use crate::gkr::layer::Layer;
    use crate::gkr::protocol::{prove as GKRProve, verify as GKRVerify, Proof as GKRProof};
    use ark_bls12_381::Fr;
    use ark_ff::{Fp64, MontBackend, MontConfig};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use std::io::Cursor;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_serialize_and_deserialize_gkrproof() {
        let circuit = test_circuit();
        let input = vec![
            Fr::from(1),
            Fr::from(2),
            Fr::from(3),
            Fr::from(4),
            Fr::from(5),
            Fr::from(6),
            Fr::from(7),
            Fr::from(8),
        ];
        let circut_eval = circuit.evaluate(input.clone()).unwrap();
        let gkr_proof = GKRProve(test_circuit(), circut_eval).unwrap();

        let mut serialized_gkr_proof = vec![];
        gkr_proof
            .serialize_uncompressed(&mut serialized_gkr_proof)
            .map_err(|_| "failed to serialize proof")
            .unwrap();

        let deserialized_gkr_proof: GKRProof<Fr> =
            GKRProof::deserialize_compressed(Cursor::new(serialized_gkr_proof)).unwrap();

        assert_eq!(deserialized_gkr_proof, gkr_proof);
    }

    #[test]
    fn test_gkr() {
        let circuit = test_circuit();
        let input = vec![
            Fr::from(1),
            Fr::from(2),
            Fr::from(3),
            Fr::from(4),
            Fr::from(5),
            Fr::from(6),
            Fr::from(7),
            Fr::from(8),
        ];
        let circut_eval = circuit.evaluate(input.clone()).unwrap();
        let gkr_proof = GKRProve(test_circuit(), circut_eval).unwrap();

        let verification_result = GKRVerify(test_circuit(), input, gkr_proof).unwrap();
        assert!(verification_result);
    }

    #[test]
    fn test_wrong_eval_gkr() {
        let circuit = test_circuit();
        let wrong_input = vec![
            Fr::from(1),
            Fr::from(2),
            Fr::from(3),
            Fr::from(4),
            Fr::from(5),
            Fr::from(6),
            Fr::from(7),
            Fr::from(8),
        ];
        let invalid_eval = circuit.evaluate(wrong_input).unwrap();
        let invalid_gkr_proof = GKRProve(test_circuit(), invalid_eval).unwrap();

        let actual_input = vec![
            Fr::from(12),
            Fr::from(2),
            Fr::from(3),
            Fr::from(49),
            Fr::from(5),
            Fr::from(6),
            Fr::from(7),
            Fr::from(8),
        ];
        let verification_result =
            GKRVerify(test_circuit(), actual_input, invalid_gkr_proof).unwrap();
        assert!(!verification_result);
    }

    #[test]
    fn test_output_zero_gkr() {
        let layer_0 = Layer::new(vec![], vec![Gate::new(0, 0, 1)]);
        let layer_1 = Layer::new(vec![Gate::new(0, 0, 1)], vec![Gate::new(1, 1, 2)]);
        let layer_2 = Layer::new(
            vec![Gate::new(0, 0, 1), Gate::new(2, 4, 5)],
            vec![Gate::new(1, 2, 3)],
        );

        let circuit = Circuit::new(vec![layer_0, layer_1, layer_2]).unwrap();
        let eval_input = vec![
            Fq::from(5),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(9),
            Fq::from(8),
        ];

        let evaluation = circuit.evaluate(eval_input.clone()).unwrap();
        let gkr_proof = GKRProve(circuit.clone(), evaluation.clone()).unwrap();

        let verification_result = GKRVerify(circuit, eval_input, gkr_proof).unwrap();
        assert!(verification_result);
    }
}
