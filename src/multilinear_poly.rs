use ark_ff::PrimeField;
use std::ops::{Add, Mul};

/// Polynomial term represents a monomial
/// The first part of the tuple is the coefficient
/// The second part of the tuple is the variable selector
/// e.g. vars = [a, b, c, d, e]
/// then 5ac = (5, vec![1, 0, 1, 0, 0])
type PolynomialTerm<F> = (F, Vec<bool>);

#[derive(Clone, PartialEq, Debug)]
/// Dense representation of the multilinear polynomial
/// the coefficient vector has a slot for each combination of variables
/// e.g. number_of_variables = 3 a, b, c
/// coefficient_vec = [constant, a, b, ab, c, ac, bc, abc]
/// each variable has an implicit id that allows for efficient lookups in the coefficient_vec
/// each variable is assigned a power of 2
/// [a, b, c] = [2^0, 2^1, 2^2] = [1, 2, 4]
/// now to index any combination of variables, just sum the individual ids
/// e.g. ab = 1 + 2 = index 3
///     or bc = 2 + 4 = index 6
pub struct MultiLinearPolynomial<F: PrimeField> {
    n_vars: u32,
    coefficients: Vec<F>,
}

impl<F: PrimeField> MultiLinearPolynomial<F> {
    /// Instantiate a new Multilinear polynomial, from polynomial terms
    pub fn new(
        number_of_variables: u32,
        terms: Vec<PolynomialTerm<F>>,
    ) -> Result<Self, &'static str> {
        let mut coefficients =
            vec![F::zero(); Self::variable_combination_count(number_of_variables)];
        for term in terms {
            if term.1.len() != number_of_variables as usize {
                return Err("the selector array len should be the same as the number of variables");
            }
            coefficients[selector_to_index(&term.1)] += term.0;
        }
        Ok(Self {
            n_vars: number_of_variables,
            coefficients,
        })
    }

    /// Instantiate Multilinear polynomial directly from coefficients
    pub fn new_with_coefficient(
        number_of_variables: u32,
        coefficients: Vec<F>,
    ) -> Result<Self, &'static str> {
        if coefficients.len() != Self::variable_combination_count(number_of_variables) {
            return Err("coefficients must be a dense representation of the number of variables");
        }

        Ok(Self {
            n_vars: number_of_variables,
            coefficients,
        })
    }

    /// Partially assign values to variables in the polynomial
    /// Returns the resulting polynomial once those variables have been fixed
    pub fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str> {
        // When partially evaluate a variable in a monomial, we need to multiply the variable assignment
        // with the previous coefficient, then move the new coefficient to the appropriate monomial
        // e.g p = 5abc partially evaluating a = 2
        // new coefficient will be 5*2 = 10 and new monomial will be bc
        // resulting in 10bc
        // recall each variable has an index in the power of two
        // [a, b, c] = [1, 2, 4]
        // given a term e.g. abc the index is 1 + 2 + 4 = 7
        // to get the index of the result, just subtract the variable being evaluated
        // 7 - 1 = 6
        // and bc = 2 + 4 = 6

        let mut evaluated_polynomial = self.clone();
        for (selector, coeff) in assignments {
            let variable_indexes = Self::get_variable_indexes(self.n_vars, selector)?;
            for i in variable_indexes {
                let result_index = i - selector_to_index(selector);
                let updated_coefficient = evaluated_polynomial.coefficients[i] * *coeff;
                evaluated_polynomial.coefficients[result_index] += updated_coefficient;
                evaluated_polynomial.coefficients[i] = F::zero();
            }
        }
        Ok(evaluated_polynomial)
    }

    /// Assign a value to every variable in the polynomial, result is a Field element
    pub fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        // Associates every assignment with the correct selector vector and calls
        // partial evaluate on the expanded assignment

        if assignments.len() != self.n_vars as usize {
            return Err("evaluate requires an assignment for every variable");
        }

        let mut indexed_assignments = vec![];
        for (position, assignment) in assignments.into_iter().enumerate() {
            indexed_assignments.push((
                selector_from_position(self.n_vars as usize, position)?,
                assignment,
            ))
        }

        let evaluated_poly = self.partial_evaluate(&indexed_assignments)?;

        Ok(evaluated_poly.coefficients[0])
    }

    /// Interpolate a set of values over the boolean hypercube
    pub fn interpolate(values: &[F]) -> Self {
        let num_of_variables = (values.len() as f32).log2().ceil() as u32;
        let mut result = Self::additive_identity(num_of_variables);
        for (i, value) in values.iter().enumerate() {
            let poly =
                Self::lagrange_basis_poly(i, num_of_variables as usize).scalar_multiply(value);
            result = (&result + &poly).unwrap();
        }
        result
    }

    /// Generate a checker polynomial for a boolean value that
    /// outputs 1 if the boolean values match, 0 otherwise
    fn lagrange_basis_poly(index: usize, num_of_vars: usize) -> Self {
        let binary_value = binary_string(index, num_of_vars);
        let mut result = Self::multiplicative_identity();
        for char in binary_value.chars() {
            if char == '1' {
                result = &result * &Self::check_one();
            } else {
                result = &result * &Self::check_zero();
            }
        }
        result
    }

    /// Multilinear polynomial to check if a variable in the boolean space is 0
    fn check_zero() -> Self {
        // p = 1 - a
        Self::new(
            1,
            vec![(F::one(), vec![false]), (F::one().neg(), vec![true])],
        )
        .unwrap()
    }

    /// Multilinear polynomial to check if a variable in the boolean space is 1
    fn check_one() -> Self {
        // p = a
        Self::new(1, vec![(F::one(), vec![true])]).unwrap()
    }

    /// Multiplicative identity poly
    fn multiplicative_identity() -> Self {
        Self::new(0, vec![(F::one(), vec![])]).unwrap()
    }

    /// Additive identity poly
    fn additive_identity(num_of_vars: u32) -> Self {
        Self::new(
            num_of_vars,
            vec![(F::zero(), vec![false; num_of_vars as usize])],
        )
        .unwrap()
    }

    /// Co-efficient wise multiplication with scalar
    pub fn scalar_multiply(&self, scalar: &F) -> Self {
        // TODO: try implementing inplace operations
        let mut updated_coefficients = self
            .coefficients
            .clone()
            .into_iter()
            .map(|coeff| coeff * scalar)
            .collect();
        Self::new_with_coefficient(self.n_vars, updated_coefficients)
            .expect("number of variables are the same in scalar mul")
    }

    /// Figure out all the index values that a variable appears in
    fn get_variable_indexes(
        number_of_variables: u32,
        selector: &[bool],
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

        let variable_id = selector_to_index(&selector);
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

impl<F: PrimeField> Add for &MultiLinearPolynomial<F> {
    type Output = Result<MultiLinearPolynomial<F>, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        // Addition doesn't require that the number of coefficient should match
        // both RHS and Self will always have coefficients that are powers of 2
        // as non of the implement functions violates this invariant.
        // only thing that can affect this is manual manipulation of the coefficient vector

        // To add we clone the longer coefficient vector then sum the smaller one into that
        let (n_vars, mut longer_coeff, shorter_coeff) =
            if self.coefficients.len() > rhs.coefficients.len() {
                (self.n_vars, self.coefficients.clone(), &rhs.coefficients)
            } else {
                (rhs.n_vars, rhs.coefficients.clone(), &self.coefficients)
            };

        for (i, coeff) in shorter_coeff.iter().enumerate() {
            longer_coeff[i] = longer_coeff[i] + coeff;
        }

        Ok(MultiLinearPolynomial::new_with_coefficient(
            n_vars,
            longer_coeff,
        )?)
    }
}

impl<F: PrimeField> Mul for &MultiLinearPolynomial<F> {
    type Output = MultiLinearPolynomial<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        // if any of the poly is a scalar poly (having no variable) we just perform scalar multiplication
        if self.n_vars == 0 {
            return rhs.scalar_multiply(&self.coefficients[0]);
        } else if rhs.n_vars == 0 {
            return self.scalar_multiply(&rhs.coefficients[0]);
        };

        // It is assumed that both lhs and rhs don't share common variables
        // if they did then this multiplication will be multivariate
        // the resulting polynomial number of variables is the sum of the lhs and rhs n_vars
        let mut new_poly_coefficients =
            vec![
                F::zero();
                MultiLinearPolynomial::<F>::variable_combination_count(self.n_vars + rhs.n_vars)
            ];

        // for each term multiplication, if any is zero, we don't compute anything as the result vector started
        // with all zeros.
        // if both are non zero, we mul the coefficient, then figure out the correct slot for this new result
        // e.g. 2a * 3b = 6ab (6 has to be inserted in the slot for ab)
        for i in 0..self.coefficients.len() {
            for j in 0..rhs.coefficients.len() {
                if self.coefficients[i].is_zero() || rhs.coefficients[j].is_zero() {
                    continue;
                }
                let new_coefficient = self.coefficients[i] * rhs.coefficients[j];
                let mut left_index_vec = selector_from_usize(i, self.n_vars as usize);
                let mut right_index_vec = selector_from_usize(j, rhs.n_vars as usize);
                left_index_vec.append(&mut right_index_vec);

                let result_index = selector_to_index(&left_index_vec);
                new_poly_coefficients[result_index] += new_coefficient;
            }
        }

        MultiLinearPolynomial::new_with_coefficient(self.n_vars + rhs.n_vars, new_poly_coefficients)
            .unwrap()
    }
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

/// Convert a number to a vec of bool
fn selector_from_usize(value: usize, min_size: usize) -> Vec<bool> {
    let binary_value = format!("{:b}", value);
    let mut result = vec![];
    for char in binary_value.chars() {
        if char == '1' {
            result.push(true)
        } else {
            result.push(false)
        }
    }
    result.reverse();
    for _ in 0..(min_size - binary_value.len()) {
        result.push(false);
    }
    result
}

/// Returns a Vec<bool> of a given size, with default value set to false, except the position index
fn selector_from_position(size: usize, position: usize) -> Result<Vec<bool>, &'static str> {
    if position > size - 1 {
        return Err("position index out of bounds");
    }

    let mut selector = vec![false; size];
    selector[position] = true;
    Ok(selector)
}

/// Convert a number to a binary string of a given size
fn binary_string(index: usize, bit_count: usize) -> String {
    let binary = format!("{:b}", index);
    "0".repeat(bit_count - binary.len()) + &binary
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::{selector_to_index, MultiLinearPolynomial};
    use ark_ff::{Fp64, MontBackend, MontConfig, One, Zero};
    use std::ops::Neg;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    // TODO: move this functionality into the polynomial struct
    fn fq_from_vec(values: Vec<i64>) -> Vec<Fq> {
        values.into_iter().map(Fq::from).collect()
    }

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
        assert_eq!(selector_to_index(&[false, false, false, false]), 0);
        // index for a is 1
        assert_eq!(selector_to_index(&[true, false, false, false]), 1);
        // index for b is 2
        assert_eq!(selector_to_index(&[false, true, false, false]), 2);
        // index for abd = 1 + 2 + 8 = 11
        assert_eq!(selector_to_index(&[true, true, false, true]), 11);
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
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[false, false, false, false])
                .is_err(),
            true
        );
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[true, false, true, false])
                .is_err(),
            true
        );

        // get all a indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[true, false, false, false])
                .unwrap(),
            vec![1, 3, 5, 7, 9, 11, 13, 15]
        );
        // get all b indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[false, true, false, false])
                .unwrap(),
            vec![2, 3, 6, 7, 10, 11, 14, 15]
        );
        // get all c indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[false, false, true, false])
                .unwrap(),
            vec![4, 5, 6, 7, 12, 13, 14, 15]
        );
        // get all d indexes
        assert_eq!(
            MultiLinearPolynomial::<Fq>::get_variable_indexes(4, &[false, false, false, true])
                .unwrap(),
            vec![8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    fn poly_5ab_7bc_8d() -> MultiLinearPolynomial<Fq> {
        // p = 5ab + 7bc + 8d
        MultiLinearPolynomial::new(
            4,
            vec![
                (Fq::from(5), vec![true, true, false, false]),
                (Fq::from(7), vec![false, true, true, false]),
                (Fq::from(8), vec![false, false, false, true]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_empty_partial_evaluation() {
        let p = poly_5ab_7bc_8d();
        let p_eval = poly_5ab_7bc_8d().partial_evaluate(&[]).unwrap();
        assert_eq!(p, p_eval);
    }

    #[test]
    fn test_partial_eval_happy_path() {
        // p = 5ab + 7bc + 8d
        // partial eval a and b
        // a = 2 b = 3
        // p = 5(2)(3) + 7(3)c + 8d
        // p = 30 + 21c + 8d
        // apply mod 17
        // p = 13 + 4c + 8d
        // [13, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]
        let p = poly_5ab_7bc_8d();
        let p_13_4c_8d = poly_5ab_7bc_8d()
            .partial_evaluate(&[
                (vec![false, true, false, false], &Fq::from(3)),
                (vec![true, false, false, false], &Fq::from(2)),
            ])
            .unwrap();
        assert_eq!(
            p_13_4c_8d.coefficients,
            fq_from_vec(vec![13, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0])
        );

        // p = 13 + 4c + 8d
        // eval c = 2
        // p = 13 + 8 + 8d = 21 + 8d
        // apply mod 17
        // p = 4 + 8d
        // dense form
        // [4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]
        let p_21_8d = p_13_4c_8d
            .partial_evaluate(&[(vec![false, false, true, false], &Fq::from(2))])
            .unwrap();
        assert_eq!(
            p_21_8d.coefficients,
            fq_from_vec(vec![4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_partial_eval_assign_all() {
        // p = 5ab + 7bc + 8d
        // a = 2, b = 4, c = 3, d = 5
        // p = 5(2)(4) + 7(4)(3) + 8(5)
        // p = 40 + 84 + 40
        // p = 164
        // apply mod 17
        // p = 11
        // dense form
        // [11, .....]
        let p = poly_5ab_7bc_8d();
        let eval = poly_5ab_7bc_8d()
            .partial_evaluate(&[
                (vec![true, false, false, false], &Fq::from(2)),
                (vec![false, true, false, false], &Fq::from(4)),
                (vec![false, false, true, false], &Fq::from(3)),
                (vec![false, false, false, true], &Fq::from(5)),
            ])
            .unwrap();
        assert_eq!(
            eval.coefficients,
            fq_from_vec(vec![11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_partial_eval_repeated_assignment() {
        // p = 5ab + 7bc + 8d
        // a = 2, a = 3, b = 4, c = 3, d = 5
        // should use the first instance of a only
        // hence:
        // p = 5(2)(4) + 7(4)(3) + 8(5)
        // p = 40 + 84 + 40
        // p = 164
        // apply mod 17
        // p = 11
        // dense form
        // [11, .....]
        let p = poly_5ab_7bc_8d();
        let eval = poly_5ab_7bc_8d()
            .partial_evaluate(&[
                (vec![true, false, false, false], &Fq::from(2)),
                (vec![true, false, false, false], &Fq::from(3)),
                (vec![false, true, false, false], &Fq::from(4)),
                (vec![false, false, true, false], &Fq::from(3)),
                (vec![false, false, false, true], &Fq::from(5)),
            ])
            .unwrap();
        assert_eq!(
            eval.coefficients,
            fq_from_vec(vec![11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_evaluation_incomplete_assignment() {
        // p has 4 variables so requires 4 assignments
        let p = poly_5ab_7bc_8d();
        assert_eq!(p.evaluate(&[Fq::from(4)]).is_err(), true);
    }

    #[test]
    fn test_evaluation_happy_path() {
        // p = 5ab + 7bc + 8d
        // a = 2, b = 4, c = 3, d = 5
        // p = 5(2)(4) + 7(4)(3) + 8(5)
        // p = 40 + 84 + 40
        // p = 164
        // apply mod 17
        // p = 11
        // dense form
        // [11, .....]
        let p = poly_5ab_7bc_8d();
        let eval = p.evaluate(&fq_from_vec(vec![2, 4, 3, 5])).unwrap();
        assert_eq!(eval, Fq::from(11));
    }

    #[test]
    fn test_polynomial_addition() {
        // p = 5ab + 7bc + 8d
        // q = 5ab + 7bc + 8d
        // sum = 10ab + 14bc + 16d
        // dense form
        // [0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0]
        let p = poly_5ab_7bc_8d();
        let q = poly_5ab_7bc_8d();
        let sum = (&p + &q).unwrap();

        assert_eq!(
            sum.coefficients,
            fq_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_scalar_multiplication() {
        // p = 5ab + 7bc + 8d
        // mul with 2
        // 2p = 10ab + 14bc + 16d
        // dense form
        // [0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0]
        let p = poly_5ab_7bc_8d();
        let two_p = p.scalar_multiply(&Fq::from(2));
        assert_eq!(
            two_p.coefficients,
            fq_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
        );

        // scalar mul with two polynomials
        let p = poly_5ab_7bc_8d();
        let scalar_poly = MultiLinearPolynomial::new(0, vec![(Fq::from(2), vec![])]).unwrap();
        let two_p = &p * &scalar_poly;
        assert_eq!(
            two_p.coefficients,
            fq_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_multilinear_poly_multiplication() {
        // p = 5ab
        // q = 6c
        // pq = 30abc
        // dense form:
        // [0, 0, 0, 0, 0, 0, 0, 30]
        let p = MultiLinearPolynomial::new(2, vec![(Fq::from(5), vec![true, true])]).unwrap();
        let q = MultiLinearPolynomial::new(1, vec![(Fq::from(6), vec![true])]).unwrap();
        let pq = &p * &q;
        assert_eq!(pq.n_vars, 3);
        assert_eq!(pq.coefficients, fq_from_vec(vec![0, 0, 0, 0, 0, 0, 0, 30]));

        // p = 3ac + 2ab
        // q = 7de
        // pq = 21acde + 14abde
        let p = MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(3), vec![true, false, true]),
                (Fq::from(2), vec![true, true, false]),
            ],
        )
        .unwrap();
        let q = MultiLinearPolynomial::new(2, vec![(Fq::from(7), vec![true, true])]).unwrap();
        let pq = &p * &q;
        assert_eq!(pq.n_vars, 5);

        let mut expected_coefficients = vec![Fq::from(0); 32];
        // [a, b, c, d, e] = [1, 2, 4, 8, 16]
        // set 14abde = 1 + 2 + 8 + 16 = 27
        // set 21acde = 1 + 4 + 8 + 16 = 29
        expected_coefficients[27] = Fq::from(14);
        expected_coefficients[29] = Fq::from(21);

        assert_eq!(pq.coefficients, expected_coefficients);
    }

    #[test]
    fn test_crazy_multilinear_poly_multiplication() {
        // p = 2a + 3bc + 6d
        // q = 4e + 5fg + 2h
        // pq = 8ae + 10afg + 4ah + 12bce + 15bcfg + 6bch + 24de + 30dfg + 12dh

        // result indexes
        // [a, b, c, d, e, f, g, h] = [1, 2, 4, 8, 16, 32, 64, 128]
        // 8ae -> 1 + 6 = 17
        // 10afg -> 1 + 32 + 64 = 97
        // 4ah = 1 + 128 = 129
        // 12bce = 2 + 4 + 16 = 22
        // 15bcfg = 2 + 4 + 32 + 64 = 102
        // 6bch = 2 + 4 + 128 = 134
        // 24de = 16 + 8 = 24
        // 30dfg = 8 + 32 + 64 = 104
        // 12dh = 8 + 128 = 136

        let p = MultiLinearPolynomial::new(
            4,
            vec![
                (Fq::from(2), vec![true, false, false, false]),
                (Fq::from(3), vec![false, true, true, false]),
                (Fq::from(6), vec![false, false, false, true]),
            ],
        )
        .unwrap();

        let q = MultiLinearPolynomial::new(
            4,
            vec![
                (Fq::from(4), vec![true, false, false, false]),
                (Fq::from(5), vec![false, true, true, false]),
                (Fq::from(2), vec![false, false, false, true]),
            ],
        )
        .unwrap();

        let pq = &p * &q;

        assert_eq!(pq.n_vars, 8);

        let mut expected_coefficients = vec![Fq::from(0); 256];
        expected_coefficients[17] = Fq::from(8);
        expected_coefficients[97] = Fq::from(10);
        expected_coefficients[129] = Fq::from(4);
        expected_coefficients[22] = Fq::from(12);
        expected_coefficients[102] = Fq::from(15);
        expected_coefficients[134] = Fq::from(6);
        expected_coefficients[24] = Fq::from(24);
        expected_coefficients[104] = Fq::from(30);
        expected_coefficients[136] = Fq::from(12);
        assert_eq!(pq.coefficients, expected_coefficients);
    }

    #[test]
    fn test_3_multilinear_multiplication() {
        // (2a + 3b) * 4c * 5d
        let p = MultiLinearPolynomial::new(
            2,
            vec![
                (Fq::from(2), vec![true, false]),
                (Fq::from(3), vec![false, true]),
            ],
        )
        .unwrap();
        let q = MultiLinearPolynomial::new(1, vec![(Fq::from(4), vec![true])]).unwrap();
        let r = MultiLinearPolynomial::new(1, vec![(Fq::from(5), vec![true])]).unwrap();

        let result = &(&p * &q) * &r;

        let mut expected_coefficients = vec![Fq::from(0); 16];
        expected_coefficients[13] = Fq::from(40);
        expected_coefficients[14] = Fq::from(60);
        assert_eq!(result.coefficients, expected_coefficients);
    }

    #[test]
    fn test_multiplicative_identity() {
        let p = poly_5ab_7bc_8d();
        let mult_identity = MultiLinearPolynomial::<Fq>::multiplicative_identity();
        let r = &p * &mult_identity;
        assert_eq!(p, r);
    }

    #[test]
    fn test_additive_identity() {
        let p = poly_5ab_7bc_8d();
        let add_identity = MultiLinearPolynomial::<Fq>::additive_identity(p.n_vars);
        let r = (&p + &add_identity).unwrap();
        assert_eq!(p, r);
    }

    #[test]
    fn test_check_zero() {
        let zero_checker = MultiLinearPolynomial::<Fq>::check_zero();
        assert_eq!(zero_checker.evaluate(&[Fq::zero()]).unwrap(), Fq::one());
        assert_eq!(zero_checker.evaluate(&[Fq::one()]).unwrap(), Fq::zero());
        assert_eq!(
            zero_checker.evaluate(&[Fq::from(5)]).unwrap(),
            Fq::from(4).neg()
        );
    }

    #[test]
    fn test_check_one() {
        let one_checker = MultiLinearPolynomial::<Fq>::check_one();
        assert_eq!(one_checker.evaluate(&[Fq::zero()]).unwrap(), Fq::zero());
        assert_eq!(one_checker.evaluate(&[Fq::one()]).unwrap(), Fq::one());
        assert_eq!(one_checker.evaluate(&[Fq::from(20)]).unwrap(), Fq::from(20));
    }

    #[test]
    fn test_lagrange_basis_polynomial() {
        // generate a poly that checks for 101 (5)
        // number of variables = 3
        let five_checker = MultiLinearPolynomial::<Fq>::lagrange_basis_poly(5, 3);
        assert_eq!(five_checker.n_vars, 3);
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![0, 0, 0])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![0, 0, 1])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![0, 1, 0])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![0, 1, 1])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![1, 0, 0])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![1, 0, 1])).unwrap(),
            Fq::one()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![1, 1, 0])).unwrap(),
            Fq::zero()
        );
        assert_eq!(
            five_checker.evaluate(&fq_from_vec(vec![1, 1, 1])).unwrap(),
            Fq::zero()
        );
    }

    #[test]
    fn test_interpolation() {
        // y = [2, 4, 8, 3]
        // p(a, b) = 2 + 6a + 2b - 7ab
        // [a, b] = [1, 2]
        let poly = MultiLinearPolynomial::<Fq>::interpolate(&fq_from_vec(vec![2, 4, 8, 3]));
        assert_eq!(poly.n_vars, 2);

        let mut expected_coefficients = vec![Fq::from(0); 4];
        expected_coefficients[0] = Fq::from(2);
        expected_coefficients[1] = Fq::from(6);
        expected_coefficients[2] = Fq::from(2);
        expected_coefficients[3] = Fq::from(7).neg();

        assert_eq!(poly.coefficients, expected_coefficients);

        // verify evaluation points
        assert_eq!(
            poly.evaluate(&fq_from_vec(vec![0, 0])).unwrap(),
            Fq::from(2)
        );
        assert_eq!(
            poly.evaluate(&fq_from_vec(vec![0, 1])).unwrap(),
            Fq::from(4)
        );
        assert_eq!(
            poly.evaluate(&fq_from_vec(vec![1, 0])).unwrap(),
            Fq::from(8)
        );
        assert_eq!(
            poly.evaluate(&fq_from_vec(vec![1, 1])).unwrap(),
            Fq::from(3)
        );
    }
}
