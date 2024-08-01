pub mod prover;
pub mod verifier;

use ark_ff::PrimeField;

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

#[cfg(test)]
mod tests {
    use crate::prover::SumcheckProver;
    use crate::verifier::SumcheckVerifier;
    use ark_bls12_381::Fr;
    use polynomial::multilinear::coefficient_form::CoeffMultilinearPolynomial;
    use polynomial::multilinear::evaluation_form::MultiLinearPolynomial;
    use polynomial::product_poly::ProductPoly;

    fn p_2ab_3bc() -> MultiLinearPolynomial<Fr> {
        let evaluations = CoeffMultilinearPolynomial::new(
            3,
            vec![
                (Fr::from(2), vec![true, true, false]),
                (Fr::from(3), vec![false, true, true]),
            ],
        )
        .unwrap()
        .to_evaluation_form();
        MultiLinearPolynomial::new(3, evaluations).unwrap()
    }

    #[test]
    fn test_sumcheck_correct_sum() {
        // non-multivariate case
        let p = p_2ab_3bc();
        let prod_poly = ProductPoly::new(vec![p]).unwrap();
        let proof = SumcheckProver::<1, Fr>::prove(prod_poly.clone(), Fr::from(10)).unwrap();
        let verification_result =
            SumcheckVerifier::verify(prod_poly, proof).expect("proof is invalid");
        assert!(verification_result);

        // TODO: test multivariate case
    }
}
