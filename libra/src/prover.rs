use crate::{
    circuit::layered_circuit::LayeredCircuit,
    util::{eq_table, phase_one_table, w_i},
};
use ark_ff::PrimeField;
use polynomial::{multilinear::evaluation_form::MultiLinearPolynomial, product_poly::ProductPoly};
use sumcheck::{SumcheckProof, prover::SumcheckProver};
use transcript::Transcript;

struct GKRProof<F: PrimeField> {
    sumcheck_proofs: Vec<SumcheckProof<F>>,
    prover_hints: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(circuit: &LayeredCircuit, evaluations: Vec<Vec<F>>) -> GKRProof<F> {
    let mut transcript = Transcript::new();

    // add public input to the transcript
    // TODO: add the circuit also
    let output_mle = w_i(&evaluations, 0);
    transcript.append(output_mle.to_bytes().as_slice());

    let r_0 = transcript.sample_n_field_elements(output_mle.n_vars());
    let m_0 = output_mle.evaluate(&r_0);

    // m_0 will serve as the claimed sum
    // r_0 will serve as g
    // what next??
    // we need to perform sumcheck for a particular layer
    // the layer 0 to layer 1 connection

    todo!()
}

fn libra_sumcheck<F: PrimeField>(
    circuit: &LayeredCircuit,
    evaluations: &[Vec<F>],
    layer_id: usize,
    output_challenges: &[F],
    transcript: &mut Transcript,
) -> SumcheckProof<F> {
    let addi = circuit.addi(layer_id);
    let muli = circuit.muli(layer_id);
    let w_next = w_i(evaluations, layer_id + 1);

    //let sumcheck_prover = SumcheckProver::prove_partial(ProductPoly::new(polynomials), sum, transcript)

    // let us focus on just the muli type polynomial
    // phase 1
    // first we need to construct the phase 1 table then run sumcheck product
    let i_gz = eq_table(output_challenges);
    let hg_table = phase_one_table(&muli, &i_gz, w_next.evaluation_slice());
    let phase_one_product_poly = ProductPoly::new(vec![
        MultiLinearPolynomial::new_with_pad(hg_table, None),
        w_next,
    ])
    .unwrap();

    let (phase_one_proof, u_challenges) =
        SumcheckProver::<2, F>::prove_partial(phase_one_product_poly, F::zero(), transcript)
            .unwrap();

    todo!()
}
