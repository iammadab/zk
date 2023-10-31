use ark_ff::PrimeField;

#[derive(Debug, PartialEq)]
pub struct Polynomial<F: PrimeField> {
    /// Dense co-efficient representation of the polynomial
    /// lower degree co-efficients to higher degree co-efficients
    coefficients: Vec<F>,
}

impl<F: PrimeField> Polynomial<F> {
    /// Instantiate a new polynomial
    fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    /// Evaluate polynomial at a given point x
    fn evaluate(&self, x: &F) -> F {
        // naive implementation
        // TODO: apply distributive law to see if things are faster (do benchmarks first)
        self.coefficients
            .iter()
            .enumerate()
            .fold(F::zero(), |acc, (exp, coeff)| {
                acc + x.pow(&[exp as u64]) * coeff
            })
    }

    // TODO: implement the rust add
    /// Add two polynomials in dense format
    fn add(&self, other: &Self) -> Self {
        // TODO: improve implementation
        if self.coefficients.is_empty() {
            return Self::new(other.coefficients.clone());
        }

        if other.coefficients.is_empty() {
            return Self::new(self.coefficients.clone());
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

        Self::new(new_coefficients)
    }
}

#[cfg(test)]
mod tests {
    use crate::polynomial::Polynomial;
    use ark_ff::MontConfig;
    use ark_ff::{Fp64, MontBackend, PrimeField};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

    fn poly_from_vec(coefficients: Vec<u64>) -> Polynomial<Fq> {
        Polynomial::new(coefficients.into_iter().map(Fq::from).collect())
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
        assert_eq!(poly_zero().add(&poly_zero()), poly_zero());

        // if either polynomial is zero, return the other polynomial
        assert_eq!(
            poly_zero().add(&poly_from_vec(vec![0, 2])),
            poly_from_vec(vec![0, 2])
        );
        assert_eq!(
            poly_from_vec(vec![0, 2]).add(&poly_zero()),
            poly_from_vec(vec![0, 2])
        );

        // p = 2x^2 + 3x + 4
        // q = 4x^3 + 4x + 3
        // p + q = 4x^3 + 2x^2 + 7x + 7
        let p = poly_from_vec(vec![4, 3, 2]);
        let q = poly_from_vec(vec![3, 4, 0, 4]);
        let p_plus_q = p.add(&q);
        let q_plus_p = q.add(&p);

        // should be commutative
        assert_eq!(p_plus_q, q_plus_p);
        // should sum to expected value
        assert_eq!(p_plus_q, poly_from_vec(vec![7, 7, 2, 4]));
    }
}
