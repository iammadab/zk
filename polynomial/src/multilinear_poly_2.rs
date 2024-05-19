// TODO: delete old multilinear file

use ark_ff::PrimeField;

// TODO: add documentation
struct MultilinearPolynomial<F: PrimeField> {
    n_vars: u32,
    evaluations: Vec<F>,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    /// Instantiates a new `MultilinearPolynomial` after ensuring variable count
    /// aligns with evaluation len
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

    // TODO: implement partial evaluation for random index values
    // this assumes you only want to partially evaluate the first variable
    fn partial_evaluate(&self, value: F) -> Result<Self, &'static str> {
        // we need an algo to do matching
        // then do linear interpolation on the left and right
        // finally reassign to the top part
        // return the truncated version

        let shift_value = self.evaluations.len() / 2;
        // TODO: look into the uninitialized optimization (does it actually optimize anything?)
        let mut new_evaluations = vec![F::zero(); shift_value];

        // iterate from half the eval length
        for i in 0..shift_value {
            let left = self.evaluations[i];
            let right = self.evaluations[i + shift_value];
            // linear interpolation
            new_evaluations[i] = (F::ONE - value) * left + value * right;
        }

        Ok(Self::new(self.n_vars - 1, new_evaluations)?)
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

    #[test]
    fn test_partial_evaluate() {
        let poly =
            MultilinearPolynomial::new(2, vec![Fr::from(3), Fr::from(1), Fr::from(2), Fr::from(5)])
                .unwrap();
        assert_eq!(
            poly.partial_evaluate(Fr::from(5)).unwrap().evaluations,
            vec![Fr::from(-2), Fr::from(21)]
        );
    }
}
