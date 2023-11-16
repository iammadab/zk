use crate::multilinear_poly::MultiLinearPolynomial;
use ark_ff::PrimeField;

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
