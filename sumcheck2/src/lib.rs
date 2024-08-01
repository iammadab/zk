pub mod prover;
pub mod verifier;

use ark_ff::{BigInteger, PrimeField};

#[derive(Debug)]
/// Holds the round polys and the initial prover claimed sum for sumcheck
pub struct SumcheckProof<F: PrimeField> {
    sum: F,
    round_polys: Vec<Vec<F>>,
}

/// Sometimes the verifier doesn't want to perform the final check
/// in such cases, a subclaim is returned, this subclaim has all information
/// needed to verify the last check
/// sum = initial_poly(challenges)
pub struct SubClaim<F: PrimeField> {
    sum: F,
    challenges: Vec<F>,
}

/// Helper method for converting field elements to bytes
fn field_elements_to_bytes<F: PrimeField>(field_elements: &[F]) -> Vec<u8> {
    field_elements
        .iter()
        .map(|elem| elem.into_bigint().to_bytes_be())
        .collect::<Vec<Vec<u8>>>()
        .concat()
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
    fn test_sumcheck_correct_sum_multilinear() {
        // p = 2ab + 3bc
        let p = p_2ab_3bc();
        let prod_poly = ProductPoly::new(vec![p]).unwrap();
        let proof = SumcheckProver::<1, Fr>::prove(prod_poly.clone(), Fr::from(10)).unwrap();
        let verification_result =
            SumcheckVerifier::verify(prod_poly, proof).expect("proof is invalid");
        assert!(verification_result);
    }

    #[test]
    fn test_correct_sum_multivariate_deg_2() {
        // p = 2a^2b + 3ab
        // decomposed into
        // p1 = (2a + 0b + 3)
        // p2 = (ab)
        // p = p1 . p2

        // p1 = (2a + 0b + 3)
        let p1 = MultiLinearPolynomial::new(
            2,
            CoeffMultilinearPolynomial::new(
                2,
                vec![
                    (Fr::from(2), vec![true, false]),
                    (Fr::from(0), vec![false, true]),
                    (Fr::from(3), vec![false, false]),
                ],
            )
            .unwrap()
            .to_evaluation_form(),
        )
        .unwrap();

        // p2 = (ab)
        let p2 = MultiLinearPolynomial::new(
            2,
            CoeffMultilinearPolynomial::new(2, vec![(Fr::from(1), vec![true, true])])
                .unwrap()
                .to_evaluation_form(),
        )
        .unwrap();

        let p = ProductPoly::new(vec![p1, p2]).unwrap();

        let proof = SumcheckProver::<2, Fr>::prove(p.clone(), Fr::from(5)).unwrap();
        let verification_result = SumcheckVerifier::verify(p, proof).expect("proof is invalid");
        assert!(verification_result);
    }

    #[test]
    fn test_correct_sum_prove_partial() {
        let p = p_2ab_3bc();
        let prod_poly = ProductPoly::new(vec![p]).unwrap();
        let (proof, _) =
            SumcheckProver::<1, Fr>::prove_partial(prod_poly.clone(), Fr::from(10)).unwrap();
        let subclaim = SumcheckVerifier::verify_partial(proof).expect("proof is invalid");
        let expected_sum = prod_poly.evaluate(subclaim.challenges.as_slice()).unwrap();
        assert_eq!(expected_sum, subclaim.sum);
    }

    #[test]
    fn test_invalid_sum() {
        // p = 2ab + 3bc
        let p = p_2ab_3bc();
        let prod_poly = ProductPoly::new(vec![p]).unwrap();
        let proof = SumcheckProver::<1, Fr>::prove(prod_poly.clone(), Fr::from(12)).unwrap();
        assert!(SumcheckVerifier::verify(prod_poly, proof).is_err());
    }
}
