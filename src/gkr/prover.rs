use crate::gkr::circuit::Circuit;
use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::transcript::Transcript;
use ark_ff::PrimeField;

struct GKRProof {}

fn prove<F: PrimeField>(circuit: Circuit, input: Vec<F>) -> Result<GKRProof, &'static str> {
    let mut transcript = Transcript::new();

    // evaluate the circuit at the given input
    let evaluations = circuit.evaluate(input)?;

    // get the mle of the output evaluation layer
    let w_0 = Circuit::w(evaluations.as_slice(), 0)?;

    // push that to the transcript
    transcript.append(w_0.to_bytes().as_slice());

    // sample k field elements to make r
    let r = transcript.sample_n_field_elements::<F>(w_0.n_vars());

    // evaluate wo(r) to get m
    let mut m = w_0.evaluate(r.as_slice())?;

    // we need to use w_i
    // do we ever need the input layer?
    // the final f would require that
    // so we need the input in the evaluation array

    // LABEL A
    // generate gate_eval_expression
    // generate partial sumcheck proof Vec<PartialSumcheckProof>
    // generate l function for the sumcheck challenges
    // generate q function
    // add q to the transcript
    // store q function, should we store l function too???
    //  - might personally prefer if the verifier regenerates the l function
    // sample random field element f, set r = l(f) and m = q(r)
    // GOTO A

    todo!()
}
