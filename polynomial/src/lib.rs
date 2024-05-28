use ark_ff::PrimeField;

use self::univariate_poly::UnivariatePolynomial;

pub mod multilinear;
pub mod univariate_poly;

pub trait Polynomial<F: PrimeField>: Clone {
    /// Returns the number of variables in the extension
    fn n_vars(&self) -> usize;

    /// Assign a value to every variable, return the evaluation
    fn evaluate_slice(&self, assignments: &[F]) -> Result<F, &'static str>;

    /// Fix certain variables in the polynomial, return the reduced polynomial
    fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str>
    where
        Self: Sized;

    /// Remove variables that are no longer used (shrinking the polynomial)
    /// e.g. 2a + 9c uses three variables [a, b, c] but b is not represented in any term
    /// we can relabel to 2a + 9b using just 2 variables
    fn relabel(self) -> Self;

    /// Additive Identity
    fn additive_identity() -> Self;

    /// Converts the polynomial to a sequence of bytes
    /// mostly used for fiat-shamir
    fn to_bytes(&self) -> Vec<u8>;

    // TODO: this might be removed (doesn't have to be a strict requirement)
    /// Attempt conversion to univariate polynomial
    fn to_univariate(&self) -> Result<UnivariatePolynomial<F>, &'static str>;
}
