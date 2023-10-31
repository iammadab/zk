use ark_ff::PrimeField;

#[derive(Debug)]
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

    #[test]
    fn test_evaluation() {
        let p = Polynomial::new(vec![Fq::from(0), Fq::from(2)]);

        // p = 2x
        // x = 4
        // expected result: 8
        assert_eq!(p.evaluate(&Fq::from(4)), Fq::from(8));
    }
}
