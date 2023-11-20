use crate::multilinear_poly::MultiLinearPolynomial;
use crate::univariate_poly::UnivariatePolynomial;

pub mod boolean_hypercube;
mod prover;
mod verifier;

struct SumcheckProof<F> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    uni_polys: Vec<UnivariatePolynomial<F>>,
}

struct Sumcheck {}

impl Sumcheck {
    fn prove<F>(poly: MultiLinearPolynomial<F>, sum: F) -> SumcheckProof<F> {
        todo!()
    }

    fn verify<F>(proof: SumcheckProof<F>) -> bool {
        todo!()
    }
}
