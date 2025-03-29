use crate::{field_elements_to_bytes, SumcheckProof};
use ark_ff::{BigInteger, PrimeField};
use polynomial::product_poly::ProductPoly;
use std::marker::PhantomData;
use transcript::Transcript;

/// `SumcheckProver`, initialized with the max_var_degree of the polynomial
/// this is used to determine how many points to evaluate the round polynomials
pub struct SumcheckProver<const MAX_VAR_DEGREE: u8, F: PrimeField> {
    _marker: PhantomData<F>,
}

impl<const MAX_VAR_DEGREE: u8, F: PrimeField> SumcheckProver<MAX_VAR_DEGREE, F> {
    /// Generates the `Sumcheck` proof (appends the initial poly to the transcript)
    pub fn prove(
        poly: ProductPoly<F>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<SumcheckProof<F>, &'static str> {
        transcript.append(poly.to_bytes().as_slice());

        Ok(Self::prove_internal(poly, sum, transcript)?.0)
    }

    /// Generates the `Sumcheck` proof, but doesn't append the initial poly to the transcript.
    /// This is used when the verifier doesn't have access to the initial poly or its commitment
    pub fn prove_partial(
        poly: ProductPoly<F>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        Self::prove_internal(poly, sum, transcript)
    }

    /// Main `Sumcheck` proof generation logic.
    fn prove_internal(
        mut poly: ProductPoly<F>,
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
                        .prod_reduce()
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

    fn prove_sum_of_products(
        polys: Vec<ProductPoly<F>>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<SumcheckProof<F>, &'static str> {
        Ok(Self::prove_sum_of_products_internal(polys, sum, transcript)?.0)
    }

    fn prove_sum_of_products_internal(
        // TODO: the assumption is that the all the product poly's have the same n_vars
        //  enforce this on a type level
        mut polys: Vec<ProductPoly<F>>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        let mut final_round_polys = vec![];
        let mut challenges = vec![];

        // append the sum to the transcript
        transcript.append(sum.into_bigint().to_bytes_be().as_slice());

        // what do we do here????
        // we want to run the same process for every intermediate polynomial
        // and then somehow merge the results into one
        for _ in 0..polys[0].n_vars() {
            let mut round_poly = vec![];
            // what do we do here
            // we iterate over each of the internal product polynomials
            for poly in polys.iter() {
                // for each poly we want to use that to generate the round poly
                let mut inner_round_poly = vec![];
                for i in 0..=MAX_VAR_DEGREE {
                    inner_round_poly.push(
                        poly.partial_evaluate(0, &[F::from(i)])?
                            .prod_reduce()
                            .iter()
                            .sum::<F>(),
                    )
                }

                // now I have the inner round poly, what do I do with that??
                // this is not the round poly that we push to the transctip
                round_poly.push(inner_round_poly);
            }

            let round_poly = element_wise_add_all(&round_poly);

            // add round_poly to the transcript
            transcript.append(field_elements_to_bytes(&round_poly).as_slice());

            // generate the challenge
            let challenge = transcript.sample_field_element::<F>();

            // partially evaluate all the polynomials at the challenge
            for poly in polys.iter_mut() {
                // TODO: implement some  kind of partial_eval in_place
                *poly = poly.partial_evaluate(0, &[challenge])?;
            }

            final_round_polys.push(round_poly);
            challenges.push(challenge);
        }

        let proof = SumcheckProof {
            sum,
            round_polys: final_round_polys,
        };

        Ok((proof, challenges))
    }
}

// TODO: move to util
fn element_wise_add_all<F: PrimeField>(vectors: &[Vec<F>]) -> Vec<F> {
    todo!()
}
