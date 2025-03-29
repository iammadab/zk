#![allow(unused)]

use ark_ff::PrimeField;

use crate::product_poly::ProductPoly;

pub struct SumPoly<F: PrimeField> {
    n_vars: usize,
    polynomials: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(polynomials: Vec<ProductPoly<F>>) -> Result<Self, &'static str> {
        todo!()
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
