use crate::multilinear_poly::MultiLinearPolynomial;
use crate::sumcheck::prover::Prover;
use crate::transcript::Transcript;
use crate::univariate_poly::UnivariatePolynomial;
use ark_ff::{BigInteger, PrimeField};

pub mod boolean_hypercube;
mod prover;
mod verifier;

struct SumcheckProof<F: PrimeField> {
    poly: MultiLinearPolynomial<F>,
    sum: F,
    uni_polys: Vec<UnivariatePolynomial<F>>,
}

struct Sumcheck {}

impl Sumcheck {
    fn prove<F: PrimeField>(poly: MultiLinearPolynomial<F>, sum: F) -> SumcheckProof<F> {
        // let mut challenges = vec![];

        // TODO: how do we add a polynomial to the transcript
        // for each variable,
        todo!()
    }

    fn verify<F: PrimeField>(proof: SumcheckProof<F>) -> bool {
        todo!()
    }
}

/// Add a polynomial to a transcript object
fn add_poly_to_transcript<F: PrimeField>(
    poly: &MultiLinearPolynomial<F>,
    transcript: &mut Transcript,
) {
    transcript.append(&poly.n_vars().to_be_bytes());
    for (var_id, coeff) in poly.coefficients() {
        transcript.append(&var_id.to_be_bytes());
        transcript.append(&coeff.into_bigint().to_bytes_be().as_slice());
    }
}

#[cfg(test)]
mod tests {
    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::add_poly_to_transcript;
    use crate::transcript::Transcript;
    use ark_ff::{Fp64, MontBackend, MontConfig, One};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_add_poly_to_transcript() {
        let mut transcript = Transcript::new();
        let poly = MultiLinearPolynomial::<Fq>::additive_identity();
        add_poly_to_transcript(&poly, &mut transcript);
        assert_eq!(transcript.sample_field_element::<Fq>(), Fq::one());
    }
}
