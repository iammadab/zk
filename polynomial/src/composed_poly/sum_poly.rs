use crate::composed_poly::ComposedPolynomial;
use ark_ff::PrimeField;

// TODO: we can merge product poly and sum poly into one polynomial

#[derive(Clone, Debug, PartialEq)]
/// Represents the sum of one or more `Composed` polynomials
/// P(x) = A(x) + B(x) + ...  + N(x)
pub struct SumPoly<F: PrimeField> {
    n_vars: usize,
    polynomials: Vec<ComposedPolynomial<F>>,
}

impl<F: PrimeField> SumPoly<F> {
    /// Instantiate a new sum_poly from a set of `Composed` polynomials
    pub fn new(polynomials: Vec<ComposedPolynomial<F>>) -> Result<Self, &'static str> {
        if polynomials.len() == 0 {
            return Err("cannot create sum polynomial from empty polynomials");
        }

        // ensure that all polynomials share the same number of variables
        let expected_num_of_vars = polynomials[0].n_vars();
        let equal_variables = polynomials
            .iter()
            .all(|poly| poly.n_vars() == expected_num_of_vars);
        if !equal_variables {
            return Err("cannot create sum polynomial from polynomial that don't share the same number of variables");
        }

        Ok(Self {
            n_vars: expected_num_of_vars,
            polynomials,
        })
    }

    /// Evaluate the sum poly using the following
    /// P(x) = A(x) + B(x) + ...  + N(x)
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars {
            return Err("evaluate must assign to all variables");
        }

        self.polynomials.iter().try_fold(F::zero(), |sum, poly| {
            poly.evaluate(assignments).map(|value| sum + value)
        })
    }

    /// Partially evaluate each component polynomial on the same input, returns a new sum_poly
    /// with the partial polynomials
    pub fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        let partial_polynomials = self
            .polynomials
            .iter()
            .map(|polynomial| polynomial.partial_evaluate(initial_var, assignments))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            n_vars: partial_polynomials[0].n_vars(),
            polynomials: partial_polynomials,
        })
    }

    /// Converts the internal polynomials to evaluations and returns their element wise summation
    pub fn sum_reduce(&self) -> Vec<F> {
        let mut result = self.polynomials[0].reduce();
        for polynomial in self.polynomials.iter().skip(1) {
            for (i, eval) in polynomial.reduce().iter().enumerate() {
                result[i] += eval
            }
        }
        result
    }

    /// Serialize the SumPoly
    pub fn to_bytes(&self) -> Vec<u8> {
        self.polynomials
            .iter()
            .map(|poly| poly.to_bytes())
            .collect::<Vec<Vec<u8>>>()
            .concat()
    }

    /// Return the number of variables
    pub fn n_vars(&self) -> usize {
        self.n_vars
    }

    /// Return the max variable degree
    pub fn max_variable_degree(&self) -> usize {
        // the max variable degree for a sum poly is the largest
        // max variable degree of it's components
        // e.g. (2a^2b) + (3ab)
        // will have degree 2 i.e max(2, 1)
        self.polynomials
            .iter()
            .map(|poly| poly.max_variable_degree())
            .max()
            .expect("guaranteed we cannot create a sum poly with empty elements")
    }
}

#[cfg(test)]
mod tests {
    use crate::composed_poly::sum_poly::SumPoly;
    use crate::multilinear::coefficient_form::CoeffMultilinearPolynomial;
    use crate::multilinear::evaluation_form::MultiLinearPolynomial;
    use ark_bls12_381::Fr;

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

    #[should_panic]
    #[test]
    fn test_cannot_create_empty_sum_poly() {
        SumPoly::<Fr>::new(vec![]).unwrap();
    }

    #[test]
    fn test_sum_poly_creation() {
        SumPoly::new(vec![p_2ab_3bc().into(), p_2ab_3bc().into()]).unwrap();
    }

    #[test]
    fn test_evaluate() {
        // (2ab + 3bc) * 3 = 6ab + 9bc
        let sum_poly = SumPoly::new(vec![
            p_2ab_3bc().into(),
            p_2ab_3bc().into(),
            p_2ab_3bc().into(),
        ])
        .unwrap();
        // eval at (3, 4, 5)
        // 6(3)(4) + 9(4)(5) = 72 + 180 = 252
        assert_eq!(
            sum_poly
                .evaluate(&[Fr::from(3), Fr::from(4), Fr::from(5)])
                .unwrap(),
            Fr::from(252)
        );
    }

    #[test]
    fn test_partial_evaluate() {
        let sum_poly = SumPoly::new(vec![p_2ab_3bc().into(), p_2ab_3bc().into()]).unwrap();
        let partial_eval = sum_poly.partial_evaluate(1, &[Fr::from(10)]).unwrap();
        let partial_component = p_2ab_3bc().partial_evaluate(1, &[Fr::from(10)]).unwrap();
        let expected_partial_eval = SumPoly::new(vec![
            partial_component.clone().into(),
            partial_component.into(),
        ])
        .unwrap();
        assert_eq!(partial_eval, expected_partial_eval);
    }

    #[test]
    fn test_sum_reduce() {
        let mle_a = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(14)],
        )
        .unwrap();
        let mle_b = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(22)],
        )
        .unwrap();
        let sum_poly = SumPoly::new(vec![mle_a.into(), mle_b.into()]).unwrap();

        assert_eq!(
            sum_poly.sum_reduce(),
            vec![Fr::from(4), Fr::from(16), Fr::from(20), Fr::from(36)]
        );
    }

    #[test]
    fn test_max_variable_degree() {
        // p = 2ab + 3bc
        // P = p + p + p
        // expected degree = 1

        let sum_poly = SumPoly::new(vec![
            p_2ab_3bc().into(),
            p_2ab_3bc().into(),
            p_2ab_3bc().into(),
        ])
        .unwrap();
        assert_eq!(sum_poly.max_variable_degree(), 1);
    }

    // #[test]
    // fn test_flatten() {
    //     let sum_1 = SumPoly::new(vec![p_2ab_3bc().into(), p_2ab_3bc().into()]).unwrap();
    //     let sum_2 = SumPoly::new(vec![p_2ab_3bc().into(), sum_1.into()]).unwrap();
    //     assert_eq!(sum_2.polynomials.len(), 3);
    // }
}
