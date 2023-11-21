use crate::multilinear_poly::MultiLinearPolynomial;
use crate::sumcheck::prover::{
    partial_evaluation_points, skip_first_var_then_sum_over_boolean_hypercube, Prover,
};
use crate::transcript::Transcript;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::{BigInteger, PrimeField};
use std::ops::Mul;

pub mod boolean_hypercube;
mod prover;
mod verifier;

struct SumcheckProof<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    uni_polys: Vec<UnivariatePolynomial<F>>,
}

impl<F: PrimeField> SumcheckProof<F> {
    fn new(poly: MultiLinearPolynomial<F>, sum: F) -> Self {
        Self {
            poly,
            sum,
            uni_polys: vec![],
        }
    }
}

struct Sumcheck {}

impl Sumcheck {
    /// Generate a sum check proof given the poly and the claimed sum
    fn prove<F: PrimeField>(poly: MultiLinearPolynomial<F>, sum: F) -> SumcheckProof<F> {
        let mut uni_polys = vec![];
        let mut challenges = vec![];
        let mut transcript = Transcript::new();

        // add the poly and sum to the transcript
        transcript.append(sum.into_bigint().to_bytes_be().as_slice());
        add_multilinear_poly_to_transcript(&poly, &mut transcript);

        for i in 0..poly.n_vars() {
            // partially evaluate the polynomial at the generated challenge points
            let challenge_assignments =
                partial_evaluation_points(poly.n_vars(), 0..challenges.len(), challenges.iter());
            let challenge_poly = poly
                .partial_evaluate(&challenge_assignments)
                .unwrap()
                .relabel();

            let uni_poly = skip_first_var_then_sum_over_boolean_hypercube(&challenge_poly);
            add_univariate_poly_to_transcript(&uni_poly, &mut transcript);
            uni_polys.push(uni_poly);

            // sample challenge
            challenges.push(transcript.sample_field_element::<F>());
        }

        SumcheckProof {
            poly,
            sum,
            uni_polys,
        }
    }

    /// Verify a sumcheck proof
    fn verify<F: PrimeField>(proof: SumcheckProof<F>) -> bool {
        if proof.uni_polys.len() != proof.poly.n_vars() {
            // number of round poly's should match total number of rounds
            return false;
        }

        let mut transcript = Transcript::new();
        let mut challenges = vec![];

        // add the poly and sum to the transcript
        transcript.append(proof.sum.into_bigint().to_bytes_be().as_slice());
        add_multilinear_poly_to_transcript(&proof.poly, &mut transcript);

        let mut claimed_sum = proof.sum;

        for poly in proof.uni_polys {
            // assert that p(0) + p(1) = sum
            let p_0 = poly.evaluate(&F::zero());
            let p_1 = poly.evaluate(&F::one());

            if claimed_sum != (p_0 + p_1) {
                return false;
            }

            // add poly to transcript
            add_univariate_poly_to_transcript(&poly, &mut transcript);

            // sample challenge and update claimed sum
            let challenge = transcript.sample_field_element::<F>();
            claimed_sum = poly.evaluate(&challenge);
            challenges.push(challenge);
        }

        // final verifier check
        // p_v(r_v) = p(r_1, r_2, ..., r_v)
        let initial_poly_eval = proof.poly.evaluate(challenges.as_slice()).unwrap();
        initial_poly_eval == claimed_sum
    }
}

/// Add a multilinear polynomial to a transcript object
fn add_multilinear_poly_to_transcript<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
    transcript: &mut Transcript,
) {
    transcript.append(&poly.n_vars().to_be_bytes());
    for (var_id, coeff) in poly.coefficients() {
        transcript.append(&var_id.to_be_bytes());
        transcript.append(&coeff.into_bigint().to_bytes_be().as_slice());
    }
}

/// Add a univariate polynomial to a transcript object
fn add_univariate_poly_to_transcript<F: PrimeField>(
    poly: &UnivariatePolynomial<F>,
    transcript: &mut Transcript,
) {
    for coeff in poly.coefficients() {
        transcript.append(coeff.into_bigint().to_bytes_be().as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::add_multilinear_poly_to_transcript;
    use crate::transcript::Transcript;
    use ark_ff::{Fp64, MontBackend, MontConfig, One};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_add_poly_to_transcript() {
        let mut transcript = Transcript::new();
        let poly = MultiLinearPolynomial::<Fq>::additive_identity();
        add_multilinear_poly_to_transcript(&poly, &mut transcript);
        assert_eq!(transcript.sample_field_element::<Fq>(), Fq::one());
    }
}
