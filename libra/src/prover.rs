use crate::circuit::layered_circuit::LayeredCircuit;
use ark_ff::PrimeField;
use polynomial::multilinear::evaluation_form::MultiLinearPolynomial;
use sumcheck::SumcheckProof;
use transcript::Transcript;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(circuit: &LayeredCircuit, evaluations: Vec<Vec<F>>) -> GKRProof<F> {
    // we need to prover layer by layer
    // each layer is supposed to generate a sumcheck proof
    // and also each layer should return hints

    // let us focus on the output layer now
    // we have the output vec we can turn that to an mle
    // first we need to put the public inputs to the transcript
    // what are the public inputs??
    // the circuit should be part but for now we skip
    // we need the output mle

    let mut transcript = Transcript::new();

    let mut evaluations_iter = evaluations.into_iter();

    let mut output_vec = evaluations_iter.next().expect("empty evaluation vec");

    // ensure that output vec has evaluations for 0 and 1
    let output_vec = if output_vec.len() == 1 {
        output_vec.push(F::zero());
        output_vec
    } else {
        output_vec
    };

    let output_mle = MultiLinearPolynomial::new_with_pad(output_vec, None);

    //let output_mle = MultiLinearPolynomial::new_with_pad(evaluations, pad_element)

    todo!()
}

fn w_i<F: PrimeField>(evaluations: &[Vec<F>], layer_id: usize) -> MultiLinearPolynomial<F> {
    let values = if layer_id == 0 {
        if evaluations[0].len() == 1 {
            vec![evaluations[0][0], F::zero()]
        } else {
            evaluations[0].clone()
        }
    } else {
        evaluations[layer_id].clone()
    };

    MultiLinearPolynomial::new_with_pad(values, None)
}
