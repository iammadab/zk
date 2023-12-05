use ark_ff::PrimeField;

/// Represents a gate wiring (2 inputs 1 output)
struct Gate {
    out: usize,
    in_a: usize,
    in_b: usize,
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

/// A circuit is just a stacked collection of layers
struct Circuit {
    layers: Vec<Layer>,
}

impl Circuit {
    fn evaluate<F: PrimeField>(&self, input: Vec<F>) -> Result<Vec<Vec<F>>, &'static str> {
        if self.layers.is_empty() {
            return Err("cannot evaluate circuit is empty");
        }

        if self.layers.last().unwrap().len != (2 * input.len()) {
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
