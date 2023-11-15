use crate::multilinear_poly::MultiLinearPolynomial;
use ark_ff::PrimeField;

/// Sumcheck Prover
struct Prover<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    challenges: Vec<F>,
}

impl<F: PrimeField> Prover<F> {
    fn new(poly: MultiLinearPolynomial<F>) -> Self {
        // TODO: sum over the bolean hypercube
        Self {
            poly,
            sum: F::zero(),
            challenges: Vec::new(),
        }
    }
}

/// Sumcheck Verifier
struct Verifier<'a, F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    claimed_sum: &'a F,
    challenges: Vec<F>,
}

impl<'a, F: PrimeField> Verifier<'a, F> {
    fn new(poly: MultiLinearPolynomial<F>, claimed_sum: &'a F) -> Self {
        Self {
            poly,
            claimed_sum,
            challenges: Vec::new(),
        }
    }
}
