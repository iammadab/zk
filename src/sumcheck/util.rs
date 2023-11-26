use crate::multilinear_poly::{selector_from_position, MultiLinearPolynomial};
use crate::sumcheck::boolean_hypercube::BooleanHyperCube;
use crate::transcript::Transcript;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::{BigInteger, PrimeField};

// TODO: there is an optimization that prevents you from having to do this all the rounds
//  by evaluating the polynomial from the back and caching the intermediate results
/// Keep the first variable free then sum over the boolean hypercube
/// Assumes polynomial has no unused free variables i.e poly has been relabelled
pub fn skip_first_var_then_sum_over_boolean_hypercube<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
) -> UnivariatePolynomial<F> {
    // evaluating at every variable other than the first one
    let n_vars_to_eval = poly.n_vars() - 1;

    if n_vars_to_eval == 0 {
        // only one variable is free
        return poly.clone().try_into().unwrap();
    }

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

/// Add a multilinear polynomial to a transcript object
pub fn add_multilinear_poly_to_transcript<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
    transcript: &mut Transcript,
) {
    transcript.append(&poly.n_vars().to_be_bytes());
    for (var_id, coeff) in poly.coefficients() {
        transcript.append(&var_id.to_be_bytes());
        transcript.append(&coeff.into_bigint().to_bytes_be().as_slice());
    }
}

/// Add a univariate polynomial to a transcript object
pub fn add_univariate_poly_to_transcript<F: PrimeField>(
    poly: &UnivariatePolynomial<F>,
    transcript: &mut Transcript,
) {
    for coeff in poly.coefficients() {
        transcript.append(coeff.into_bigint().to_bytes_be().as_slice())
    }
}
