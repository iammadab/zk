use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::sumcheck::util::{
    partial_evaluation_points, skip_first_var_then_sum_over_boolean_hypercube,
};
use crate::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::ops::Add;

pub mod boolean_hypercube;
pub mod util;

#[derive(Debug, Clone)]
pub struct SumcheckProof<F: PrimeField, P: MultiLinearExtension<F>> {
    poly: P,
    sum: F,
    uni_polys: Vec<UnivariatePolynomial<F>>,
}

impl<F: PrimeField, P: MultiLinearExtension<F>> SumcheckProof<F, P> {
    pub fn new(poly: P, sum: F) -> Self {
        Self {
            poly,
            sum,
            uni_polys: vec![],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.poly.to_bytes());
        result.extend(self.sum.into_bigint().to_bytes_be());
        for poly in &self.uni_polys {
            result.extend(poly.to_bytes());
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
/// Same as the sumcheck proof without the initial poly
pub struct PartialSumcheckProof<F: PrimeField> {
    pub(crate) sum: F,
    uni_polys: Vec<UnivariatePolynomial<F>>,
}

impl<F: PrimeField, P: MultiLinearExtension<F>> From<SumcheckProof<F, P>>
    for PartialSumcheckProof<F>
{
    fn from(value: SumcheckProof<F, P>) -> Self {
        Self {
            sum: value.sum,
            uni_polys: value.uni_polys,
        }
    }
}

impl<F: PrimeField> PartialSumcheckProof<F> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.sum.into_bigint().to_bytes_be());
        for poly in &self.uni_polys {
            result.extend(poly.to_bytes());
        }
        result
    }
}

#[derive(Debug, PartialEq)]
/// Represents the result of a partial sumcheck proof verification
pub struct SubClaim<F: PrimeField> {
    pub(crate) sum: F,
    pub(crate) challenges: Vec<F>,
}

pub struct Sumcheck {}

impl Sumcheck {
    /// Generate a sum check proof given the poly and the claimed sum
    pub fn prove<F: PrimeField, P: MultiLinearExtension<F>>(poly: P, sum: F) -> SumcheckProof<F, P>
    where
        for<'a> &'a P: Add<Output = Result<P, &'static str>>,
    {
        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        Self::prove_internal(poly, sum, &mut transcript).0
    }

    /// Generates a sumcheck proof that makes no statement about the initial poly
    pub fn prove_partial<F: PrimeField, P: MultiLinearExtension<F>>(
        poly: P,
        sum: F,
    ) -> (PartialSumcheckProof<F>, Vec<F>)
    where
        for<'a> &'a P: Add<Output = Result<P, &'static str>>,
    {
        let mut transcript = Transcript::new();
        let (proof, challenges) = Self::prove_internal(poly, sum, &mut transcript);
        (proof.into(), challenges)
    }

    fn prove_internal<F: PrimeField, P: MultiLinearExtension<F>>(
        poly: P,
        sum: F,
        transcript: &mut Transcript,
    ) -> (SumcheckProof<F, P>, Vec<F>)
    where
        for<'a> &'a P: Add<Output = Result<P, &'static str>>,
    {
        let mut uni_polys = vec![];
        let mut challenges = vec![];

        transcript.append(sum.into_bigint().to_bytes_be().as_slice());

        for _ in 0..poly.n_vars() {
            // partially evaluate the polynomial at the generated challenge points
            let challenge_assignments =
                partial_evaluation_points(poly.n_vars(), 0..challenges.len(), challenges.iter());
            let challenge_poly = poly
                .partial_evaluate(&challenge_assignments)
                .unwrap()
                .relabel();

            let uni_poly = skip_first_var_then_sum_over_boolean_hypercube(challenge_poly)
                .to_univariate()
                .unwrap();
            transcript.append(uni_poly.to_bytes().as_slice());
            uni_polys.push(uni_poly);

            // sample challenge
            challenges.push(transcript.sample_field_element::<F>());
        }

        (
            SumcheckProof {
                poly,
                sum,
                uni_polys,
            },
            challenges,
        )
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
        // add poly to transcript
        transcript.append(proof.poly.to_bytes().as_slice());

        let initial_poly = proof.poly.clone();

        if let Some(subclaim) = Self::verify_internal(proof.into(), &mut transcript) {
            // final verifier check
            // p_v(r_v) = p(r_1, r_2, ..., r_v)
            let initial_poly_eval = initial_poly
                .evaluate(subclaim.challenges.as_slice())
                .unwrap();
            initial_poly_eval == subclaim.sum
        } else {
            false
        }
    }

    /// Verify partial sumcheck proof
    pub fn verify_partial<F: PrimeField>(proof: PartialSumcheckProof<F>) -> Option<SubClaim<F>> {
        let mut transcript = Transcript::new();
        Self::verify_internal(proof, &mut transcript)
    }

    fn verify_internal<F: PrimeField>(
        proof: PartialSumcheckProof<F>,
        transcript: &mut Transcript,
    ) -> Option<SubClaim<F>> {
        let mut challenges = vec![];

        transcript.append(proof.sum.into_bigint().to_bytes_be().as_slice());

        let mut claimed_sum = proof.sum;

        for poly in proof.uni_polys {
            // assert that p(0) + p(1) = sum
            let p_0 = poly.evaluate(&F::zero());
            let p_1 = poly.evaluate(&F::one());

            if claimed_sum != (p_0 + p_1) {
                return None;
            }

            // add poly to transcript
            transcript.append(poly.to_bytes().as_slice());

            // sample challenge and update claimed sum
            let challenge = transcript.sample_field_element::<F>();
            claimed_sum = poly.evaluate(&challenge);
            challenges.push(challenge);
        }

        Some(SubClaim {
            sum: claimed_sum,
            challenges,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::Sumcheck;
    use ark_ff::{Fp64, MontBackend, MontConfig};

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

    #[test]
    fn test_partial_sumcheck() {
        // invalid sum
        let partial_proof = Sumcheck::prove_partial(p_2ab_3bc(), Fq::from(200));
        let verification_result = Sumcheck::verify_partial(partial_proof.0);
        assert_eq!(verification_result, None);

        // valid sum
        let partial_proof = Sumcheck::prove_partial(p_2ab_3bc(), Fq::from(10));
        let verification_result = Sumcheck::verify_partial(partial_proof.0);
        assert!(verification_result.is_some());
    }
}
