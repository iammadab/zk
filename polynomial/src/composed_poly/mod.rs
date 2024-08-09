use crate::composed_poly::product_poly::ProductPoly;
use crate::multilinear::evaluation_form::MultiLinearPolynomial;
use ark_ff::PrimeField;

pub mod product_poly;
mod sum_poly;

#[derive(Clone, Debug, PartialEq)]
// TODO: add documentation (also document each piece)
pub enum ComposedPolynomial<F: PrimeField> {
    Unit(MultiLinearPolynomial<F>),
    Product(ProductPoly<F>),
}

impl<F: PrimeField> ComposedPolynomial<F> {
    // TODO: add documentation
    pub fn unit(poly: MultiLinearPolynomial<F>) -> Self {
        Self::Unit(poly)
    }

    // TODO: add documentation
    pub fn product(polys: Vec<ComposedPolynomial<F>>) -> Result<Self, &'static str> {
        Ok(Self::Product(ProductPoly::new(polys)?))
    }

    // TODO: add documentation
    // TODO: in need of macro
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.evaluate(assignments),
            ComposedPolynomial::Product(poly) => poly.evaluate(assignments),
        }
    }

    // TODO: add documentation
    pub fn partial_evaluate(
        &self,
        initial_var: usize,
        assignments: &[F],
    ) -> Result<Self, &'static str> {
        match &self {
            ComposedPolynomial::Unit(poly) => poly
                .partial_evaluate(initial_var, assignments)
                .map(Self::Unit),
            ComposedPolynomial::Product(poly) => poly
                .partial_evaluate(initial_var, assignments)
                .map(Self::Product),
        }
    }

    // TODO: add documentation
    pub fn reduce(&self) -> Vec<F> {
        match &self {
            // TODO: get rid of the to_vec if possible
            //  if not then make sure it is not called alot
            ComposedPolynomial::Unit(poly) => poly.evaluation_slice().to_vec(),
            ComposedPolynomial::Product(poly) => poly.prod_reduce(),
        }
    }

    // TODO: add documentation
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.to_bytes(),
            ComposedPolynomial::Product(poly) => poly.to_bytes(),
        }
    }

    // TODO: add documentation
    // TODO: this function can be generated via a macro
    pub fn n_vars(&self) -> usize {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.n_vars(),
            ComposedPolynomial::Product(poly) => poly.n_vars(),
        }
    }

    // TODO: add documentation
    pub fn max_variable_degree(&self) -> usize {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.max_variable_degree(),
            ComposedPolynomial::Product(poly) => poly.max_variable_degree(),
        }
    }
}

// TODO: add documentation
impl<F: PrimeField> From<MultiLinearPolynomial<F>> for ComposedPolynomial<F> {
    fn from(poly: MultiLinearPolynomial<F>) -> Self {
        Self::Unit(poly)
    }
}

// TODO: add documentation
impl<F: PrimeField> From<ProductPoly<F>> for ComposedPolynomial<F> {
    fn from(poly: ProductPoly<F>) -> Self {
        Self::Product(poly)
    }
}
