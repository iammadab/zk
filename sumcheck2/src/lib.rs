use ark_ff::{BigInteger, PrimeField};
use polynomial::product_poly::ProductPoly;
use transcript::Transcript;

// TODO: add documentation
pub struct SumcheckProof<F: PrimeField> {
    sum: F,
    round_polys: Vec<Vec<F>>,
}

// TODO: add documentation
pub struct SubClaim<F: PrimeField> {
    sum: F,
    challenges: Vec<F>,
}

// TODO: break up file
//  split into prover and verifier

// TODO: consider splitting into prover and verifier
//  also storing the max_var_degree in them, so we don't
//  have to pass it everywhere
pub struct Sumcheck {}

impl Sumcheck {
    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove<F: PrimeField + std::convert::From<usize>>(
        poly: ProductPoly<F>,
        max_variable_degree: usize,
        sum: F,
    ) -> Result<SumcheckProof<F>, &'static str> {
        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        Ok(Self::prove_internal(poly, sum, max_variable_degree, &mut transcript)?.0)
    }

    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove_partial<F: PrimeField + std::convert::From<usize>>(
        poly: ProductPoly<F>,
        max_variable_degree: usize,
        sum: F,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        let mut transcript = Transcript::new();
        Self::prove_internal(poly, sum, max_variable_degree, &mut transcript)
    }

    // TODO: add documentation
    // TODO: explain why we are passing the max variable degree
    pub fn prove_internal<F: PrimeField + std::convert::From<usize>>(
        mut poly: ProductPoly<F>,
        sum: F,
        max_variable_degree: usize,
        transcript: &mut Transcript,
    ) -> Result<(SumcheckProof<F>, Vec<F>), &'static str> {
        // TODO: comment this section
        let mut round_polys = vec![];
        let mut challenges = vec![];

        transcript.append(sum.into_bigint().to_bytes_be().as_slice());

        for _ in 0..poly.n_vars() {
            // calculate round_poly
            let mut round_poly = vec![];
            for i in 0..=max_variable_degree {
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

    // TODO: add documentation
    pub fn verify<F: PrimeField>(poly: ProductPoly<F>, proof: SumcheckProof<F>) -> bool {
        todo!()
    }

    // TODO: add documentation
    // TODO: explain return type
    pub fn verify_partial<F: PrimeField>(proof: SumcheckProof<F>) -> Option<SubClaim<F>> {
        todo!()
    }

    pub fn verify_internal<F: PrimeField>(
        proof: SumcheckProof<F>,
        transcript: &mut Transcript,
    ) -> Option<SubClaim<F>> {
        // main verification loop
        todo!()
    }
}
