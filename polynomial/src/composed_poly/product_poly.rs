use crate::composed_poly::ComposedPolynomial;
use ark_ff::PrimeField;

/// Represents the product of one or more `Composed` polynomials
/// P(x) = A(x) x B(x) x ... x N(x)
#[derive(Clone, Debug, PartialEq)]
pub struct ProductPoly<F: PrimeField> {
    n_vars: usize,
    polynomials: Vec<ComposedPolynomial<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    /// Instantiate a new product_poly from a set of `Composed` polynomials
    pub fn new(polynomials: Vec<ComposedPolynomial<F>>) -> Result<Self, &'static str> {
        if polynomials.len() == 0 {
            return Err("cannot create product polynomial from empty polynomials");
        }

        // ensure that all polynomials share the same number of variables
        let expected_num_of_vars = polynomials[0].n_vars();
        let equal_variables = polynomials
            .iter()
            .all(|poly| poly.n_vars() == expected_num_of_vars);
        if !equal_variables {
            return Err("cannot create product polynomial from polynomial that don't share the same number of variables");
        }

        Ok(Self {
            n_vars: expected_num_of_vars,
            polynomials,
        })
    }

    /// Evaluate the product poly using the following
    /// P(x) = A(x) x B(x) x ... x N(x)
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars {
            return Err("evaluate must assign to all variables");
        }

        self.polynomials.iter().try_fold(F::one(), |product, poly| {
            poly.evaluate(assignments).map(|value| product * value)
        })
    }

    /// Partially evaluate each component polynomial on the same input, returns a new product_poly
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

    /// Converts the internal polynomials to evaluations and returns their element wise product
    pub fn prod_reduce(&self) -> Vec<F> {
        let mut result = self.polynomials[0].reduce();
        for polynomial in self.polynomials.iter().skip(1) {
            for (i, eval) in polynomial.reduce().iter().enumerate() {
                result[i] *= eval
            }
        }
        result
    }

    /// Serialize the ProductPoly
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
        // the max variable degree for a product poly is the sum of the
        // max variable degree of each component poly
        // e.g. 2a^2b * 3ab = 2a^3b^2 (2 + 1 = 3)
        // obviously this represents the max possible degree (which might deviate from the
        // true max variable degree).
        // TODO: is it possible to accurately determine the variable degree
        //  (what is the time complexity for this?)
        self.polynomials
            .iter()
            .map(|poly| poly.max_variable_degree())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::composed_poly::product_poly::ProductPoly;
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
    fn test_cannot_create_empty_product_poly() {
        ProductPoly::<Fr>::new(vec![]).unwrap();
    }

    #[test]
    fn test_product_poly_creation() {
        // create prod_poly from mle's with the same number of variables
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
        ProductPoly::new(vec![mle_a.into(), mle_b.into()]).unwrap();

        // create prod_poly from mle's with different number of variables
        let mle_a = MultiLinearPolynomial::new(1, vec![Fr::from(2), Fr::from(8)]).unwrap();
        let mle_b = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(22)],
        )
        .unwrap();
        let prod_poly = ProductPoly::new(vec![mle_a.into(), mle_b.into()]);
        assert_eq!(prod_poly.is_err(), true);
    }

    #[test]
    fn test_evaluate() {
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
        let mle_c = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(22)],
        )
        .unwrap();

        let direct_product = mle_a.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap()
            * mle_b.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap()
            * mle_c.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap();

        let prod_poly = ProductPoly::new(vec![mle_a.into(), mle_b.into(), mle_c.into()]).unwrap();

        assert_eq!(
            prod_poly.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap(),
            direct_product
        );
    }

    #[test]
    fn test_partial_evaluate() {
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
        let prod_poly = ProductPoly::new(vec![mle_a.clone().into(), mle_b.clone().into()]).unwrap();

        let mle_a_partial = mle_a.partial_evaluate(1, &[Fr::from(10)]).unwrap();
        let mle_b_partial = mle_b.partial_evaluate(1, &[Fr::from(10)]).unwrap();
        let prod_poly_expected_partial =
            ProductPoly::new(vec![mle_a_partial.into(), mle_b_partial.into()]).unwrap();

        assert_eq!(
            prod_poly.partial_evaluate(1, &[Fr::from(10)]).unwrap(),
            prod_poly_expected_partial
        );
    }

    #[test]
    fn test_prod_reduce() {
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
        let prod_poly = ProductPoly::new(vec![mle_a.into(), mle_b.into()]).unwrap();

        assert_eq!(
            prod_poly.prod_reduce(),
            vec![Fr::from(4), Fr::from(64), Fr::from(100), Fr::from(308)]
        );
    }

    #[test]
    fn test_max_variable_degree() {
        // p = 2ab + 3bc
        // create a product polynomial with 3 copies of p
        // P = p . p . p
        // we expect a degree 3 polynomial

        let p = p_2ab_3bc();

        let product_1 = ProductPoly::new(vec![p.clone().into(), p.clone().into()]).unwrap();
        assert_eq!(product_1.max_variable_degree(), 2);

        let product_2 = ProductPoly::new(vec![product_1.into(), p.into()]).unwrap();
        assert_eq!(product_2.max_variable_degree(), 3);
    }

    #[test]
    fn test_flatten() {
        let p = p_2ab_3bc();

        let product_1 = ProductPoly::new(vec![p.clone().into(), p.clone().into()]).unwrap();
        assert_eq!(product_1.max_variable_degree(), 2);

        let product_2 = ProductPoly::new(vec![product_1.into(), p.into()]).unwrap();
        assert_eq!(product_2.max_variable_degree(), 3);

        // TODO: ideally product 2 should collapse, into a vector of 3 elements
        //  this won't be the case now but will fix this
        //  failing test
        assert_eq!(product_2.polynomials.len(), 3);
    }
}
