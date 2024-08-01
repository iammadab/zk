use ark_ff::PrimeField;
use polynomial::product_poly::ProductPoly;
use transcript::Transcript;

// TODO: add documentation
pub struct SumcheckProof<F: PrimeField> {
    sum: F,
    round_poly: Vec<Vec<F>>,
}

// TODO: add documentation
pub struct SubClaim<F: PrimeField> {
    sum: F,
    challenged: Vec<F>,
}

pub struct Sumcheck {}

impl Sumcheck {
    // TODO: add documentation
    pub fn prove<F: PrimeField>(poly: ProductPoly<F>, sum: F) -> SumcheckProof<F> {
        let mut transcript = Transcript::new();
        transcript.append(poly.to_bytes().as_slice());

        Self::prove_internal(poly, sum, &mut transcript).0
    }

    // TODO: add documentation
    pub fn prove_partial<F: PrimeField>(
        poly: ProductPoly<F>,
        sum: F,
    ) -> (SumcheckProof<F>, Vec<F>) {
        let mut transcript = Transcript::new();
        Self::prove_internal(poly, sum, &mut transcript)
    }

    // TODO: add documentation
    pub fn prove_internal<F: PrimeField>(
        poly: ProductPoly<F>,
        sum: F,
        transcript: &mut Transcript,
    ) -> (SumcheckProof<F>, Vec<F>) {
        // main sumcheck loop
        todo!()
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
