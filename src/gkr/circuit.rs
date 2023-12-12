use crate::multilinear_poly::{binary_string, bit_count_for_n_elem, MultiLinearPolynomial};
use ark_ff::PrimeField;

/// Represents a gate wiring (2 inputs 1 output)
struct Gate {
    out: usize,
    in_a: usize,
    in_b: usize,
}

impl Gate {
    fn new(out: usize, in_a: usize, in_b: usize) -> Self {
        Self { out, in_a, in_b }
    }

    // TODO: add documentation
    // Should return the bit representation for a, b, and c as a long string
    // will need to pass the size of the bits, making some assumption about
    // the structure of the circuit
    fn to_bit_string(&self, out_var_count: usize, in_var_count: usize) -> String {
        let out_binary_string = binary_string(self.out, out_var_count);
        let in_a_binary_string = binary_string(self.in_a, in_var_count);
        let in_b_binary_string = binary_string(self.in_b, in_var_count);

        out_binary_string + &in_a_binary_string + &in_b_binary_string
    }
}

/// Holds the add and mul gates in a given layer
struct Layer {
    add_gates: Vec<Gate>,
    mul_gates: Vec<Gate>,
    len: usize,
}

impl Layer {
    /// Instantiate a new gate layer, calculate the total gate count
    fn new(add_gates: Vec<Gate>, mul_gates: Vec<Gate>) -> Self {
        Self {
            len: add_gates.len() + mul_gates.len(),
            add_gates,
            mul_gates,
        }
    }
}

/// Generate the add_i and mult_i multilinear extension polynomials given a layer
impl<F: PrimeField> From<&Layer> for [MultiLinearPolynomial<F>; 2] {
    fn from(layer: &Layer) -> Self {
        let layer_var_count = bit_count_for_n_elem(layer.len);
        dbg!(layer_var_count);
        // we assume input fan in of 2
        let input_var_count = bit_count_for_n_elem(layer.len * 2);
        dbg!(input_var_count);

        let add_mle = layer.add_gates.iter().fold(
            MultiLinearPolynomial::<F>::additive_identity(),
            |acc, gate| {
                // what do we do per gate?
                // we need to convert it to a string we know the var count for each
                let gate_bits = gate.to_bit_string(layer_var_count, input_var_count);
                let gate_bit_checker = MultiLinearPolynomial::<F>::bit_string_checker(gate_bits);

                (&acc + &gate_bit_checker).unwrap()
            },
        );

        let mult_mle = layer.mul_gates.iter().fold(
            MultiLinearPolynomial::<F>::additive_identity(),
            |acc, gate| {
                let gate_bits = gate.to_bit_string(layer_var_count, input_var_count);
                let gate_bit_checker = MultiLinearPolynomial::<F>::bit_string_checker(gate_bits);

                (&acc + &gate_bit_checker).unwrap()
            },
        );

        [add_mle, mult_mle]
    }
}

/// A circuit is just a stacked collection of layers
struct Circuit {
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
    fn evaluate<F: PrimeField>(&self, input: Vec<F>) -> Result<Vec<Vec<F>>, &'static str> {
        if self.layers.is_empty() {
            return Err("cannot evaluate circuit is empty");
        }

        if (self.layers.last().unwrap().len * 2) != input.len() {
            return Err("not enough values for input layer");
        }

        let mut current_layer_input = input;

        Ok(self
            .layers
            .iter()
            .rev()
            .map(|layer| {
                let mut layer_evaluations = vec![F::zero(); layer.len];

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

                layer_evaluations
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::gkr::circuit::{Circuit, Gate, Layer};

    use crate::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::util::sum_over_boolean_hyper_cube;
    use ark_bls12_381::Fr;

    fn test_circuit() -> Circuit {
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
        assert_eq!(layer_0.len, 1);

        let layer_1 = Layer::new(vec![Gate::new(0, 0, 1)], vec![Gate::new(1, 2, 3)]);
        assert_eq!(layer_1.len, 2);

        let circuit = Circuit::new(vec![layer_0, layer_1]);

        let circuit_eval = circuit
            .evaluate(vec![Fr::from(2), Fr::from(3), Fr::from(4), Fr::from(5)])
            .expect("should eval");

        assert_eq!(circuit_eval.len(), 2);
        assert_eq!(circuit_eval[0], vec![Fr::from(5), Fr::from(20)]);
        assert_eq!(circuit_eval[1], vec![Fr::from(100)]);

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
        assert_eq!(
            circuit_eval[0],
            vec![Fr::from(2), Fr::from(12), Fr::from(11), Fr::from(15)]
        );
        assert_eq!(circuit_eval[1], vec![Fr::from(14), Fr::from(165)]);
        assert_eq!(circuit_eval[2], vec![Fr::from(179)]);
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
        let [add_1, mul_1]: [MultiLinearPolynomial<Fr>; 2] = (&circuit.layers[0]).into();
        // the number of variables for the add function should be 3
        assert_eq!(add_1.n_vars(), 3);
        // the number of variables for the mul function should be 0
        assert_eq!(mul_1.n_vars(), 0);
        // the sum over the boolean hypercube should equate to 1
        // as we have only 1 add gate
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&add_1), Fr::from(1));
        // the only eval should be what we expect (a, b, c) -> (0, 0, 1)
        assert_eq!(
            add_1
                .evaluate(&[Fr::from(0), Fr::from(0), Fr::from(1)])
                .unwrap(),
            Fr::from(1)
        );
        dbg!(&mul_1);
        // mul gate should sum to 0 over the boolean hypercube
        // TODO: fix evaluate to return 0 on no vars
        assert_eq!(sum_over_boolean_hyper_cube::<Fr>(&mul_1), Fr::from(0));

        // TODO: allow the ability to evaluate with more than n-vars
    }
}
