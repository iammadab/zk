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

    // TODO: add documentation
    // TODO: add reasoning behind decision to go this route
    // TODO: implement partial evaluation for random index values
    // this assumes you only want to partially evaluate the first variable
    fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        // we need an algo to do matching
        // then do linear interpolation on the left and right
        // finally reassign to the top part
        // return the truncated version

        let mut new_evaluations = self.evaluations.clone();
        let pairing_index = Self::compute_paring_index(self.n_vars, initial_var)?;

        // TODO: what checks do I have to make?

        for i in 0..assignments.len() {
            // how do we truncate the index, going to be based on the shift value
            // shift value changes with every iteration right??
            let current_evaluation_length = 1 << (self.n_vars - i);
            let shift_value = current_evaluation_length / (1 << (initial_var + 1));

            for j in 0..shift_value {
                // here we do the interpolation right
                let left = self.evaluations[pairing_index[j]];
                let right = self.evaluations[pairing_index[j] + shift_value];
                new_evaluations[j] = (F::ONE - assignments[i]) * left + assignments[i] * right;
            }
        }

        // TODO: truncate the new_evaluations length
        Ok(Self::new(self.n_vars - 1, new_evaluations)?)
    }

    // next thing is partial evaluation but for multiple variables
    // to do this I need a way to get the shift value
    // and a way to get the indexes
    // shift value = 2^n_vars / 2^i

    // TODO: add documentation and add assumptions made
    //  pairing var is 0 indexed
    fn compute_paring_index(n_vars: usize, pairing_var: usize) -> Result<Vec<usize>, &'static str> {
        // TODO: clean up

        if pairing_var >= n_vars {
            return Err("pairing variable must exist in the polynomial");
        }

        let evaluation_len = 1 << n_vars;

        let mut result = vec![];
        let shift_value = evaluation_len / (1 << (pairing_var + 1));

        // take shift value number of elements

        // we know how many elements we'd need
        let mut to_push = 0;

        for i in 0..(evaluation_len / 2) {
            result.push(to_push);
            to_push += 1;

            // what do we want here?
            // hmm, multiple?
            if (i + 1) % shift_value == 0 {
                to_push += shift_value;
            }
        }

        Ok(result)
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
            poly.partial_evaluate(0, &[Fr::from(5)])
                .unwrap()
                .evaluations,
            vec![Fr::from(-2), Fr::from(21)]
        );
    }

    #[test]
    fn test_compute_pairing_index() {
        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(3, 0).unwrap();
        assert_eq!(pairing_index, vec![0, 1, 2, 3]);

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(3, 1).unwrap();
        assert_eq!(pairing_index, vec![0, 1, 4, 5]);

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(3, 2).unwrap();
        assert_eq!(pairing_index, vec![0, 2, 4, 6]);

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(3, 3);
        assert!(pairing_index.is_err());

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(4, 0).unwrap();
        assert_eq!(pairing_index, vec![0, 1, 2, 3, 4, 5, 6, 7]);

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(4, 1).unwrap();
        assert_eq!(pairing_index, vec![0, 1, 2, 3, 8, 9, 10, 11]);

        let pairing_index = MultilinearPolynomial::<Fr>::compute_paring_index(4, 2).unwrap();
        assert_eq!(pairing_index, vec![0, 1, 4, 5, 8, 9, 12, 13]);
    }
}
