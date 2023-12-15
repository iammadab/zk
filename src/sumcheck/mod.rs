use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::sumcheck::util::{
    add_multilinear_poly_to_transcript, add_univariate_poly_to_transcript,
    partial_evaluation_points, skip_first_var_then_sum_over_boolean_hypercube,
};
use crate::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};

pub mod boolean_hypercube;
pub mod util;

#[derive(Debug)]
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

        for _ in 0..poly.n_vars() {
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

#[cfg(test)]
mod tests {
    use crate::transcript::Transcript;
    use sha3::digest::typenum::Sum;

    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::Sumcheck;
    use ark_ff::{Fp64, MontBackend, MontConfig, One};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    fn p_2ab_3bc() -> MultiLinearPolynomial<Fq> {
        MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_sumcheck_protocol_correct_sum() {
        // p = 2ab + 3bc
        // sum over boolean hypercube = 10
        let p = p_2ab_3bc();
        let sumcheck_proof = Sumcheck::prove(p, Fq::from(10));
        assert!(Sumcheck::verify(sumcheck_proof));
    }

    #[test]
    fn test_sumcheck_protocol_invalid_sum() {
        // p = 2ab + 3bc
        // sum over boolean hypercube = 10
        let p = p_2ab_3bc();
        let sumcheck_proof = Sumcheck::prove(p, Fq::from(200));
        assert!(!Sumcheck::verify(sumcheck_proof));
    }
}
