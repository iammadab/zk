use crate::multilinear::evaluation_form::MultiLinearPolynomial;
use ark_ff::PrimeField;

// TODO: add documentation
// TODO: can be generalized further (more operations, not just mles too)
/// P(x) = A(x).B(x).C(x)
#[derive(Debug)]
struct ProductPoly<F: PrimeField> {
    n_vars: usize,
    polynomials: Vec<MultiLinearPolynomial<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    /// Instantiate a new product_poly from a set of `Multilinear` polynomials
    pub fn new(polynomials: Vec<MultiLinearPolynomial<F>>) -> Result<Self, &'static str> {
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
    /// P(x) = A(x).B(x).C(x)
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars {
            return Err("evaluate must assign to all variables");
        }

        self.polynomials.iter().try_fold(F::one(), |product, poly| {
            poly.evaluate(assignments).map(|value| product * value)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::multilinear::evaluation_form::MultiLinearPolynomial;
    use crate::product_poly::ProductPoly;
    use ark_bls12_381::Fr;

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
        let prod_poly = ProductPoly::new(vec![mle_a, mle_b]).unwrap();

        // create prod_poly from mle's with different number of variables
        let mle_a = MultiLinearPolynomial::new(1, vec![Fr::from(2), Fr::from(8)]).unwrap();
        let mle_b = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(22)],
        )
        .unwrap();
        let prod_poly = ProductPoly::new(vec![mle_a, mle_b]);
        assert_eq!(prod_poly.is_err(), true);
    }

    #[test]
    fn test_evaluate() {
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
        let mle_c = MultiLinearPolynomial::new(
            2,
            vec![Fr::from(2), Fr::from(8), Fr::from(10), Fr::from(22)],
        )
        .unwrap();

        let direct_product = mle_a.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap()
            * mle_b.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap()
            * mle_c.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap();

        let prod_poly = ProductPoly::new(vec![mle_a, mle_b, mle_c]).unwrap();

        assert_eq!(
            prod_poly.evaluate(&[Fr::from(1), Fr::from(10)]).unwrap(),
            direct_product
        );
    }
}
