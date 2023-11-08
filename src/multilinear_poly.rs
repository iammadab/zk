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
        let mut coefficients =
            vec![F::zero(); Self::variable_combination_count(number_of_variables)];
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

    /// Figure out all the index values that a variable appears in
    fn get_variable_indexes(
        number_of_variables: u32,
        selector: Vec<bool>,
    ) -> Result<Vec<usize>, &'static str> {
        if selector.len() != number_of_variables as usize {
            return Err("the selector array len should be the same as the number of variables");
        }

        // Ensure that only a single variable is selected
        // return an error if the constant is selected or multiple variables are selected
        let selector_sum = selector.iter().fold(0, |sum, selection| {
            if *selection {
                return sum + 1;
            }
            sum
        });

        if selector_sum != 1 {
            return Err("only select single variable, cannot get indexes for constant or multiple variables");
        }

        let variable_id = Self::selector_to_index(&selector);
        let mut indexes = vec![];
        let mut count = 0;
        let mut skip = false;

        let max_array_index = Self::variable_combination_count(number_of_variables) - 1;

        for i in variable_id..=max_array_index {
            if count == variable_id {
                skip = !skip;
                count = 0;
            }

            if !skip {
                indexes.push(i);
            }

            count += 1;
        }

        Ok(indexes)
    }

    /// Returns the number of elements in the dense polynomial representation
    fn variable_combination_count(number_of_variables: u32) -> usize {
        2_i32.pow(number_of_variables) as usize
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

    #[test]
    fn test_get_variable_indexes() {
        // Given 4 variables [a, b, c, d]
        // Dense form is this:
        // [const (0), a (1), b (2), ab (3), c (4), ac (5), bc (6), abc (7), d (8),
        //     ad (9), bd (10), abd (11), cd (12), acd (13), bcd (14), abcd (15)]
        // indexes per variables:
        //  a = [1, 3, 5, 7, 9, 11, 13, 15]
        //  b = [2, 3, 6, 7, 10, 11, 14, 15]
        //  c = [4, 5, 6, 7, 12, 13, 14, 15]
        //  d = [8, 9, 10, 11, 12, 13, 14, 15]

        // you cannot get indexes for const or multiple variables
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![false, false, false, false])
                .is_err(),
            true
        );
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![true, false, true, false])
                .is_err(),
            true
        );

        // get all a indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![true, false, false, false])
                .unwrap(),
            vec![1, 3, 5, 7, 9, 11, 13, 15]
        );
        // get all b indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![false, true, false, false])
                .unwrap(),
            vec![2, 3, 6, 7, 10, 11, 14, 15]
        );
        // get all c indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![false, false, true, false])
                .unwrap(),
            vec![4, 5, 6, 7, 12, 13, 14, 15]
        );
        // get all d indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, vec![false, false, false, true])
                .unwrap(),
            vec![8, 9, 10, 11, 12, 13, 14, 15]
        );
    }
}
