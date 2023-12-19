use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::{selector_from_position, MultiLinearPolynomial};
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use crate::sumcheck::boolean_hypercube::BooleanHyperCube;
use crate::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};
use std::ops::Add;

// TODO: there is an optimization that prevents you from having to do this all the rounds
//  by evaluating the polynomial from the back and caching the intermediate results
/// Keep the first variable free then sum over the boolean hypercube
/// Assumes polynomial has no unused free variables i.e poly has been relabelled
pub fn skip_first_var_then_sum_over_boolean_hypercube<F: PrimeField, P: MultiLinearExtension<F>>(
    poly: P,
) -> P
where
    for<'a> &'a P: Add<Output = Result<P, &'static str>>,
{
    // evaluating at every variable other than the first one
    let n_vars_to_eval = poly.n_vars() - 1;

    if n_vars_to_eval == 0 {
        // only one variable is free
        return poly.clone();
    }

    let mut sum = P::additive_identity();

    // for each point in the boolean hypercube, perform a partial evaluation
    for point in BooleanHyperCube::<F>::new(n_vars_to_eval) {
        let evaluation_points =
            partial_evaluation_points(poly.n_vars(), 1..=n_vars_to_eval, &mut point.iter());
        let partial_eval = poly.partial_evaluate(evaluation_points.as_slice()).unwrap();
        sum = (&sum + &partial_eval).unwrap();
    }

    // TODO: do we check that this has just one variable here??
    sum.relabel()
}

/// Sum a polynomial over the boolean hypercube
pub fn sum_over_boolean_hyper_cube<F: PrimeField, P: MultiLinearExtension<F>>(poly: &P) -> F {
    BooleanHyperCube::<F>::new(poly.n_vars()).fold(F::zero(), |sum, point| {
        sum + poly.evaluate(point.as_slice()).unwrap()
    })
}

/// Generate partial evaluation points given var positions and evaluation values as iterators
pub fn partial_evaluation_points<'a, F: PrimeField>(
    n_vars: usize,
    positions: impl Iterator<Item = usize>,
    evals: impl Iterator<Item = &'a F>,
) -> Vec<(Vec<bool>, &'a F)> {
    positions
        .zip(evals)
        .map(|(pos, eval)| (selector_from_position(n_vars, pos).unwrap(), eval))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::util::sum_over_boolean_hyper_cube;
    use ark_ff::{Fp64, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_sum_over_boolean_hypercube() {
        let poly = MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap();

        // expected sum = 10
        let sum = sum_over_boolean_hyper_cube(&poly);
        assert_eq!(sum, Fq::from(10));
    }
}
