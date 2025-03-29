use crate::{circuit::layered_circuit::LayeredCircuit, util::w_i};
use ark_ff::PrimeField;
use sumcheck::SumcheckProof;
use transcript::Transcript;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(circuit: &LayeredCircuit, evaluations: Vec<Vec<F>>) -> GKRProof<F> {
    let mut transcript = Transcript::new();

    // add public input to the transcript
    // TODO: add the circuit also
    let output_mle = w_i(&evaluations, 0);
    transcript.append(output_mle.to_bytes().as_slice());

    let r_0 = transcript.sample_n_field_elements(output_mle.n_vars());
    let m_0 = output_mle.evaluate(&r_0);

    // m_0 will serve as the claimed sum
    // r_0 will serve as g
    // what next??
    // we need to perform sumcheck for a particular layer
    // the layer 0 to layer 1 connection

    todo!()
}

fn libra_sumcheck<F: PrimeField>(layer_id: usize) -> SumcheckProof<F> {
    todo!()
}
