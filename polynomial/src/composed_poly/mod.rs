use crate::multilinear::evaluation_form::MultiLinearPolynomial;
use crate::product_poly::ProductPoly;
use ark_ff::PrimeField;

#[derive(Clone, Debug, PartialEq)]
// TODO: add documentation (also document each piece)
pub enum ComposedPolynomial<F: PrimeField> {
    Unit(MultiLinearPolynomial<F>),
    // Product(ProductPoly<F>)
}

impl<F: PrimeField> ComposedPolynomial<F> {
    // TODO: add documentation
    pub fn unit_poly(poly: MultiLinearPolynomial<F>) -> Self {
        Self::Unit(poly)
    }

    // TODO: add documentation
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.evaluate(assignments),
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
                .map(|p| p.into()),
        }
    }

    // TODO: add documentation
    pub fn reduce(&self) -> Vec<F> {
        match &self {
            // TODO: get rid of the to_vec if possible
            //  if not then make sure it is not called alot
            ComposedPolynomial::Unit(poly) => poly.evaluation_slice().to_vec(),
        }
    }

    // TODO: add documentation
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.to_bytes(),
        }
    }

    // TODO: add documentation
    // TODO: this function can be generated via a macro
    pub fn n_vars(&self) -> usize {
        match &self {
            ComposedPolynomial::Unit(poly) => poly.n_vars(),
        }
    }
}

// TODO: add documentation
impl<F: PrimeField> From<MultiLinearPolynomial<F>> for ComposedPolynomial<F> {
    fn from(value: MultiLinearPolynomial<F>) -> Self {
        Self::Unit(value)
    }
}
