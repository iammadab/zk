use ark_ff::PrimeField;
use crate::multilinear::evaluation_form::MultiLinearPolynomial;
use crate::product_poly::ProductPoly;

// TODO: add documentation (also document each piece)
enum ComposedPolynomial<F: PrimeField> {
    Unit(MultiLinearPolynomial<F>),
    // Product(ProductPoly<F>)
}

impl<F: PrimeField> ComposedPolynomial<F> {
    // TODO: add documentation
    fn new_unit_poly(poly: MultiLinearPolynomial<F>) -> Self {
        Self::Unit(poly)
    }

    // TODO: add documentation
    // fn new_product_poly(polys: Vec<ComposedPolynomial<F>>) -> Self {
    //     Self::Product(ProductPoly::new(polys))
    // }
}