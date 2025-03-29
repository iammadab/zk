use ark_ff::PrimeField;

use crate::product_poly::ProductPoly;

#[derive(Clone)]
pub struct SumPoly<F: PrimeField> {
    n_vars: usize,
    polynomials: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(polynomials: Vec<ProductPoly<F>>) -> Result<Self, &'static str> {
        if polynomials.is_empty() {
            return Err("cannot create sum poly from empty polynomials");
        }

        // ensure all the product polynomials have the same number of variables
        let expected_num_of_vars = polynomials[0].n_vars();
        let equal_variables = polynomials
            .iter()
            .all(|poly| poly.n_vars() == expected_num_of_vars);
        if !equal_variables {
            return Err("all product polys should have the same number of variables");
        }

        Ok(Self {
            n_vars: expected_num_of_vars,
            polynomials,
        })
    }

    /// Evaluates P(x) = A(x) + B(x) + ... + N(x)
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars {
            return Err("evaluate must assign to all variables");
        }

        self.polynomials.iter().try_fold(F::zero(), |sum, poly| {
            poly.evaluate(assignments).map(|value| sum + value)
        })
    }

    /// Partially evalutes the constituent polynomials
    pub fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        let partial_polynomials = self
            .polynomials
            .iter()
            .map(|poly| poly.partial_evaluate(initial_var, assignments))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            n_vars: partial_polynomials[0].n_vars(),
            polynomials: partial_polynomials,
        })
    }

    /// The boolean hypercube representation of the sum poly
    pub fn sum_reduce(&self) -> Vec<F> {
        let mut result = self.polynomials[0].prod_reduce().to_vec();
        for poly in self.polynomials.iter().skip(1) {
            for (i, eval) in poly.prod_reduce().iter().enumerate() {
                result[i] += eval
            }
        }
        result
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.polynomials
            .iter()
            .map(|poly| poly.to_bytes())
            .collect::<Vec<Vec<u8>>>()
            .concat()
    }

    pub fn n_vars(&self) -> usize {
        self.n_vars
    }
}

#[cfg(test)]
mod test {
    use crate::{multilinear::evaluation_form::MultiLinearPolynomial, product_poly::ProductPoly};

    use super::SumPoly;
    use ark_bls12_381::Fr;

    fn sum_poly() -> SumPoly<Fr> {
        // 2a + 2b
        let p1 = MultiLinearPolynomial::new_with_pad(
            vec![Fr::from(0), Fr::from(2), Fr::from(2), Fr::from(4)],
            None,
        );

        // 3a + b
        let p2 = MultiLinearPolynomial::new_with_pad(
            vec![Fr::from(0), Fr::from(1), Fr::from(3), Fr::from(4)],
            None,
        );

        SumPoly::new(vec![
            ProductPoly::new(vec![p2]).unwrap(),
            ProductPoly::new(vec![p1]).unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn test_sum_poly_evaluate() {
        let poly = sum_poly();

        // 2(2) + 2(3) = 4 + 6 = 10
        // 3(2) + 3 = 6 + 3 = 9
        // 10 + 9 = 19
        assert_eq!(
            poly.evaluate(&[Fr::from(2), Fr::from(3)]).unwrap(),
            Fr::from(19)
        );
    }

    #[test]
    fn test_sum_poly_partial_evaluate() {
        let poly = sum_poly();
        let poly2 = poly.partial_evaluate(0, &[Fr::from(2)]).unwrap();
        assert_eq!(poly2.n_vars(), 1);
        assert_eq!(poly2.evaluate(&[Fr::from(3)]).unwrap(), Fr::from(19));
    }

    #[test]
    fn test_sum_reduce() {
        let poly = sum_poly();
        assert_eq!(
            poly.sum_reduce(),
            vec![Fr::from(0), Fr::from(3), Fr::from(5), Fr::from(8)]
        );
    }
}
