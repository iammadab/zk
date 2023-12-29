use crate::gkr::circuit::Circuit;
use crate::gkr::gate_eval_extension::GateEvalExtension;
use crate::gkr::util::{evaluate_l_function, l, q};
use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::sumcheck::{PartialSumcheckProof, Sumcheck};
use crate::transcript::Transcript;
use ark_ff::PrimeField;

struct GKRProof<F: PrimeField> {
    output_mle: MultiLinearPolynomial<F>,
    sumcheck_proofs: Vec<PartialSumcheckProof<F, GateEvalExtension<F>>>,
    q_functions: Vec<UnivariatePolynomial<F>>,
}

/// Prove correct circuit evaluation using the GKR protocol
fn prove<F: PrimeField>(
    circuit: Circuit,
    evaluations: Vec<Vec<F>>,
) -> Result<GKRProof<F>, &'static str> {
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
        let [add_mle, mul_mle] = circuit.add_mul_mle(layer_index)?;
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

    Ok(GKRProof {
        output_mle: w_0,
        sumcheck_proofs,
        q_functions,
    })
}

/// Verify a GKR proof
// TODO: should I return a bool??
// TODO: add sectioned comments
fn verify<F: PrimeField>(
    circuit: Circuit,
    input: Vec<F>,
    proof: GKRProof<F>,
) -> Result<bool, &'static str> {
    if proof.sumcheck_proofs.len() != proof.q_functions.len() {
        return Err("invalid gkr proof");
    }

    let mut transcript = Transcript::new();
    transcript.append(proof.output_mle.to_bytes().as_slice());

    let mut r = transcript.sample_n_field_elements(proof.output_mle.n_vars());
    let mut m = proof.output_mle.evaluate(r.as_slice())?;
    let mut layer_index = 1;

    let sumcheck_and_q_functions = proof
        .sumcheck_proofs
        .clone()
        .into_iter()
        .zip(proof.q_functions.clone().into_iter());

    // Verify each sumcheck proof and update next round parameters
    for (partial_sumcheck_proof, q_function) in sumcheck_and_q_functions {
        // here we ensure that the sumcheck proof proves the correct sum
        if partial_sumcheck_proof.sum != m {
            return Err("invalid sumcheck proof");
        }

        // TODO: fix ordering
        transcript.append(q_function.to_bytes().as_slice());
        transcript.append(partial_sumcheck_proof.to_bytes().as_slice());

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
        layer_index += 1;
    }

    // since the verifier has access to the input layer
    // the verifier can check for the correctness of last m itself
    // by evaluating the input_mle at r and comparing that to the claimed m
    let input_mle = MultiLinearPolynomial::<F>::interpolate(input.as_slice());
    let actual_m = input_mle.evaluate(r.as_slice())?;
    if actual_m != m {
        return Ok(false);
    }

    Ok(true)
}
