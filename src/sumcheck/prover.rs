use crate::multilinear_poly::{selector_from_position, MultiLinearPolynomial};
use crate::sumcheck::boolean_hypercube::BooleanHyperCube;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::PrimeField;

/// Sumcheck Prover
struct Prover<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    challenges: Vec<F>,
}

impl<F: PrimeField> Prover<F> {
    fn new(poly: MultiLinearPolynomial<F>) -> Self {
        let sum = sum_over_boolean_hypercube::<F>(&poly);
        Self {
            poly,
            sum,
            challenges: Vec::new(),
        }
    }

    // TODO: return univariate polynomial
    fn prove_round(&mut self, round: usize, challenge: Option<F>) -> MultiLinearPolynomial<F> {
        if round == 0 {
            skip_first_var_then_sum_over_boolean_hypercube::<F>(&self.poly)
        } else {
            // store the verifier challenge
            // TODO: fix unwrap when implementing non interactive version
            self.challenges.push(challenge.unwrap());

            // generate partial evaluation input for stored challenges
            // TODO: abstract this
            let challenge_assignments = self
                .challenges
                .iter()
                .enumerate()
                .map(|(i, challenge)| {
                    (
                        selector_from_position(self.poly.n_vars(), i + 1).unwrap(),
                        challenge,
                    )
                })
                .collect::<Vec<_>>();

            // partially evaluate the original poly at the challenge points
            let challenge_poly = self.poly.partial_evaluate(&challenge_assignments).unwrap();

            skip_first_var_then_sum_over_boolean_hypercube::<F>(&challenge_poly)
        }
    }
}

/// Keep the first variable free then sum over the boolean hypercube
fn skip_first_var_then_sum_over_boolean_hypercube<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
) -> MultiLinearPolynomial<F> {
    // the variable names will be known and fixed
    // could create a boolean hyper cube iterator
    // we zip the var names with each value in the iter
    // then run partial eval on that zip
    // BooleanHyperCube(3) -> [0,0,0] [0,0,1]
    todo!()
}

/// Sum the evaluations of a polynomial over the boolean hypercube
fn sum_over_boolean_hypercube<F: PrimeField>(poly: &MultiLinearPolynomial<F>) -> F {
    let mut hypercube = BooleanHyperCube::<F>::new(poly.n_vars());
    hypercube.fold(F::zero(), |sum, point| sum + poly.evaluate(&point).unwrap())
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::prover::sum_over_boolean_hypercube;
    use ark_ff::{Fp64, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_summation_over_boolean_hypercube() {
        // p = 2ab + 3bc
        // sum over boolean hypercube = 10
        let p = MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap();
        let sum = sum_over_boolean_hypercube::<Fq>(&p);
        assert_eq!(sum, Fq::from(10));
    }
}
