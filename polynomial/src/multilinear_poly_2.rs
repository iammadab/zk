// TODO: delete old multilinear file

use ark_ff::PrimeField;

// TODO: add documentation
struct MultilinearPolynomial<F: PrimeField> {
    n_vars: u32,
    evaluations: Vec<F>,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    // TODO: add documentation
    fn new(n_vars: u32, evaluations: Vec<F>) -> Result<Self, &'static str> {
        // the evaluation vec length must exactly be equal to 2^n_vars
        // this is because we might not always be able to assume the appropriate
        // element to pad the vector with.
        if evaluations.len() != (1 << n_vars) {
            return Err("evaluation vec len should equal 2^n_vars");
        }

        Ok(Self {
            n_vars,
            evaluations,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly_2::MultilinearPolynomial;
    use ark_bls12_381::Fr;

    #[test]
    fn test_new_multilinear_poly() {
        // should not allow n_vars / evaluation count mismatch
        let poly = MultilinearPolynomial::new(2, vec![Fr::from(3), Fr::from(1), Fr::from(2)]);
        assert_eq!(poly.is_err(), true);
        let poly = MultilinearPolynomial::new(2, vec![Fr::from(3), Fr::from(1)]);
        assert_eq!(poly.is_err(), true);

        // correct inputs
        let poly = MultilinearPolynomial::new(1, vec![Fr::from(3), Fr::from(1)]);
        assert_eq!(poly.is_err(), false);
        let poly =
            MultilinearPolynomial::new(2, vec![Fr::from(3), Fr::from(1), Fr::from(2), Fr::from(5)]);
        assert_eq!(poly.is_err(), false);
    }
}
