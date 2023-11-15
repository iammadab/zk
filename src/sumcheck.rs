use crate::multilinear_poly::{selector_from_position, MultiLinearPolynomial};
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
        // TODO: sum over the bolean hypercube
        Self {
            poly,
            sum: F::zero(),
            challenges: Vec::new(),
        }
    }

    // TODO: return univariate polynomial
    fn prove_round(&mut self, round: usize, challenge: Option<F>) -> MultiLinearPolynomial<F> {
        // if round 0 then the challenge should be None
        // if not round 0 then the challenge should be Some
        if round == 0 {
            Self::skip_first_then_sum_over_boolean_hypercube(&self.poly)
        } else {
            // store the verifier challenge
            self.challenges.push(challenge.unwrap());

            // generate partial evaluation input for stored challenges
            let challenge_assignments = self
                .challenges
                .iter()
                .enumerate()
                .map(|(i, challenge)| {
                    (selector_from_position(self.poly.n_vars(), i + 1), challenge)
                })
                .collect::<Vec<_>>();

            // partially evaluate the original poly at the challenge points
            let challenge_poly = self.poly.partial_evaluate(&challenge_assignments).unwrap();

            Self::skip_first_then_sum_over_boolean_hypercube(&challenge_poly)
        }
    }

    fn skip_first_then_sum_over_boolean_hypercube(poly: &MultiLinearPolynomial<F>) -> MultiLinearPolynomial<F> {
        // the variable names will be known and fixed
        // could create a boolean hyper cube iterator
        // we zip the var names with each value in the iter
        // then run partial eval on that zip
        // BooleanHyperCube(3) -> [0,0,0] [0,0,1]
        todo!()
    }

    fn sum_over_boolean_hypercube(poly: &MultiLinearPolynomial<F>) -> F {
        // we know how many variables before hand
        // we can iterate over 0 to 2^v
        // converting each to a set of field elements
        for i in 0..2_u32.pow(poly.n_vars() as u32) {
            // convert i to a Vec<F>
            // first convert it to binary of a certain length
            // then convert that to a vec of field elements
            // we can now evaluate based on that
            // wrap this in a fold and we are good to go
        }
        todo!()
    }
}

/// Sumcheck Verifier
struct Verifier<'a, F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    claimed_sum: &'a F,
    challenges: Vec<F>,
}

impl<'a, F: PrimeField> Verifier<'a, F> {
    fn new(poly: MultiLinearPolynomial<F>, claimed_sum: &'a F) -> Self {
        Self {
            poly,
            claimed_sum,
            challenges: Vec::new(),
        }
    }
}
