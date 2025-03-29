#![allow(unused)]

use ark_ff::PrimeField;

use crate::product_poly::ProductPoly;

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

    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        todo!()
    }

    pub fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        todo!()
    }

    pub fn sum_reduce(&self) -> Vec<F> {
        todo!()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn n_vars(&self) -> usize {
        self.n_vars
    }
}
