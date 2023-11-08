use ark_ff::PrimeField;

/// Polynomial term represents a monomial
/// The first part of the tuple is the coefficient
/// The second part of the tuple is the variable selector
/// e.g. vars = [a, b, c, d, e]
/// then 5ac = (5, vec![1, 0, 1, 0, 0])
type PolynomialTerm<F> = (F, Vec<bool>);

// TODO: add documentation explaining the structure
struct MultiLinearPolynomial<F: PrimeField> {
    n_vars: usize,
    coefficients: Vec<F>
}

impl<F: PrimeField> MultiLinearPolynomial<F> {
    fn new(number_of_variables: usize, terms: Vec<PolynomialTerm<F>>) -> Self {
        // need to convert the term to
        todo!()
    }

    /// Convert a selector to an index in the dense polynomial
    fn selector_to_index(selector: &[bool]) -> usize {
        let mut sum = 0;
        let mut adder = 1;

        for i in 0..selector.len() {
            if selector[i] {
                sum += adder;
            }
            adder *= 2;
        }

        sum
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::{Fp64, MontBackend, MontConfig};
    use crate::multilinear_poly::MultiLinearPolynomial;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_selector_to_index() {
        // [a, b, c, d] -> [1, 2, 4, 8]
        // index for constant is 0
        assert_eq!(MultiLinearPolynomial::<Fq>::selector_to_index(&[false, false, false, false]), 0);
        // index for a is 1
        assert_eq!(MultiLinearPolynomial::<Fq>::selector_to_index(&[true, false, false, false]), 1);
        // index for b is 2
        assert_eq!(MultiLinearPolynomial::<Fq>::selector_to_index(&[false, true, false, false]), 2);
        // index for abd = 1 + 2 + 8 = 11
        assert_eq!(MultiLinearPolynomial::<Fq>::selector_to_index(&[true, true, false, true]), 11);
    }
}