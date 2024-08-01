use crate::{SubClaim, SumcheckProof};
use ark_ff::PrimeField;
use polynomial::product_poly::ProductPoly;
use std::marker::PhantomData;
use transcript::Transcript;

pub struct SumcheckVerifier<F: PrimeField> {
    _marker: PhantomData<F>,
}

impl<F: PrimeField> SumcheckVerifier<F> {
    // TODO: add documentation
    pub fn verify(poly: ProductPoly<F>, proof: SumcheckProof<F>) -> bool {
        todo!()
    }

    // TODO: add documentation
    // TODO: explain return type
    pub fn verify_partial(proof: SumcheckProof<F>) -> Option<SubClaim<F>> {
        todo!()
    }

    pub fn verify_internal(
        proof: SumcheckProof<F>,
        transcript: &mut Transcript,
    ) -> Option<SubClaim<F>> {
        // main verification loop
        todo!()
    }
}
