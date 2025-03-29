use ark_ff::PrimeField;
use sumcheck::SumcheckProof;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}
