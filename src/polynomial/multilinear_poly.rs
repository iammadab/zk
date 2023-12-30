use crate::polynomial::multilinear_extension::MultiLinearExtension;
use ark_ff::{BigInteger, PrimeField};
use ark_std::iterable::Iterable;
use std::collections::BTreeMap;
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
    coefficients: BTreeMap<usize, F>,
}

// TODO: cleanup
impl<F: PrimeField> MultiLinearExtension<F> for MultiLinearPolynomial<F> {
    /// Return the number of variables in the poly
    fn n_vars(&self) -> usize {
        self.n_vars as usize
    }

    /// Assign a value to every variable in the polynomial, result is a Field element
    fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        // Associates every assignment with the correct selector vector and calls
        // partial evaluate on the expanded assignment

        if self.n_vars == 0 {
            return Ok(*self.coefficients.get(&0).unwrap_or(&F::zero()));
        }

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

        Ok(*evaluated_poly
            .coefficients
            .get(&0)
            .expect("full evaluation returns a constant"))
    }

    /// Partially assign values to variables in the polynomial
    /// Returns the resulting polynomial once those variables have been fixed
    fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str> {
        // When partially evaluating a variable in a monomial, we need to multiply the variable assignment
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
                // update only if there is an associated coefficient for this variable index
                if let Some(old_coeff) = evaluated_polynomial.coefficients.remove(&i) {
                    let result_index = i - selector_to_index(selector);
                    let updated_coefficient = old_coeff * *coeff;
                    *evaluated_polynomial
                        .coefficients
                        .entry(result_index)
                        .or_insert(F::zero()) += updated_coefficient;
                }
            }
        }
        Ok(evaluated_polynomial)
    }

    /// Relabelling removes variables that are no longer used (shrinking the polynomial)
    /// e.g. 2a + 9c uses three variables [a, b, c] but b is not represented in any term
    /// we can relabel to 2a + 9b uses 2 variables
    fn relabel(self) -> Self {
        // if the polynomial has no variable, nothing to do
        if self.n_vars == 0 {
            return self;
        }

        let variable_presence = self.variable_presence_vector();
        let mapping_instructions = mapping_instruction_from_variable_presence(&variable_presence);
        let mut relabelled_poly = remap_coefficient_keys(self.n_vars, self, mapping_instructions);
        let new_var_count = variable_presence
            .iter()
            .fold(0_u32, |acc, curr| acc + *curr as u32);
        relabelled_poly.n_vars = new_var_count;
        relabelled_poly
    }

    /// Additive identity poly
    fn additive_identity() -> Self {
        Self::new(0, vec![]).unwrap()
    }

    /// Serialize the multilinear polynomial
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.n_vars.to_be_bytes());
        for (var_id, coeff) in self.coefficients() {
            result.extend(var_id.to_be_bytes());
            result.extend(coeff.into_bigint().to_bytes_be());
        }
        result
    }
}

impl<F: PrimeField> MultiLinearPolynomial<F> {
    /// Instantiate a new Multilinear polynomial, from polynomial terms
    pub fn new(
        number_of_variables: u32,
        terms: Vec<PolynomialTerm<F>>,
    ) -> Result<Self, &'static str> {
        let mut coefficients = BTreeMap::new();
        for term in terms {
            if term.1.len() != number_of_variables as usize {
                return Err("the selector array len should be the same as the number of variables");
            }
            *coefficients
                .entry(selector_to_index(&term.1))
                .or_insert(F::zero()) += term.0;
        }
        Ok(Self {
            n_vars: number_of_variables,
            coefficients,
        })
    }

    /// Instantiate Multilinear polynomial directly from coefficients
    pub fn new_with_coefficient(
        number_of_variables: u32,
        coefficients: BTreeMap<usize, F>,
    ) -> Result<Self, &'static str> {
        if let Some((largest_key, value)) = coefficients.last_key_value() {
            if largest_key >= &Self::variable_combination_count(number_of_variables) {
                return Err("coefficient map represents more than specificed number of variables");
            }
        }

        Ok(Self {
            n_vars: number_of_variables,
            coefficients,
        })
    }

    /// Return the coefficent map of the polynomial
    pub fn coefficients(&self) -> BTreeMap<usize, F> {
        self.coefficients.clone()
    }

    /// Interpolate a set of values over the boolean hypercube
    pub fn interpolate(values: &[F]) -> Self {
        // if no points to interpolate, return zero poly
        if values.is_empty() {
            return Self::new(0, vec![]).unwrap();
        }

        let num_of_variables = bit_count_for_n_elem(values.len());

        let mut result = Self::additive_identity();
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
        Self::bit_string_checker(binary_value)
    }

    /// Given some bit string of len n e.g. 0100
    /// constructs an n-var multilinear polynomial that evaluates to 1
    /// when the given bit string is given as input
    /// and evaluates to 0 for another bit string
    pub fn bit_string_checker(bit_string: String) -> Self {
        bit_string
            .chars()
            .fold(Self::multiplicative_identity(), |acc, char| {
                if char == '1' {
                    &acc * &Self::check_one()
                } else {
                    &acc * &Self::check_zero()
                }
            })
    }

    /// Determines which variables are represented in the polynomial
    /// e.g. a polynomial of 3 variables should have [a, b, c]
    /// if the poly is of the form 3a + 4c then it only represents [a, c]
    /// the presence vector will be [true, false, true]
    fn variable_presence_vector(&self) -> Vec<bool> {
        self.coefficients
            .keys()
            .fold(vec![false; self.n_vars as usize], |acc, key| {
                let current_bool_rep = selector_from_usize(*key, self.n_vars as usize);
                acc.into_iter()
                    .zip(current_bool_rep.into_iter())
                    .map(|(a, b)| a | b)
                    .collect()
            })
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

    /// Co-efficient wise multiplication with scalar
    pub fn scalar_multiply(&self, scalar: &F) -> Self {
        // TODO: consider inplace operations
        let mut updated_coefficients = self
            .coefficients
            .clone()
            .into_iter()
            .map(|(index, coeff)| (index, coeff * scalar))
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
        1 << number_of_variables
    }

    /// Multiplicative identity poly
    fn multiplicative_identity() -> Self {
        Self::new(0, vec![(F::one(), vec![])]).unwrap()
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

        for (index, coeff) in shorter_coeff.iter() {
            *longer_coeff.entry(*index).or_insert(F::zero()) += coeff;
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
            return rhs.scalar_multiply(&self.coefficients.get(&0).unwrap_or(&F::zero()));
        } else if rhs.n_vars == 0 {
            return self.scalar_multiply(&rhs.coefficients.get(&0).unwrap_or(&F::zero()));
        };

        // It is assumed that both lhs and rhs don't share common variables
        // if they did then this multiplication will be multivariate
        // the resulting polynomial number of variables is the sum of the lhs and rhs n_vars
        let mut new_poly_coefficients = BTreeMap::new();

        for (i, self_coeff) in self.coefficients.iter() {
            for (j, rhs_coeff) in rhs.coefficients.iter() {
                if self_coeff.is_zero() || rhs_coeff.is_zero() {
                    continue;
                }

                let new_coefficient = *self_coeff * rhs_coeff;
                let mut left_index_vec = selector_from_usize(*i, self.n_vars as usize);
                let mut right_index_vec = selector_from_usize(*j, rhs.n_vars as usize);
                left_index_vec.append(&mut right_index_vec);

                let result_index = selector_to_index(&left_index_vec);
                *new_poly_coefficients
                    .entry(result_index)
                    .or_insert(F::zero()) += new_coefficient;
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
pub fn selector_from_usize(value: usize, exact_size: usize) -> Vec<bool> {
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
    for _ in 0..(exact_size - binary_value.len()) {
        result.push(false);
    }
    result
}

// TODO: move to until file
/// Returns a Vec<bool> of a given size, with default value set to false, except the position index
pub fn selector_from_position(size: usize, position: usize) -> Result<Vec<bool>, &'static str> {
    if position > size - 1 {
        return Err("position index out of bounds");
    }

    let mut selector = vec![false; size];
    selector[position] = true;
    Ok(selector)
}

/// Convert a number to a binary string of a given size
pub fn binary_string(index: usize, bit_count: usize) -> String {
    let binary = format!("{:b}", index);
    "0".repeat(bit_count.checked_sub(binary.len()).unwrap_or(0)) + &binary
}

/// Generate remapping instruction for truncating a presence vector
/// e.g. [t, f, t] t at index 2 should be pushed to index 1 as that contains false
/// so mapping = [(2, 1)]
fn mapping_instruction_from_variable_presence(
    variable_presence_vector: &[bool],
) -> Vec<(usize, usize)> {
    let mut next_var = 0;
    let mut mapping_vector = vec![];
    for (index, is_present) in variable_presence_vector.iter().enumerate() {
        if *is_present {
            if next_var != index {
                mapping_vector.push((index, next_var));
            }
            next_var += 1;
        }
    }
    mapping_vector
}

/// Converts mapping instruction to powers of 2
fn to_power_of_two(instruction: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    instruction
        .into_iter()
        .map(|(a, b)| (2_usize.pow(a as u32), 2_usize.pow(b as u32)))
        .collect()
}

/// Use remapping instructions to remap a polynomial coefficients
fn remap_coefficient_keys<F: PrimeField>(
    n_vars: u32,
    mut poly: MultiLinearPolynomial<F>,
    mapping_instructions: Vec<(usize, usize)>,
) -> MultiLinearPolynomial<F> {
    let mapping_instruction_as_powers_of_2 = to_power_of_two(mapping_instructions);
    for (old_var, new_var) in mapping_instruction_as_powers_of_2 {
        let old_var_indexes = MultiLinearPolynomial::<F>::get_variable_indexes(
            n_vars,
            &selector_from_usize(old_var, n_vars as usize),
        )
        .unwrap();
        for index in old_var_indexes {
            if let Some(coeff) = poly.coefficients.remove(&index) {
                let new_index = index - old_var + new_var;
                *poly.coefficients.entry(new_index).or_insert(F::zero()) += coeff
            }
        }
    }
    poly
}

/// Determines the number of bits needed to represent a number
pub fn bit_count_for_n_elem(size: usize) -> usize {
    format!("{:b}", size - 1).len()
}

#[cfg(test)]
mod tests {
    use crate::polynomial::multilinear_extension::MultiLinearExtension;
    use crate::polynomial::multilinear_poly::{
        mapping_instruction_from_variable_presence, remap_coefficient_keys, selector_to_index,
        to_power_of_two, MultiLinearPolynomial,
    };
    use ark_ff::{Fp64, MontBackend, MontConfig, One, Zero};
    use std::collections::BTreeMap;
    use std::ops::Neg;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    // TODO: move this functionality into the polynomial struct
    // TODO: make this generic over field
    fn fq_from_vec(values: Vec<i64>) -> Vec<Fq> {
        values.into_iter().map(Fq::from).collect()
    }

    fn fq_map_from_vec(values: Vec<i64>) -> BTreeMap<usize, Fq> {
        let mut result = BTreeMap::new();
        for (index, val) in values.into_iter().enumerate() {
            if val != 0 {
                result.insert(index, Fq::from(val));
            }
        }
        result
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
            BTreeMap::from([(3, Fq::from(2))]) // vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)]
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
            BTreeMap::from([(1, Fq::from(2)), (2, Fq::from(3)), (3, Fq::from(5))])
        );

        // constant = 5
        // expected dense form = [5, 0, 0, 0]
        assert_eq!(
            MultiLinearPolynomial::new(2, vec![(Fq::from(5), vec![false, false])])
                .unwrap()
                .coefficients,
            BTreeMap::from([(0, Fq::from(5))])
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
            BTreeMap::from([(2, Fq::from(4)), (3, Fq::from(5))])
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
            fq_map_from_vec(vec![13, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0])
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
            fq_map_from_vec(vec![4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0])
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
            fq_map_from_vec(vec![11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
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
            fq_map_from_vec(vec![11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
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
    fn test_evaluation_with_more_than_n_points() {
        // p has 4 variables, but passing 5
        let p = poly_5ab_7bc_8d();
        assert_eq!(
            p.evaluate(&[
                Fq::from(2),
                Fq::from(3),
                Fq::from(4),
                Fq::from(5),
                Fq::from(6),
            ])
            .unwrap(),
            Fq::from(11)
        );
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
            fq_map_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
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
            fq_map_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
        );

        // scalar mul with two polynomials
        let p = poly_5ab_7bc_8d();
        let scalar_poly = MultiLinearPolynomial::new(0, vec![(Fq::from(2), vec![])]).unwrap();
        let two_p = &p * &scalar_poly;
        assert_eq!(
            two_p.coefficients,
            fq_map_from_vec(vec![0, 0, 0, 10, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0, 0, 0])
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
        assert_eq!(
            pq.coefficients,
            fq_map_from_vec(vec![0, 0, 0, 0, 0, 0, 0, 30])
        );

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

        let mut expected_coefficients = vec![0; 32];
        // [a, b, c, d, e] = [1, 2, 4, 8, 16]
        // set 14abde = 1 + 2 + 8 + 16 = 27
        // set 21acde = 1 + 4 + 8 + 16 = 29
        expected_coefficients[27] = 14;
        expected_coefficients[29] = 21;

        assert_eq!(pq.coefficients, fq_map_from_vec(expected_coefficients));
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

        let mut expected_coefficients = vec![0; 256];
        expected_coefficients[17] = 8;
        expected_coefficients[97] = 10;
        expected_coefficients[129] = 4;
        expected_coefficients[22] = 12;
        expected_coefficients[102] = 15;
        expected_coefficients[134] = 6;
        expected_coefficients[24] = 24;
        expected_coefficients[104] = 30;
        expected_coefficients[136] = 12;
        assert_eq!(pq.coefficients, fq_map_from_vec(expected_coefficients));
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

        let mut expected_coefficients = vec![0; 16];
        expected_coefficients[13] = 40;
        expected_coefficients[14] = 60;
        assert_eq!(result.coefficients, fq_map_from_vec(expected_coefficients));
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
        let add_identity = MultiLinearPolynomial::<Fq>::additive_identity();
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

        let mut expected_coefficients = vec![0; 4];
        expected_coefficients[0] = 2;
        expected_coefficients[1] = 6;
        expected_coefficients[2] = 2;
        expected_coefficients[3] = -7;

        assert_eq!(poly.coefficients, fq_map_from_vec(expected_coefficients));

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

    #[test]
    fn test_variable_presence_vector() {
        // p = 3a + 2c
        // number of variables at creation = 3
        // actual number of variables is 2 as b is not represented
        let poly = MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(3), vec![true, false, false]),
                (Fq::from(2), vec![false, false, true]),
            ],
        )
        .unwrap();

        assert_eq!(poly.variable_presence_vector(), vec![true, false, true]);
    }

    #[test]
    fn test_mapping_instruction_from_variable_presence() {
        assert_eq!(
            mapping_instruction_from_variable_presence(&[true, false, false, true]),
            vec![(3, 1)]
        );

        assert_eq!(
            mapping_instruction_from_variable_presence(&[true, false, false, true, true]),
            vec![(3, 1), (4, 2)]
        );

        assert_eq!(
            mapping_instruction_from_variable_presence(&[false, false, true, true]),
            vec![(2, 0), (3, 1)]
        );

        assert_eq!(
            mapping_instruction_from_variable_presence(&[true, true]),
            vec![]
        );

        assert_eq!(
            mapping_instruction_from_variable_presence(&[false, false]),
            vec![]
        );

        assert_eq!(
            to_power_of_two(mapping_instruction_from_variable_presence(&[
                false, true, false, false, true, false
            ])),
            vec![(2, 1), (16, 2)]
        );
    }

    #[test]
    fn test_poly_relabelling() {
        // poly of 4 variables, [a, b, c, d] -> [1, 2, 4, 8]
        // p = 2ab + 3cd + 5acd + 6bd
        // partial evaluate at b = 1 and c = 1
        // q = 2a(1) + 3(1)d + 5a(1)d + 6(1)d
        // q = 2a + 3d + 5ad + 6d = 2a + 9d + 5ad
        // after relabelling (d -> b)
        // q = 2a + 9b + 5ab

        let p = MultiLinearPolynomial::new(
            4,
            vec![
                (Fq::from(2), vec![true, true, false, false]),
                (Fq::from(3), vec![false, false, true, true]),
                (Fq::from(5), vec![true, false, true, true]),
                (Fq::from(6), vec![false, true, false, true]),
            ],
        )
        .unwrap();

        // partial eval b = 1 c = 1
        // q = 2a + 9d + 5ad
        let q = p
            .partial_evaluate(&[
                (vec![false, true, false, false], &Fq::one()),
                (vec![false, false, true, false], &Fq::one()),
            ])
            .unwrap();

        assert_eq!(q.n_vars, 4);
        assert_eq!(
            q.coefficients,
            BTreeMap::from([(1, Fq::from(2)), (8, Fq::from(9)), (9, Fq::from(5)),])
        );

        // next we relabel
        // that changes d to b
        // so 2a + 9d + 5ad changes to 2a + 9b + 5ab
        let q = q.relabel();
        assert_eq!(q.n_vars, 2);
        assert_eq!(
            q.coefficients,
            BTreeMap::from([(1, Fq::from(2)), (2, Fq::from(9)), (3, Fq::from(5)),])
        );

        // constant polynomial
        // relabel should have no effect
        let poly = MultiLinearPolynomial::<Fq>::multiplicative_identity();
        let poly = poly.relabel();
        assert_eq!(poly, MultiLinearPolynomial::<Fq>::multiplicative_identity());
    }

    #[test]
    fn test_bit_string_checker() {
        // poly to check 001
        let checker = MultiLinearPolynomial::<Fq>::bit_string_checker("001".to_string());
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![0, 0, 0])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![0, 0, 1])).unwrap(),
            Fq::from(1)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![0, 1, 0])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![0, 1, 1])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![1, 0, 0])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![1, 0, 1])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![1, 1, 0])).unwrap(),
            Fq::from(0)
        );
        assert_eq!(
            checker.evaluate(&fq_from_vec(vec![1, 1, 1])).unwrap(),
            Fq::from(0)
        );
    }

    #[test]
    fn test_evaluate_zero_poly() {
        let zero_poly = MultiLinearPolynomial::<Fq>::additive_identity();
        assert_eq!(zero_poly.evaluate(&[]).unwrap(), Fq::from(0));
    }
}
