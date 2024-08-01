use crate::{SubClaim, SumcheckProof};
use ark_ff::{BigInteger, PrimeField};
use polynomial::product_poly::ProductPoly;
use polynomial::univariate_poly::UnivariatePolynomial;
use polynomial::Polynomial;
use std::marker::PhantomData;
use transcript::Transcript;

pub struct SumcheckVerifier<F: PrimeField> {
    _marker: PhantomData<F>,
}

impl<F: PrimeField> SumcheckVerifier<F> {
    // TODO: add documentation
    pub fn verify(poly: ProductPoly<F>, proof: SumcheckProof<F>) -> Result<bool, &'static str> {
        // number of round_poly in the proof should match n_vars
        if proof.round_polys.len() != poly.n_vars() {
            return Err("invalid proof: require 1 round poly for each variable in poly");
        }

        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        // TODO: is this clone needed?
        let initial_poly = poly.clone();

        let subclaim = Self::verify_internal(poly, proof, &mut transcript)?;

        // final verifier check
        // p_v(r_v) = p(r_1, r_2, ..., r_v)
        let initial_poly_eval = initial_poly
            .evaluate(subclaim.challenges.as_slice())
            .map_err(|_| "couldn't evaluate initial poly")?;
        // ensure the oracle evaluation equals the claimed sum
        Ok(initial_poly_eval == subclaim.sum)
    }

    // TODO: add documentation
    // TODO: explain return type
    pub fn verify_partial(
        poly: ProductPoly<F>,
        proof: SumcheckProof<F>,
    ) -> Result<SubClaim<F>, &'static str> {
        let mut transcript = Transcript::new();
        Self::verify_internal(poly, proof, &mut transcript)
    }

    pub fn verify_internal(
        poly: ProductPoly<F>,
        proof: SumcheckProof<F>,
        transcript: &mut Transcript,
    ) -> Result<SubClaim<F>, &'static str> {
        // TODO: document section
        let mut challenges = vec![];

        transcript.append(proof.sum.into_bigint().to_bytes_be().as_slice());

        let mut claimed_sum = proof.sum;

        for round_poly in proof.round_polys {
            // TODO: abstract this functionality
            // append the round poly to the transcript
            transcript.append(
                round_poly
                    .iter()
                    .map(|elem| elem.into_bigint().to_bytes_be())
                    .collect::<Vec<Vec<u8>>>()
                    .concat()
                    .as_slice(),
            );

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
