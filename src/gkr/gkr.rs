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
    sumcheck_proof: Vec<PartialSumcheckProof<F, GateEvalExtension<F>>>,
    q_functions: Vec<UnivariatePolynomial<F>>,
}

// TODO: add documentation
fn prove<F: PrimeField>(circuit: Circuit, input: Vec<F>) -> Result<GKRProof<F>, &'static str> {
    let mut transcript = Transcript::new();
    let mut sumcheck_proofs = vec![];
    let mut q_functions = vec![];

    // evaluate the circuit at the given input
    let evaluations = circuit.evaluate(input)?;

    // get the mle of the output evaluation layer
    let w_0 = Circuit::w(evaluations.as_slice(), 0)?;

    // push that to the transcript
    transcript.append(w_0.to_bytes().as_slice());

    // sample k field elements to make r
    let mut r = transcript.sample_n_field_elements::<F>(w_0.n_vars());

    // evaluate wo(r) to get m
    let mut m = w_0.evaluate(r.as_slice())?;

    for layer_index in 1..evaluations.len() {
        let [add_mle, mul_mle] = circuit.add_mul_mle(layer_index)?;
        let w_i = Circuit::w(evaluations.as_slice(), layer_index)?;
        let f_a_b = GateEvalExtension::new(r.clone(), add_mle, mul_mle, w_i.clone())?;

        let (partial_sumcheck_proof, challenges) = Sumcheck::prove_partial(f_a_b, m);
        transcript.append(partial_sumcheck_proof.to_bytes().as_slice());
        sumcheck_proofs.push(partial_sumcheck_proof);

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
        sumcheck_proof: sumcheck_proofs,
        q_functions,
    })
}

// TODO: add documentation
fn verify<F: PrimeField>(proof: GKRProof<F>) -> bool {
    todo!()
}
