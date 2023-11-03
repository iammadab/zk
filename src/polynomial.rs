use ark_ff::PrimeField;
use std::ops;

#[derive(Debug, PartialEq)]
pub struct Polynomial<F: PrimeField> {
    /// Dense co-efficient representation of the polynomial
    /// lower degree co-efficients to higher degree co-efficients
    coefficients: Vec<F>,
}

impl<F: PrimeField> Polynomial<F> {
    /// Instantiate a new polynomial
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    // TODO: implement method to simplify coefficients by truncation
    //  e.g. [0, 2, 0, 0] is equivalent to [0, 2]

    /// Evaluate polynomial at a given point x
    pub fn evaluate(&self, x: &F) -> F {
        // naive implementation
        // TODO: apply distributive law to see if things are faster (do benchmarks first)
        self.coefficients
            .iter()
            .enumerate()
            .fold(F::zero(), |acc, (exp, coeff)| {
                acc + x.pow(&[exp as u64]) * coeff
            })
    }

    /// returns a new polynomial that interpolates all the given points
    // TODO: prevent duplication in the x values (use a new type)
    fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        let mut result = Polynomial::new(vec![]);

        for (lagrange_basis_index, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
            let mut lagrange_basis = Polynomial::new(vec![F::from(1_u8)]);

            // compute the lagrange basis polynomial
            for (x_index, x_value) in xs.iter().enumerate() {
                if x_index == lagrange_basis_index {
                    continue;
                }

                // numerator = x -xs[i] where i != lagrange_basis_index
                let numerator = Polynomial::new(vec![-x_value.clone(), F::from(1_u8)]);
                let denominator = (*x - x_value).inverse().unwrap();

                lagrange_basis =
                    &lagrange_basis * &(&numerator * &Polynomial::new(vec![denominator]));
            }

            let monomial = &lagrange_basis * &Polynomial::new(vec![*y]);
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
        return if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        };
    }
}

impl<F: PrimeField> ops::Add for &Polynomial<F> {
    type Output = Polynomial<F>;

    fn add(self, other: Self) -> Self::Output {
        // TODO: improve implementation
        if self.is_zero() {
            return Polynomial::new(other.coefficients.clone());
        }

        if other.is_zero() {
            return Polynomial::new(self.coefficients.clone());
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

        Polynomial::new(new_coefficients)
    }
}

impl<F: PrimeField> ops::Mul for &Polynomial<F> {
    type Output = Polynomial<F>;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return Polynomial::new(vec![]);
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

        Polynomial::new(product_coefficients)
    }
}

#[cfg(test)]
mod tests {
    use super::Polynomial;
    use ark_ff::MontConfig;
    use ark_ff::{Fp64, MontBackend, PrimeField};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

    fn fq_from_vec(values: Vec<i64>) -> Vec<Fq> {
        values.into_iter().map(Fq::from).collect()
    }

    fn poly_from_vec(coefficients: Vec<i64>) -> Polynomial<Fq> {
        Polynomial::new(fq_from_vec(coefficients))
    }

    fn poly_zero() -> Polynomial<Fq> {
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
        let p = Polynomial::interpolate(fq_from_vec(vec![0, 1]), fq_from_vec(vec![0, 2]));
        assert_eq!(p, poly_from_vec(vec![0, 2]));

        // p = 2x^2 + 5
        // evaluations = [(0, 5), (1, 7), (2, 13)]
        let p = Polynomial::interpolate(fq_from_vec(vec![0, 1, 2]), fq_from_vec(vec![5, 7, 13]));
        assert_eq!(p, poly_from_vec(vec![5, 0, 2]));

        // p = 8x^5 + 12x^4 + 7x^3 + 1x^2 + 8x + 12
        let p = Polynomial::interpolate(
            fq_from_vec(vec![0, 1, 3, 4, 5, 8]),
            fq_from_vec(vec![12, 48, 3150, 11772, 33452, 315020]),
        );
        assert_eq!(p, poly_from_vec(vec![12, 25, 18, 24, 12, 8]));

        // p = 5x^3 - 12x
        let p = Polynomial::interpolate(
            fq_from_vec(vec![5, 7, 9, 1]),
            fq_from_vec(vec![565, 1631, 3537, -7]),
        );
        assert_eq!(p, poly_from_vec(vec![0, -12, 0, 5]));
    }
}
