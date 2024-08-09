use crate::{field_elements_to_bytes, SubClaim, SumcheckProof};
use ark_ff::{BigInteger, PrimeField};
use polynomial::composed_poly::product_poly::ProductPoly;
use polynomial::univariate_poly::UnivariatePolynomial;
use std::marker::PhantomData;
use transcript::Transcript;

/// Sumcheck Verifier
pub struct SumcheckVerifier<F: PrimeField> {
    _marker: PhantomData<F>,
}

impl<F: PrimeField> SumcheckVerifier<F> {
    /// Verify a `Sumcheck` proof (verifier has access to the initial poly or its commitment)
    pub fn verify(poly: ProductPoly<F>, proof: SumcheckProof<F>) -> Result<bool, &'static str> {
        // number of round_poly in the proof should match n_vars
        if proof.round_polys.len() != poly.n_vars() {
            return Err("invalid proof: require 1 round poly for each variable in poly");
        }

        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        let subclaim = Self::verify_internal(proof, &mut transcript)?;

        // final verifier check
        // p_v(r_v) = p(r_1, r_2, ..., r_v)
        let initial_poly_eval = poly
            .evaluate(subclaim.challenges.as_slice())
            .map_err(|_| "couldn't evaluate initial poly")?;
        // ensure the oracle evaluation equals the claimed sum
        Ok(initial_poly_eval == subclaim.sum)
    }

    /// Verify a `Sumcheck` proof (when the veifier doesn't have access to the initial poly or its commitment)
    /// in such a case, the verifier performs all checks other than the last check.
    /// Returns a subclaim that can later be used for that final check verification.
    pub fn verify_partial(proof: SumcheckProof<F>) -> Result<SubClaim<F>, &'static str> {
        let mut transcript = Transcript::new();
        Self::verify_internal(proof, &mut transcript)
    }

    /// Main `Sumcheck` verification logic.
    fn verify_internal(
        proof: SumcheckProof<F>,
        transcript: &mut Transcript,
    ) -> Result<SubClaim<F>, &'static str> {
        let mut challenges = vec![];

        transcript.append(proof.sum.into_bigint().to_bytes_be().as_slice());

        let mut claimed_sum = proof.sum;

        for round_poly in proof.round_polys {
            // append the round poly to the transcript
            transcript.append(field_elements_to_bytes(&round_poly).as_slice());

            let round_univariate_poly = UnivariatePolynomial::interpolate(round_poly);

            // assert that p(0) + p(1) = sum
            let p_0 = round_univariate_poly.evaluate(&F::ZERO);
            let p_1 = round_univariate_poly.evaluate(&F::ONE);

            if claimed_sum != (p_0 + p_1) {
                return Err("verifier check failed: claimed_sum != p(0) + p(1)");
            }

            // sample challenge and update claimed sum
            let challenge = transcript.sample_field_element::<F>();
            claimed_sum = round_univariate_poly.evaluate(&challenge);
            challenges.push(challenge);
        }

        Ok(SubClaim {
            sum: claimed_sum,
            challenges,
        })
    }
}
