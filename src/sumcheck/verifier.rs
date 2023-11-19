use crate::multilinear_poly::MultiLinearPolynomial;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::PrimeField;
use ark_std::test_rng;

/// Sumcheck Verifier
struct Verifier<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    claimed_sum: F,
    challenges: Vec<F>,
}

impl<F: PrimeField> Verifier<F> {
    /// Instantiate new sumcheck verifier
    fn new(poly: MultiLinearPolynomial<F>, claimed_sum: F) -> Self {
        Self {
            poly,
            claimed_sum,
            challenges: Vec::new(),
        }
    }

    /// Verify nth round of the sumcheck protocol
    fn verify_round(&mut self, round: usize, poly: UnivariatePolynomial<F>) -> (bool, Option<F>) {
        let p_0 = poly.evaluate(&F::zero());
        let p_1 = poly.evaluate(&F::one());
        if self.claimed_sum != p_0 + p_1 {
            return (false, None);
        }

        // TODO: replace this with fiat shamir, also not sure the randomness generator is
        //  cryptographically secure.
        // sample a random field element
        let mut rng = test_rng();
        let challenge = F::rand(&mut rng);

        self.claimed_sum = poly.evaluate(&challenge);
        self.challenges.push(challenge.clone());

        // if last round, evaluate the original poly at all the challenge points
        return if round == self.poly.n_vars() - 1 {
            let actual_eval = self.poly.evaluate(self.challenges.as_slice()).unwrap();
            (actual_eval == self.claimed_sum, None)
        } else {
            (true, Some(challenge))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::prover::Prover;
    use crate::sumcheck::verifier::Verifier;
    use ark_ff::{Fp64, MontBackend, MontConfig};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn prover_verifier_sumcheck() {
        let p = MultiLinearPolynomial::new(
            3,
            vec![
                (Fq::from(2), vec![true, true, false]),
                (Fq::from(3), vec![false, true, true]),
            ],
        )
        .unwrap();

        // instantiate prover and verifier
        let (mut prover, sum) = Prover::new(p.clone());
        let mut verifier = Verifier::new(p.clone(), sum);

        // round 0
        let uni_poly = prover.prove_round(0, None);
        let (accept_round_0, round_1_challenge) = verifier.verify_round(0, uni_poly);
        assert_eq!(accept_round_0, true);
        assert_eq!(round_1_challenge.is_some(), true);

        // round 1
        let uni_poly = prover.prove_round(1, round_1_challenge);
        let (accept_round_1, round_2_challenge) = verifier.verify_round(1, uni_poly);
        assert_eq!(accept_round_1, true);
        assert_eq!(round_2_challenge.is_some(), true);

        // round 2, final round
        // the verifier doesn't need to defer to the prover
        // so expectation is no challenge should be sent
        let uni_poly = prover.prove_round(2, round_2_challenge);
        let (accept_round_2, round_3_challenge) = verifier.verify_round(2, uni_poly);
        assert_eq!(accept_round_2, true);
        assert_eq!(round_3_challenge.is_some(), false);
    }
}
