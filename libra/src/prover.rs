use crate::circuit::layered_circuit::LayeredCircuit;
use ark_ff::PrimeField;
use polynomial::multilinear::evaluation_form::MultiLinearPolynomial;
use sumcheck::SumcheckProof;
use transcript::Transcript;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(circuit: &LayeredCircuit, evaluations: Vec<Vec<F>>) -> GKRProof<F> {
    // we need to prover layer by layer
    // each layer is supposed to generate a sumcheck proof
    // and also each layer should return hints

    // let us focus on the output layer now
    // we have the output vec we can turn that to an mle
    // first we need to put the public inputs to the transcript
    // what are the public inputs??
    // the circuit should be part but for now we skip
    // we need the output mle

    let mut transcript = Transcript::new();

    let output_mle = MultiLinearPolynomial::new()

    todo!()
}
