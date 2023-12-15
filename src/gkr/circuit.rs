use crate::gkr::layer::Layer;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use ark_ff::PrimeField;
use ark_std::iterable::Iterable;

/// A circuit is just a stacked collection of layers
pub struct Circuit {
    layers: Vec<Layer>,
}

impl Circuit {
    // TODO: implement circuit construction validation
    fn new(layers: Vec<Layer>) -> Self {
        Self { layers }
    }
}

impl Circuit {
    /// Evaluate the circuit on a given input
    // TODO: this doesn't return the input back, decide if returning the input makes sense.
    // TODO: test this with a larger depth e.g. 4 or 5
    pub fn evaluate<F: PrimeField>(&self, input: Vec<F>) -> Result<Vec<Vec<F>>, &'static str> {
        if self.layers.is_empty() {
            return Err("cannot evaluate circuit is empty");
        }

        if (self.layers.last().unwrap().len() * 2) != input.len() {
            return Err("not enough values for input layer");
        }

        let mut current_layer_input = input;

        let mut evaluations = vec![];

        for layer in self.layers.iter().rev() {
            let mut layer_evaluations = vec![F::zero(); layer.len()];

            // add gate evaluations
            for wire in &layer.add_gates {
                layer_evaluations[wire.out] =
                    current_layer_input[wire.in_a] + current_layer_input[wire.in_b];
            }

            // mul gate evaluations
            for wire in &layer.mul_gates {
                layer_evaluations[wire.out] =
                    current_layer_input[wire.in_a] * current_layer_input[wire.in_b];
            }

            current_layer_input = layer_evaluations.clone();

            evaluations.push(layer_evaluations);
        }

        evaluations.reverse();

        Ok(evaluations)
    }

    /// Returns the mle extensions for evaluations at layer
    /// Evaluation order: [output_layer ...]
    /// output_layer index = 0
    pub fn w<F: PrimeField>(
        evaluations: &[Vec<F>],
        layer_index: usize,
    ) -> Result<MultiLinearPolynomial<F>, &'static str> {
        if layer_index >= evaluations.len() {
            return Err("invalid layer index");
        }

        Ok(MultiLinearPolynomial::<F>::interpolate(
            &evaluations[layer_index],
        ))
    }

    /// Returns the add_mle and mul_mle for the given layer
    pub fn add_mul_mle<F: PrimeField>(
        &self,
        layer_index: usize,
    ) -> Result<[MultiLinearPolynomial<F>; 2], &'static str> {
        if layer_index >= self.layers.len() {
            return Err("invalid layer index");
        }

        Ok((&self.layers[layer_index]).into())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::gkr::circuit::{Circuit, Layer};

    use crate::gkr::gate::Gate;
    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::util::sum_over_boolean_hyper_cube;
    use ark_bls12_381::Fr;

    pub fn test_circuit() -> Circuit {
        let layer_0 = Layer::new(vec![Gate::new(0, 0, 1)], vec![]);
        let layer_1 = Layer::new(vec![Gate::new(0, 0, 1)], vec![Gate::new(1, 2, 3)]);
        let layer_2 = Layer::new(
            vec![Gate::new(2, 4, 5), Gate::new(3, 6, 7)],
            vec![Gate::new(0, 0, 1), Gate::new(1, 2, 3)],
        );
        Circuit::new(vec![layer_0, layer_1, layer_2])
    }

    #[test]
    fn test_circuit_evaluation() {
        // sample circuit evaluation
        //      100(*)    - layer 0
        //     /     \
        //   5(+)   20(*) - layer 1
        //   / \    /  \
        //  2   3   4   5

        // instantiate circuit
        let layer_0 = Layer::new(vec![], vec![Gate::new(0, 0, 1)]);
        assert_eq!(layer_0.len(), 1);

        let layer_1 = Layer::new(vec![Gate::new(0, 0, 1)], vec![Gate::new(1, 2, 3)]);
        assert_eq!(layer_1.len(), 2);

        let circuit = Circuit::new(vec![layer_0, layer_1]);

        let circuit_eval = circuit
            .evaluate(vec![Fr::from(2), Fr::from(3), Fr::from(4), Fr::from(5)])
            .expect("should eval");

        assert_eq!(circuit_eval.len(), 2);
        assert_eq!(circuit_eval[0], vec![Fr::from(100)]);
        assert_eq!(circuit_eval[1], vec![Fr::from(5), Fr::from(20)]);

        // Larger circuit
        let circuit = test_circuit();
        let circuit_eval = circuit
            .evaluate(vec![
                Fr::from(1),
                Fr::from(2),
                Fr::from(3),
                Fr::from(4),
                Fr::from(5),
                Fr::from(6),
                Fr::from(7),
                Fr::from(8),
            ])
            .unwrap();
        assert_eq!(circuit_eval.len(), 3);
        assert_eq!(circuit_eval[0], vec![Fr::from(179)]);
        assert_eq!(circuit_eval[1], vec![Fr::from(14), Fr::from(165)]);
        assert_eq!(
            circuit_eval[2],
            vec![Fr::from(2), Fr::from(12), Fr::from(11), Fr::from(15)]
        );
    }

    #[test]
    fn test_gate_to_binary_string() {
        let g1 = Gate::new(0, 0, 1);
        let gate_bit = g1.to_bit_string(1, 1);
        assert_eq!(gate_bit, "001".to_string());

        let g2 = Gate::new(1, 2, 3);
        let gate_bit = g2.to_bit_string(1, 2);
        assert_eq!(gate_bit, "11011");
    }

    #[test]
    fn test_add_and_mul_mle_generation() {
        // add_mle and mul_mle should correctly identify the add and mul gates respectively
        let circuit = test_circuit();

        // circuit has 3 layers

        // layer 0 - output layer
        let [add_0, mult_0]: [MultiLinearPolynomial<Fr>; 2] = (&circuit.layers[0]).into();
        // the number of variables for the add function should be 3
        assert_eq!(add_0.n_vars(), 3);
        // the number of variables for the mul function should be 0
        assert_eq!(mult_0.n_vars(), 0);
        // the sum over the boolean hypercube should equate to 1
        // as we have only 1 add gate
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&add_0), Fr::from(1));
        // the only eval should be what we expect (a, b, c) -> (0, 0, 1)
        assert_eq!(
            add_0
                .evaluate(&[Fr::from(0), Fr::from(0), Fr::from(1)])
                .unwrap(),
            Fr::from(1)
        );
        // mul gate should sum to 0 over the boolean hypercube
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&mult_0), Fr::from(0));

        // layer 1
        let [add_1, mult_1]: [MultiLinearPolynomial<Fr>; 2] = (&circuit.layers[1]).into();
        // number of variables for add should be 5 (1 for current layer, then 2 each for next layer)
        assert_eq!(add_1.n_vars(), 5);
        // number of variables for mul should also be 5
        assert_eq!(mult_1.n_vars(), 5);
        // we have a single add gate, sum over boolean hypercube should be 1
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&add_1), Fr::from(1));
        // exact evaluation should be (a, b, c) -> (0, 0, 1) -> (0, 00, 01)
        assert_eq!(
            add_1
                .evaluate(&[
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );
        // mul gate should sum to 1 over the boolean hypercube
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&mult_1), Fr::from(1));
        // exact evaluation should be (a, b, c) -> (1, 2, 3) -> (1, 10, 11)
        assert_eq!(
            mult_1
                .evaluate(&[
                    Fr::from(1),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );

        // layer 2
        let [add_2, mult_2]: [MultiLinearPolynomial<Fr>; 2] = (&circuit.layers[2]).into();
        // number of variables for add should be 8 (2 for current layer, then 3 each for next layer)
        assert_eq!(add_2.n_vars(), 8);
        // number of variables for mul should also be 8
        assert_eq!(mult_2.n_vars(), 8);
        // we have 2 add gates, sum over boolean hypercube should be 2
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&add_2), Fr::from(2));
        // exact evaluations for add
        // (2, 4, 5) -> (10, 100, 101)
        // (3, 6, 7) -> (11, 110, 111)
        assert_eq!(
            add_2
                .evaluate(&[
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );
        assert_eq!(
            add_2
                .evaluate(&[
                    Fr::from(1),
                    Fr::from(1),
                    Fr::from(1),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(1),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );
        // we also have 2 mul gates, sum over boolean hypercube should be 2
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&mult_2), Fr::from(2));
        // exact evaluations for mul
        // (0, 0, 1) -> (00, 000, 001)
        // (1, 2, 3) -> (01, 010, 011)
        assert_eq!(
            mult_2
                .evaluate(&[
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );
        assert_eq!(
            mult_2
                .evaluate(&[
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(0),
                    Fr::from(0),
                    Fr::from(1),
                    Fr::from(1)
                ])
                .unwrap(),
            Fr::from(1)
        );
    }

    #[test]
    fn test_w_function() {
        let evaluations = vec![vec![Fr::from(1), Fr::from(2)]];
        // try to generate w mle for non-existent layer
        assert!(Circuit::w(evaluations.as_slice(), 1).is_err(),);
        assert_eq!(
            Circuit::w(evaluations.as_slice(), 0).unwrap(),
            MultiLinearPolynomial::<Fr>::interpolate(evaluations[0].as_slice())
        );
    }
}
