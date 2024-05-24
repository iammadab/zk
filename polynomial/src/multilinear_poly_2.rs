// TODO: delete old multilinear file

use crate::pairing_index::PairingIndex;
use ark_ff::PrimeField;

// TODO: add documentation
struct MultilinearPolynomial<F: PrimeField> {
    n_vars: usize,
    evaluations: Vec<F>,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    /// Instantiates a new `MultilinearPolynomial` after ensuring variable count
    /// aligns with evaluation len
    fn new(n_vars: usize, evaluations: Vec<F>) -> Result<Self, &'static str> {
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

    // TODO: add documentation
    // TODO: add reasoning behind decision to go this route
    fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        let mut new_evaluations = self.evaluations.clone();

        // for each assignment, get the pairing index
        for (i, assignment) in assignments.iter().enumerate() {
            let pairing_iterator = PairingIndex::new(self.n_vars - i, initial_var)?;
            let shift_value = pairing_iterator.shift_value();
            for (i, index) in pairing_iterator.enumerate() {
                let left = new_evaluations[index];
                let right = new_evaluations[index + shift_value];
                // linear interpolation
                new_evaluations[i] = ((F::ONE - assignment) * left) + (*assignment * right);
            }
        }

        // truncate and return new polynomial
        let new_n_vars = self.n_vars - assignments.len();
        Ok(Self::new(
            new_n_vars,
            new_evaluations[..(1 << new_n_vars)].to_vec(),
        )?)
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
    fn test_partial_evaluate_single_variable() {
        let poly =
            MultilinearPolynomial::new(2, vec![Fr::from(3), Fr::from(1), Fr::from(2), Fr::from(5)])
                .unwrap();
        assert_eq!(
            poly.partial_evaluate(0, &[Fr::from(5)])
                .unwrap()
                .evaluations,
            vec![Fr::from(-2), Fr::from(21)]
        );
    }

    #[test]
    fn test_partial_evaluate_consecutive_variables() {
        // f(a, b, c) = 2ab + 3bc
        let poly = MultilinearPolynomial::new(
            3,
            vec![
                Fr::from(0),
                Fr::from(0),
                Fr::from(0),
                Fr::from(3),
                Fr::from(0),
                Fr::from(0),
                Fr::from(2),
                Fr::from(5),
            ],
        )
        .unwrap();

        let f_of_a_evaluations = poly
            .partial_evaluate(1, &[Fr::from(2), Fr::from(3)])
            .unwrap()
            .evaluations;
        assert_eq!(f_of_a_evaluations.len(), 2);
        assert_eq!(f_of_a_evaluations, &[Fr::from(18), Fr::from(22)]);
    }
}
