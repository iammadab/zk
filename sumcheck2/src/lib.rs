pub mod prover;
pub mod verifier;

use ark_ff::PrimeField;

// TODO: add documentation
pub struct SumcheckProof<F: PrimeField> {
    sum: F,
    round_polys: Vec<Vec<F>>,
}

// TODO: add documentation
pub struct SubClaim<F: PrimeField> {
    sum: F,
    challenges: Vec<F>,
}
