use crate::gkr::layer::Layer;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use ark_ff::PrimeField;
use ark_std::iterable::Iterable;
use std::ops::Add;

/// A circuit is just a stacked collection of layers
#[derive(Clone, Debug, PartialEq)]
pub struct Circuit {
    layers: Vec<Layer>,
}

impl Circuit {
    // TODO: implement circuit construction validation
    pub fn new(layers: Vec<Layer>) -> Result<Self, &'static str> {
        if layers.is_empty() {
            return Err("cannot create circuit with no layers");
        }

        Ok(Self { layers })
    }

    /// Evaluate the circuit on a given input
    pub fn evaluate<F: PrimeField>(&self, input: Vec<F>) -> Result<Vec<Vec<F>>, &'static str> {
        if self.layers.is_empty() {
            return Err("cannot evaluate circuit is empty");
        }

        // TODO: deal with this, it's not necessarily a uniform circuit
        //  figure out how you can still ensure the input is valid
        //  can replace this with the check for minimum length, can get the min length from the max
        //  index in the layer above
        // if (self.layers.last().unwrap().len() * 2) != input.len() {
        //     return Err("not enough values for input layer");
        // }

        let mut current_layer_input = input;

        let mut evaluations = vec![current_layer_input.clone()];

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
    /// output layer index = 0
    pub fn add_mul_mle<F: PrimeField>(
        &self,
        layer_index: usize,
    ) -> Result<[MultiLinearPolynomial<F>; 2], &'static str> {
        if layer_index >= self.layers.len() {
            return Err("invalid layer index");
        }

        let next_layer_len = if layer_index == self.layers.len() - 1 {
            // next layer is the input layer
            (self.layers[layer_index].max_input_index() + 1)
                .try_into()
                .unwrap()
        } else {
            self.layers[layer_index + 1].len()
        };

        Ok(self.layers[layer_index].add_mul_mle(next_layer_len))
    }

    /// Return the additive identity of a circuit
    pub fn additive_identity(no_of_layers: usize) -> Self {
        Self::new(
            (0..no_of_layers)
                .map(|_| Layer::new(vec![], vec![]))
                .collect(),
        )
        .unwrap()
    }
}

/// Addition of two circuits is just the concatenation of the layer
/// this can be visualized as putting the two circuits side by side
impl Add for Circuit {
    type Output = Result<Self, &'static str>;
    fn add(self, rhs: Self) -> Self::Output {
        // the circuits must have the same depth
        if self.layers.len() != rhs.layers.len() {
            return Err("can only add circuits that have the same depth");
        }

        let combined_layers = self
            .layers
            .into_iter()
            .zip(rhs.layers.into_iter())
            .map(|(mut l1, l2)| {
                l1.add_gates.extend(l2.add_gates);
                l1.mul_gates.extend(l2.mul_gates);
                Layer::new(l1.add_gates, l1.mul_gates)
            })
            .collect();

        Ok(Circuit::new(combined_layers)?)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::gkr::circuit::{Circuit, Layer};
    use std::ops::Add;

    use crate::gkr::gate::Gate;
    use crate::polynomial::multilinear_extension::MultiLinearExtension;
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
        Circuit::new(vec![layer_0, layer_1, layer_2]).unwrap()
    }

    pub fn non_uniform_circuit() -> Circuit {
        let layer_0 = Layer::new(vec![Gate::new(0, 0, 1), Gate::new(1, 2, 3)], vec![]);
        let layer_1 = Layer::new(
            vec![],
            vec![
                Gate::new(0, 0, 1),
                Gate::new(1, 2, 3),
                Gate::new(2, 4, 5),
                Gate::new(3, 6, 7),
            ],
        );
        let layer_2 = Layer::new(
            vec![],
            vec![
                Gate::new(0, 1, 0),
                Gate::new(1, 1, 0),
                Gate::new(2, 2, 0),
                Gate::new(3, 0, 5),
                Gate::new(4, 2, 0),
                Gate::new(5, 1, 0),
                Gate::new(6, 3, 0),
                Gate::new(7, 0, 5),
            ],
        );
        Circuit::new(vec![layer_0, layer_1, layer_2]).unwrap()
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

        let circuit = Circuit::new(vec![layer_0, layer_1]).unwrap();

        let circuit_eval = circuit
            .evaluate(vec![Fr::from(2), Fr::from(3), Fr::from(4), Fr::from(5)])
            .expect("should eval");

        assert_eq!(circuit_eval.len(), 3);
        assert_eq!(circuit_eval[0], vec![Fr::from(100)]);
        assert_eq!(circuit_eval[1], vec![Fr::from(5), Fr::from(20)]);
        assert_eq!(
            circuit_eval[2],
            vec![Fr::from(2), Fr::from(3), Fr::from(4), Fr::from(5)]
        );

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
        assert_eq!(circuit_eval.len(), 4);
        assert_eq!(circuit_eval[0], vec![Fr::from(179)]);
        assert_eq!(circuit_eval[1], vec![Fr::from(14), Fr::from(165)]);
        assert_eq!(
            circuit_eval[2],
            vec![Fr::from(2), Fr::from(12), Fr::from(11), Fr::from(15)]
        );
        assert_eq!(
            circuit_eval[3],
            vec![
                Fr::from(1),
                Fr::from(2),
                Fr::from(3),
                Fr::from(4),
                Fr::from(5),
                Fr::from(6),
                Fr::from(7),
                Fr::from(8),
            ]
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
        let [add_0, mult_0]: [MultiLinearPolynomial<Fr>; 2] = circuit.add_mul_mle(0).unwrap();
        // the number of variables for the add function should be 3
        assert_eq!(add_0.n_vars(), 3);
        // the number of variables for the mul function should be 0
        assert_eq!(mult_0.n_vars(), 0);
        // the sum over the boolean hypercube should equate to 1
        // as we have only 1 add gate
        assert_eq!(sum_over_boolean_hyper_cube(&add_0), Fr::from(1));
        // the only eval should be what we expect (a, b, c) -> (0, 0, 1)
        assert_eq!(
            add_0
                .evaluate(&[Fr::from(0), Fr::from(0), Fr::from(1)])
                .unwrap(),
            Fr::from(1)
        );
        // mul gate should sum to 0 over the boolean hypercube
        assert_eq!(sum_over_boolean_hyper_cube(&mult_0), Fr::from(0));

        // layer 1
        let [add_1, mult_1]: [MultiLinearPolynomial<Fr>; 2] = circuit.add_mul_mle(1).unwrap();
        // number of variables for add should be 5 (1 for current layer, then 2 each for next layer)
        assert_eq!(add_1.n_vars(), 5);
        // number of variables for mul should also be 5
        assert_eq!(mult_1.n_vars(), 5);
        // we have a single add gate, sum over boolean hypercube should be 1
        assert_eq!(sum_over_boolean_hyper_cube(&add_1), Fr::from(1));
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
        assert_eq!(sum_over_boolean_hyper_cube(&mult_1), Fr::from(1));
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
        let [add_2, mult_2]: [MultiLinearPolynomial<Fr>; 2] = circuit.add_mul_mle(2).unwrap();
        // number of variables for add should be 8 (2 for current layer, then 3 each for next layer)
        assert_eq!(add_2.n_vars(), 8);
        // number of variables for mul should also be 8
        assert_eq!(mult_2.n_vars(), 8);
        // we have 2 add gates, sum over boolean hypercube should be 2
        assert_eq!(sum_over_boolean_hyper_cube(&add_2), Fr::from(2));
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
        assert_eq!(sum_over_boolean_hyper_cube(&mult_2), Fr::from(2));
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
    fn test_add_mle_and_mul_mle_generation_for_non_uniform_circuit() {
        let circuit = non_uniform_circuit();

        // the non-uniform circuit requires input of length 5
        // normally it's assumed that the next layer is 2 * previous layer length
        // layer just above input is of length 8
        // so input would be of length 16 in a uniform circuit
        let [add_last, mul_last]: [MultiLinearPolynomial<Fr>; 2] =
            circuit.add_mul_mle(circuit.layers.len() - 1).unwrap();
        // number of variables for add_i and mul_i is given by the following equation
        // no_of_vars_for_i + (2 * no_of_vars_for_i+1)
        // no_of_vars_for_i = log_2(8) = 3
        // no_of_vars_for_i+1 = log_2(5) = 3
        // total = 3 + 3(2) = 3 + 6 = 9
        assert_eq!(add_last.n_vars(), 0);
        assert_eq!(mul_last.n_vars(), 9);
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

    #[test]
    fn test_circuit_addition() {
        // Circuit A
        //    x
        //  /   \
        // a     b
        // Circuit B
        //    +
        //  /   \
        // c     d
        // Expected Combination
        //    x        +
        //  /   \    /   \
        // a     b  c     d

        let circuit_a = Circuit::new(vec![Layer::new(vec![], vec![Gate::new(0, 0, 1)])]).unwrap();
        let circuit_b = Circuit::new(vec![Layer::new(vec![Gate::new(1, 2, 3)], vec![])]).unwrap();

        // c = a + b
        let circuit_c = (circuit_a + circuit_b).unwrap();

        let evaluation_result = circuit_c
            .evaluate(vec![Fr::from(2), Fr::from(3), Fr::from(4), Fr::from(5)])
            .unwrap();
        assert_eq!(evaluation_result.len(), 2);
        assert_eq!(evaluation_result[0], vec![Fr::from(6), Fr::from(9)]);
    }

    #[test]
    fn test_circuit_additive_identity() {
        let circuit_a = Circuit::new(vec![Layer::new(vec![], vec![Gate::new(0, 0, 1)])]).unwrap();
        let circuit_b = Circuit::additive_identity(1);
        let circuit_c = (circuit_a.clone() + circuit_b).unwrap();
        assert_eq!(circuit_c, circuit_a);
    }
}
