use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::sumcheck::util::{
    partial_evaluation_points, skip_first_var_then_sum_over_boolean_hypercube,
};
use crate::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};
use std::ops::Add;

pub mod boolean_hypercube;
pub mod util;

#[derive(Debug)]
pub struct SumcheckProof<F: PrimeField, P: MultiLinearExtension<F>> {
    poly: P,
    sum: F,
    uni_polys: Vec<P>,
}

impl<F: PrimeField, P: MultiLinearExtension<F>> SumcheckProof<F, P> {
    fn new(poly: P, sum: F) -> Self {
        Self {
            poly,
            sum,
            uni_polys: vec![],
        }
    }
}

pub struct Sumcheck {}

impl Sumcheck {
    /// Generate a sum check proof given the poly and the claimed sum
    pub fn prove<F: PrimeField, P: MultiLinearExtension<F>>(poly: P, sum: F) -> SumcheckProof<F, P>
    where
        for<'a> &'a P: Add<Output = Result<P, &'static str>>,
    {
        let mut uni_polys = vec![];
        let mut challenges = vec![];
        let mut transcript = Transcript::new();

        // add the poly and sum to the transcript
        transcript.append(sum.into_bigint().to_bytes_be().as_slice());
        transcript.append(poly.to_bytes().as_slice());

        for _ in 0..poly.n_vars() {
            // partially evaluate the polynomial at the generated challenge points
            let challenge_assignments =
                partial_evaluation_points(poly.n_vars(), 0..challenges.len(), challenges.iter());
            let challenge_poly = poly
                .partial_evaluate(&challenge_assignments)
                .unwrap()
                .relabel();

            let uni_poly = skip_first_var_then_sum_over_boolean_hypercube(challenge_poly);
            transcript.append(uni_poly.to_bytes().as_slice());
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
    pub fn verify<F: PrimeField, P: MultiLinearExtension<F>>(proof: SumcheckProof<F, P>) -> bool
    where
        for<'a> &'a P: Add<Output = Result<P, &'static str>>,
    {
        if proof.uni_polys.len() != proof.poly.n_vars() {
            // number of round poly's should match total number of rounds
            return false;
        }

        let mut transcript = Transcript::new();
        let mut challenges = vec![];

        // add the poly and sum to the transcript
        transcript.append(proof.sum.into_bigint().to_bytes_be().as_slice());
        transcript.append(&proof.poly.to_bytes().as_slice());

        let mut claimed_sum = proof.sum;

        for poly in proof.uni_polys {
            // TODO: this evaluations should take a single value rather than a slice
            // TODO: also remove unwrap
            // assert that p(0) + p(1) = sum
            let p_0 = poly.evaluate(&[F::zero()]).unwrap();
            let p_1 = poly.evaluate(&[F::one()]).unwrap();

            if claimed_sum != (p_0 + p_1) {
                return false;
            }

            // add poly to transcript
            transcript.append(&poly.to_bytes().as_slice());

            // sample challenge and update claimed sum
            let challenge = transcript.sample_field_element::<F>();
            // TODO: should take a single value here rather than slice
            claimed_sum = poly.evaluate(&[challenge]).unwrap();
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
