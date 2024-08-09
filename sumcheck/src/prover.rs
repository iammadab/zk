use crate::{field_elements_to_bytes, SumcheckProof};
use ark_ff::{BigInteger, PrimeField};
use polynomial::composed_poly::product_poly::ProductPoly;
use polynomial::composed_poly::ComposedPolynomial;
use std::marker::PhantomData;
use transcript::Transcript;

/// `SumcheckProver`, initialized with the max_var_degree of the polynomial
/// this is used to determine how many points to evaluate the round polynomials
pub struct SumcheckProver<const MAX_VAR_DEGREE: u8, F: PrimeField + std::convert::From<u8>> {
    _marker: PhantomData<F>,
}

impl<const MAX_VAR_DEGREE: u8, F: PrimeField + std::convert::From<u8>>
    SumcheckProver<MAX_VAR_DEGREE, F>
{
    /// Generates the `Sumcheck` proof (appends the initial poly to the transcript)
    pub fn prove(poly: ComposedPolynomial<F>, sum: F) -> Result<SumcheckProof<F>, &'static str> {
        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        Ok(Self::prove_internal(poly, sum, &mut transcript)?.0)
    }

    /// Generates the `Sumcheck` proof, but doesn't append the initial poly to the transcript.
    /// This is used when the verifier doesn't have access to the initial poly or its commitment
    pub fn prove_partial(
        poly: ComposedPolynomial<F>,
        sum: F,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        let mut transcript = Transcript::new();
        Self::prove_internal(poly, sum, &mut transcript)
    }

    /// Main `Sumcheck` proof generation logic.
    fn prove_internal(
        mut poly: ComposedPolynomial<F>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        let mut round_polys = vec![];
        let mut challenges = vec![];

        // append the sum to the transcript
        transcript.append(sum.into_bigint().to_bytes_be().as_slice());

        for _ in 0..poly.n_vars() {
            // calculate round_poly
            // for a round poly of a certain degree d (denoted by MAX_VAR_DEGREE)
            // we evaluate the polynomial at d + 1 points
            let mut round_poly = vec![];
            for i in 0..=MAX_VAR_DEGREE {
                round_poly.push(
                    poly.partial_evaluate(0, &[F::from(i)])?
                        .reduce()
                        .iter()
                        .sum::<F>(),
                )
            }

            // add round_poly to transcript
            transcript.append(field_elements_to_bytes(&round_poly).as_slice());

            // generate challenge
            let challenge = transcript.sample_field_element::<F>();
            // partially evaluate the poly at the challenge
            poly = poly.partial_evaluate(0, &[challenge])?;

            round_polys.push(round_poly);
            challenges.push(challenge);
        }

        let proof = SumcheckProof { sum, round_polys };

        Ok((proof, challenges))
    }
}
