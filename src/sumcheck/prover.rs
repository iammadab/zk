use crate::multilinear_poly::{selector_from_position, MultiLinearPolynomial};
use crate::sumcheck::boolean_hypercube::BooleanHyperCube;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::PrimeField;


// TODO: implementation doesn't use fiat shamir so it doesn't enforce certain checks
//  as fiat shamir will be implemented soon (no need to do unnecessary work)
//

/// Sumcheck Prover
struct Prover<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    challenges: Vec<F>,
}

impl<F: PrimeField> Prover<F> {
    /// Instantiate a new sumcheck prover
    fn new(poly: MultiLinearPolynomial<F>) -> Self {
        let sum = sum_over_boolean_hypercube::<F>(&poly);
        Self {
            poly,
            sum,
            challenges: Vec::new(),
        }
    }

    /// Prove the nth round of the sum check protocol
    fn prove_round(&mut self, round: usize, challenge: Option<F>) -> UnivariatePolynomial<F> {
        if round == 0 {
            skip_first_var_then_sum_over_boolean_hypercube::<F>(&self.poly)
        } else {
            // store the verifier challenge
            // TODO: fix unwrap when implementing non interactive version
            self.challenges.push(challenge.unwrap());

            // generate partial evaluation input for stored challenges
            let challenge_assignments = partial_evaluation_points(
                self.poly.n_vars(),
                0..self.challenges.len(),
                self.challenges.iter(),
            );

            // partially evaluate the original poly at the challenge points
            let challenge_poly = self
                .poly
                .partial_evaluate(&challenge_assignments)
                .unwrap()
                .relabel();

            skip_first_var_then_sum_over_boolean_hypercube::<F>(&challenge_poly)
        }
    }
}

// TODO: there is an optimization that prevents you from having to do this all the rounds
//  by evaluating the polynomial from the back and caching the intermediate results
/// Keep the first variable free then sum over the boolean hypercube
/// Assumes polynomial has no unused free variables i.e poly has been relabelled
fn skip_first_var_then_sum_over_boolean_hypercube<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
) -> UnivariatePolynomial<F> {
    // evaluating at every variable other than the first one
    let n_vars_to_eval = poly.n_vars() - 1;

    let mut sum = MultiLinearPolynomial::<F>::additive_identity();

    // for each point in the boolean hypercube, perform a partial evaluation
    for point in BooleanHyperCube::<F>::new(n_vars_to_eval) {
        let evaluation_points =
            partial_evaluation_points(poly.n_vars(), 1..=n_vars_to_eval, &mut point.iter());
        let partial_eval = poly.partial_evaluate(evaluation_points.as_slice()).unwrap();
        sum = (&sum + &partial_eval).unwrap();
    }

    sum.relabel().try_into().unwrap()
}

/// Generate partial evaluation points given var positions and evaluation values as iterators
fn partial_evaluation_points<'a, F: PrimeField>(
    n_vars: usize,
    positions: impl Iterator<Item = usize>,
    evals: impl Iterator<Item = &'a F>,
) -> Vec<(Vec<bool>, &'a F)> {
    positions
        .zip(evals)
        .map(|(pos, eval)| (selector_from_position(n_vars, pos).unwrap(), eval))
        .collect()
}

/// Sum the evaluations of a polynomial over the boolean hypercube
fn sum_over_boolean_hypercube<F: PrimeField>(poly: &MultiLinearPolynomial<F>) -> F {
    let mut hypercube = BooleanHyperCube::<F>::new(poly.n_vars());
    hypercube.fold(F::zero(), |sum, point| sum + poly.evaluate(&point).unwrap())
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::prover::{
        partial_evaluation_points, skip_first_var_then_sum_over_boolean_hypercube,
        sum_over_boolean_hypercube,
    };
    use crate::univariate_poly::UnivariatePolynomial;
    use ark_ff::{Fp64, MontBackend, MontConfig, One, Zero};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    fn p_2ab_3bc() -> MultiLinearPolynomial<Fq> {
        MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_summation_over_boolean_hypercube() {
        // p = 2ab + 3bc
        // sum over boolean hypercube = 10
        let p = p_2ab_3bc();
        let sum = sum_over_boolean_hypercube::<Fq>(&p);
        assert_eq!(sum, Fq::from(10));
    }

    #[test]
    fn test_skip_first_var_sum_over_bool_hypercube() {
        // p = 2ab + 3bc
        // p(a, 0, 0) = 0
        // p(a, 0, 1) = 0
        // p(a, 1, 0) = 2a
        // p(a, 1, 1) = 2a + 3
        // sum = 4a + 3
        let p = p_2ab_3bc();
        let q = skip_first_var_then_sum_over_boolean_hypercube(&p);
        assert_eq!(q, UnivariatePolynomial::new(vec![Fq::from(3), Fq::from(4)]));

        // p = 2ab + 3bc
        // partial evaluate a = 1
        // p = 2b + 3bc
        // apply skip first then sum
        // p(b, 0) = 2b
        // p(b, 1) = 5b
        // sum = 7b
        let p = p_2ab_3bc();
        let p = p
            .partial_evaluate(&[(vec![true, false, false], &Fq::one())])
            .unwrap()
            .relabel();
        let q = skip_first_var_then_sum_over_boolean_hypercube(&p);
        assert_eq!(q, UnivariatePolynomial::new(vec![Fq::zero(), Fq::from(7)]));
    }

    #[test]
    fn test_partial_evaluation_point_generation() {
        // assume a 4 variable polynomial [a, b, c, d]
        // want to partially evaluate a and b with values 3, 4
        let eval_values = vec![Fq::from(3), Fq::from(4)];
        let evaluation_points = partial_evaluation_points(4, 0..=1, eval_values.iter());
        assert_eq!(
            evaluation_points,
            vec![
                (vec![true, false, false, false], &Fq::from(3)),
                (vec![false, true, false, false], &Fq::from(4))
            ],
        );
    }
}
