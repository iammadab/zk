use crate::SumcheckProof;
use ark_ff::{BigInteger, PrimeField};
use polynomial::product_poly::ProductPoly;
use std::marker::PhantomData;
use transcript::Transcript;

// TODO: add documentation
pub struct SumcheckProver<const MAX_VAR_DEGREE: usize, F: PrimeField + std::convert::From<usize>> {
    _marker: PhantomData<F>,
}

impl<const MAX_VAR_DEGREE: usize, F: PrimeField + std::convert::From<usize>>
    SumcheckProver<MAX_VAR_DEGREE, F>
{
    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove(poly: ProductPoly<F>, sum: F) -> Result<SumcheckProof<F>, &'static str> {
        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        Ok(Self::prove_internal(poly, sum, &mut transcript)?.0)
    }

    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove_partial(
        poly: ProductPoly<F>,
        sum: F,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        let mut transcript = Transcript::new();
        Self::prove_internal(poly, sum, &mut transcript)
    }

    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove_internal(
        mut poly: ProductPoly<F>,
        sum: F,
        transcript: &mut Transcript,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        // TODO: comment this section
        let mut round_polys = vec![];
        let mut challenges = vec![];

        transcript.append(sum.into_bigint().to_bytes_be().as_slice());

        for _ in 0..poly.n_vars() {
            // calculate round_poly
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
            transcript.append(
                round_poly
                    .iter()
                    .map(|elem| elem.into_bigint().to_bytes_be())
                    .collect::<Vec<Vec<u8>>>()
                    .concat()
                    .as_slice(),
            );

            // generate challenge
            let challenge = transcript.sample_field_element::<F>();

            // partially evaluate the poly at the challenge
            poly = poly.partial_evaluate(0, &[challenge])?;

            round_polys.push(round_poly)
        }

        let proof = SumcheckProof { sum, round_polys };

        Ok((proof, challenges))
    }
}
