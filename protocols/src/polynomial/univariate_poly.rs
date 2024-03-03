use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::ops;

#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct UnivariatePolynomial<F: PrimeField> {
    /// Dense co-efficient representation of the polynomial
    /// lower degree co-efficients to higher degree co-efficients
    coefficients: Vec<F>,
}

impl<F: PrimeField> UnivariatePolynomial<F> {
    /// Instantiate a new polynomial
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    /// Returns the list of coefficients as a slice
    pub fn coefficients(&self) -> &[F] {
        self.coefficients.as_slice()
    }

    // TODO: implement method to simplify coefficients by truncation
    //  e.g. [0, 2, 0, 0] is equivalent to [0, 2]

    /// Evaluate polynomial at a given point x
    pub fn evaluate(&self, x: &F) -> F {
        // 5 + 2x + 3x^2
        // can be evaluated as
        // ((3 * x + 2) * x) + 5 = 3x^2 + 2x + 5 -> 2 multiplications
        // rather than
        // 5 + 2 * x + 3 * x * x -> 3 multiplications
        // 2 mul instead of 3 and this scales with the degree
        self.coefficients
            .iter()
            .rev()
            .fold(F::zero(), |acc, coeff| acc * x + coeff)
    }

    /// Interpolate a set of y values over the interpolating set [0, 1, 2, ...]
    pub fn interpolate(ys: Vec<F>) -> Self {
        let mut xs = vec![];
        for i in 0..ys.len() {
            xs.push(F::from(i as u64));
        }
        Self::interpolate_xy(xs, ys)
    }

    /// returns a new polynomial that interpolates all the given points
    // TODO: prevent duplication in the x values (use a new type)
    // TODO: use new type to prevent x and y from being of different lengths
    pub fn interpolate_xy(xs: Vec<F>, ys: Vec<F>) -> Self {
        let mut result = UnivariatePolynomial::new(vec![]);

        for (lagrange_basis_index, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
            let mut lagrange_basis = UnivariatePolynomial::new(vec![F::from(1_u8)]);

            // compute the lagrange basis polynomial
            for (x_index, x_value) in xs.iter().enumerate() {
                if x_index == lagrange_basis_index {
                    continue;
                }

                // numerator = x -xs[i] where i != lagrange_basis_index
                let numerator = UnivariatePolynomial::new(vec![-x_value.clone(), F::from(1_u8)]);
                let denominator = (*x - x_value).inverse().unwrap();

                lagrange_basis =
                    &lagrange_basis * &(&numerator * &UnivariatePolynomial::new(vec![denominator]));
            }

            let monomial = &lagrange_basis * &UnivariatePolynomial::new(vec![*y]);
            // TODO: implement add assign
            result = &result + &monomial;
        }

        result
    }

    /// return true if polynomial is a zero poly i.e p(..) = 0
    fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    /// return the degree of a polynomial
    fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        }
    }

    /// Additive identity poly
    pub fn additive_identity() -> Self {
        Self::new(vec![])
    }

    /// Multiplicative identity poly
    pub fn multiplicative_identity() -> Self {
        Self::new(vec![F::one()])
    }

    /// Serialize the univariate polynomial
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        for coeff in self.coefficients() {
            result.extend(coeff.into_bigint().to_bytes_be());
        }
        result
    }
}

impl<F: PrimeField> ops::Add for &UnivariatePolynomial<F> {
    type Output = UnivariatePolynomial<F>;

    fn add(self, other: Self) -> Self::Output {
        // TODO: improve implementation
        if self.is_zero() {
            return other.clone();
            // return UnivariatePolynomial::new(other.coefficients.clone());
        }

        if other.is_zero() {
            return UnivariatePolynomial::new(self.coefficients.clone());
        }

        let (mut new_coefficients, other_coeff) =
            if self.coefficients.len() >= other.coefficients.len() {
                (self.coefficients.clone(), &other.coefficients)
            } else {
                (other.coefficients.clone(), &self.coefficients)
            };

        for i in 0..other_coeff.len() {
            new_coefficients[i] += other_coeff[i];
        }

        UnivariatePolynomial::new(new_coefficients)
    }
}

impl<F: PrimeField> ops::Mul for &UnivariatePolynomial<F> {
    type Output = UnivariatePolynomial<F>;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return UnivariatePolynomial::new(vec![]);
        }

        // Given 2 polynomials A, B of degree a, b respectively
        // the product polynomial C = AB will have max degree of a + b
        let product_max_degree = self.degree() + other.degree();

        // we need d + 1 element to represent a polynomial of degree d
        let mut product_coefficients = vec![F::zero(); product_max_degree + 1];

        for i in 0..=self.degree() {
            for j in 0..=other.degree() {
                product_coefficients[i + j] += self.coefficients[i] * other.coefficients[j];
            }
        }

        UnivariatePolynomial::new(product_coefficients)
    }
}

impl<F: PrimeField> TryFrom<MultiLinearPolynomial<F>> for UnivariatePolynomial<F> {
    type Error = &'static str;

    fn try_from(value: MultiLinearPolynomial<F>) -> Result<Self, Self::Error> {
        if value.n_vars() > 1 {
            return Err("cannot convert multilinear polynomial with more than one variable to univariate poly");
        }

        let coefficients = value.coefficients();

        // TODO: might need to relabel the poly before getting the coefficients
        Ok(UnivariatePolynomial::new(vec![
            *coefficients.get(&0).unwrap_or(&F::zero()),
            *coefficients.get(&1).unwrap_or(&F::zero()),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::UnivariatePolynomial;
    use crate::polynomial::multilinear_extension::MultiLinearExtension;
    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use ark_ff::MontConfig;
    use ark_ff::{Fp64, MontBackend};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    // TODO: move this functionality into the polynomial struct
    fn fq_from_vec(values: Vec<i64>) -> Vec<Fq> {
        values.into_iter().map(Fq::from).collect()
    }

    fn poly_from_vec(coefficients: Vec<i64>) -> UnivariatePolynomial<Fq> {
        UnivariatePolynomial::new(fq_from_vec(coefficients))
    }

    fn poly_zero() -> UnivariatePolynomial<Fq> {
        poly_from_vec(vec![])
    }

    #[test]
    fn test_evaluation() {
        // p = 2x
        // x = 4
        // expected result: 8
        let p = poly_from_vec(vec![0, 2]);
        assert_eq!(p.evaluate(&Fq::from(4)), Fq::from(8));
    }

    #[test]
    fn test_polynomial_addition() {
        // both polynomials are zero polynomials
        assert_eq!(&poly_zero() + &poly_zero(), poly_zero());

        // if either polynomial is zero, return the other polynomial
        assert_eq!(
            &poly_zero() + &poly_from_vec(vec![0, 2]),
            poly_from_vec(vec![0, 2])
        );
        assert_eq!(
            &poly_from_vec(vec![0, 2]) + &poly_zero(),
            poly_from_vec(vec![0, 2])
        );

        // p = 2x^2 + 3x + 4
        // q = 4x^3 + 4x + 3
        // p + q = 4x^3 + 2x^2 + 7x + 7
        let p = poly_from_vec(vec![4, 3, 2]);
        let q = poly_from_vec(vec![3, 4, 0, 4]);
        let p_plus_q = &p + &q;
        let q_plus_p = &q + &p;

        // should be commutative
        assert_eq!(p_plus_q, q_plus_p);
        // should sum to expected value
        assert_eq!(p_plus_q, poly_from_vec(vec![7, 7, 2, 4]));
    }

    #[test]
    fn test_polynomial_multiplication() {
        // if either polynomial is the zero polynomial, return zero
        assert_eq!(
            &poly_zero() * &poly_from_vec(vec![0, 2]),
            poly_from_vec(vec![])
        );
        assert_eq!(
            &poly_from_vec(vec![0, 2]) * &poly_zero(),
            poly_from_vec(vec![])
        );

        // p = 2x^2 + 3x + 4
        // q = 4x^3 + 4x + 3
        // pq = 8x^5 + 12x^4 + 24x^3 + 18x^2 + 25x + 12
        // pq mod 17 = 8x^5 + 12x^4 + 7x^3 + 1x^2 + 8x + 12
        let p = poly_from_vec(vec![4, 3, 2]);
        let q = poly_from_vec(vec![3, 4, 0, 4]);
        let p_mul_q = &p * &q;
        let q_mul_p = &q * &p;

        // should be commutative
        assert_eq!(p_mul_q, q_mul_p);
        // should mul to expected value
        assert_eq!(p_mul_q, poly_from_vec(vec![12, 25, 18, 24, 12, 8]));
    }

    #[test]
    fn test_polynomial_interpolation() {
        // p = 2x
        // evaluations = [(0, 0), (1, 2)]
        let p =
            UnivariatePolynomial::interpolate_xy(fq_from_vec(vec![0, 1]), fq_from_vec(vec![0, 2]));
        assert_eq!(p, poly_from_vec(vec![0, 2]));

        // p = 2x^2 + 5
        // evaluations = [(0, 5), (1, 7), (2, 13)]
        let p = UnivariatePolynomial::interpolate_xy(
            fq_from_vec(vec![0, 1, 2]),
            fq_from_vec(vec![5, 7, 13]),
        );
        assert_eq!(p, poly_from_vec(vec![5, 0, 2]));

        // p = 8x^5 + 12x^4 + 7x^3 + 1x^2 + 8x + 12
        let p = UnivariatePolynomial::interpolate_xy(
            fq_from_vec(vec![0, 1, 3, 4, 5, 8]),
            fq_from_vec(vec![12, 48, 3150, 11772, 33452, 315020]),
        );
        assert_eq!(p, poly_from_vec(vec![12, 25, 18, 24, 12, 8]));

        // p = 5x^3 - 12x
        let p = UnivariatePolynomial::interpolate_xy(
            fq_from_vec(vec![5, 7, 9, 1]),
            fq_from_vec(vec![565, 1631, 3537, -7]),
        );
        assert_eq!(p, poly_from_vec(vec![0, -12, 0, 5]));
    }

    #[test]
    fn test_identity_poly() {
        // p = 2x
        let p = poly_from_vec(vec![0, 2]);

        // additive identity
        let additive_identity = UnivariatePolynomial::<Fq>::additive_identity();
        let p_plus_e = &p + &additive_identity;
        assert_eq!(p_plus_e, p);

        // multiplicative identity
        let multiplicative_identity = UnivariatePolynomial::<Fq>::multiplicative_identity();
        let p_mul_e = &p * &multiplicative_identity;
        assert_eq!(p_mul_e, p);
    }

    #[test]
    fn test_from_multilinear() {
        // p = 2ab + 3bc
        let p = MultiLinearPolynomial::<Fq>::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap();

        // should not be able to build uni poly from multilinear poly with 3 variables
        let uni_poly_result: Result<UnivariatePolynomial<_>, _> = p.clone().try_into();
        assert_eq!(uni_poly_result.is_err(), true);

        // partial evaluate b
        // p = 2a + 3c
        let p = p
            .partial_evaluate(&[(vec![false, true, false], &Fq::from(1))])
            .unwrap();

        // should fail, 2 variables
        let uni_poly_result: Result<UnivariatePolynomial<_>, _> = p.clone().try_into();
        assert_eq!(uni_poly_result.is_err(), true);

        // Partial evaluate a
        // p = 2 + 3c
        let p = p
            .partial_evaluate(&[(vec![true, false, false], &Fq::from(1))])
            .unwrap()
            .relabel();

        // should be successful, p has just 1 variable
        let uni_poly_result: Result<UnivariatePolynomial<_>, _> = p.try_into();
        assert_eq!(uni_poly_result.is_err(), false);
        let uni_poly = uni_poly_result.unwrap();
        assert_eq!(uni_poly, poly_from_vec(vec![2, 3]));
    }
}
