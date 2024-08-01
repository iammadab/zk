use ark_ff::PrimeField;
use polynomial::product_poly::ProductPoly;
use transcript::Transcript;

// TODO: add documentation
pub struct SumcheckProof<F: PrimeField> {
    sum: F,
    round_poly: Vec<Vec<F>>,
}

fn prove<F: PrimeField>(poly: &ProductPoly<F>, claimed_sum: F) -> SumcheckProof<F> {
    // steps
    let transcript = Transcript::new();
    // transcript.append(poly.to_bytes());
    // transcript.append()
    todo!()
}
