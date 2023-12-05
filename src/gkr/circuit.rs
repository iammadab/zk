use ark_ff::PrimeField;

#[derive(Clone)]
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
}

#[derive(Clone)]
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
                dbg!(layer.len);

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

    use ark_bls12_381::Fr;

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
        assert_eq!(circuit_eval[1], vec![Fr::from(5), Fr::from(20)]);
        assert_eq!(circuit_eval[0], vec![Fr::from(100)]);
    }
}
