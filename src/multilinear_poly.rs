use ark_ff::PrimeField;

/// Polynomial term represents a monomial
/// The first part of the tuple is the coefficient
/// The second part of the tuple is the variable selector
/// e.g. vars = [a, b, c, d, e]
/// then 5ac = (5, vec![1, 0, 1, 0, 0])
type PolynomialTerm<F> = (F, Vec<bool>);

// TODO: add documentation explaining the structure
struct MultiLinearPolynomial<F: PrimeField> {
    n_vars: u32,
    coefficients: Vec<F>,
}

impl<F: PrimeField> MultiLinearPolynomial<F> {
    /// Instantiate a new Multilinear polynomial, from polynomial terms
    // TODO: use error object not string
    fn new(number_of_variables: u32, terms: Vec<PolynomialTerm<F>>) -> Result<Self, &'static str> {
        let total_variable_combinations = 2_i32.pow(number_of_variables) as usize;
        let mut coefficients = vec![F::zero(); total_variable_combinations];
        for term in terms {
            if term.1.len() != number_of_variables as usize {
                return Err("the selector array len should be the same as the number of variables");
            }
            coefficients[Self::selector_to_index(&term.1)] += term.0;
        }
        Ok(Self {
            n_vars: number_of_variables,
            coefficients,
        })
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
    use crate::multilinear_poly::MultiLinearPolynomial;
    use ark_ff::{Fp64, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_polynomial_instantiation() {
        // variables = [a, b]
        // dense form [constant, a, b, ab]

        // Poly = 2ab
        // expected dense form = [0, 0, 0, 2]
        assert_eq!(
            MultiLinearPolynomial::new(2, vec![(Fq::from(2), vec![true, true])])
                .unwrap()
                .coefficients,
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)]
        );

        // Poly = 2a + 3b + 5ab
        // expected dense form = [0, 2, 3, 5]
        assert_eq!(
            MultiLinearPolynomial::new(
                2,
                vec![
                    (Fq::from(2), vec![true, false]),
                    (Fq::from(3), vec![false, true]),
                    (Fq::from(5), vec![true, true])
                ]
            )
            .unwrap()
            .coefficients,
            vec![Fq::from(0), Fq::from(2), Fq::from(3), Fq::from(5)]
        );

        // constant = 5
        // expected dense form = [5, 0, 0, 0]
        assert_eq!(
            MultiLinearPolynomial::new(2, vec![(Fq::from(5), vec![false, false])])
                .unwrap()
                .coefficients,
            vec![Fq::from(5), Fq::from(0), Fq::from(0), Fq::from(0)]
        );

        // Simplification
        // Poly = 2ab + 3ab + 4b
        // simplified = 5ab + 4b
        // expected dense form = [0, 0, 4, 5]
        assert_eq!(
            MultiLinearPolynomial::new(
                2,
                vec![
                    (Fq::from(2), vec![true, true]),
                    (Fq::from(3), vec![true, true]),
                    (Fq::from(4), vec![false, true])
                ]
            )
            .unwrap()
            .coefficients,
            vec![Fq::from(0), Fq::from(0), Fq::from(4), Fq::from(5)]
        );
    }

    #[test]
    fn test_polynomial_instantiation_invalid_variables() {
        // polynomial expects 3 variables by passed a term with just 2 variables
        assert_eq!(
            MultiLinearPolynomial::new(3, vec![(Fq::from(2), vec![true, true])]).is_err(),
            true
        );
    }

    #[test]
    fn test_selector_to_index() {
        // [a, b, c, d] -> [1, 2, 4, 8]
        // index for constant is 0
        assert_eq!(
            MultiLinearPolynomial::<Fq>::selector_to_index(&[false, false, false, false]),
            0
        );
        // index for a is 1
        assert_eq!(
            MultiLinearPolynomial::<Fq>::selector_to_index(&[true, false, false, false]),
            1
        );
        // index for b is 2
        assert_eq!(
            MultiLinearPolynomial::<Fq>::selector_to_index(&[false, true, false, false]),
            2
        );
        // index for abd = 1 + 2 + 8 = 11
        assert_eq!(
            MultiLinearPolynomial::<Fq>::selector_to_index(&[true, true, false, true]),
            11
        );
    }
}
