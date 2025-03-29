use crate::circuit::layered_circuit::LayeredCircuit;
use ark_ff::PrimeField;
use sumcheck::SumcheckProof;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(circuit: &LayeredCircuit, evaluations: Vec<Vec<F>>) -> GKRProof<F> {
    todo!()
}
