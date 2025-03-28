use ark_ff::PrimeField;

struct LayeredCircuit {
    // we assume that the output layer is at index 0
    layers: Vec<Layer>,
}

impl LayeredCircuit {
    fn evaluate<F: PrimeField>(&self, inputs: &[F]) -> Vec<Vec<F>> {
        let mut evaluations = Vec::with_capacity(self.layers.len());
        evaluations.push(inputs.to_vec());

        for layer in self.layers.iter().rev() {
            let mut layer_evaluations = vec![F::zero(); layer.len];
            let layer_inputs = evaluations.last().unwrap();
            for gate in &layer.add_gates {
                layer_evaluations[gate[0]] = layer_inputs[gate[1]] + layer_inputs[gate[2]];
            }

            for gate in &layer.mul_gates {
                layer_evaluations[gate[0]] = layer_inputs[gate[1]] * layer_inputs[gate[2]];
            }

            evaluations.push(layer_evaluations);
        }

        evaluations
    }
}

struct Layer {
    add_gates: Vec<[usize; 3]>,
    mul_gates: Vec<[usize; 3]>,
    len: usize,
}

impl Layer {
    fn new(add_gates: Vec<[usize; 3]>, mul_gates: Vec<[usize; 3]>) -> Self {
        Self {
            len: add_gates.len() + mul_gates.len(),
            add_gates,
            mul_gates,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::{Layer, LayeredCircuit};
    use ark_bn254::Fr;

    #[test]
    fn test_evaluate_layered_circuit() {
        let circuit = LayeredCircuit {
            layers: vec![
                Layer::new(vec![[0, 0, 1]], vec![]),
                Layer::new(vec![[0, 0, 1]], vec![[1, 2, 3]]),
            ],
        };
        let evaluations = circuit.evaluate(
            vec![1, 2, 3, 4]
                .into_iter()
                .map(Fr::from)
                .collect::<Vec<_>>()
                .as_slice(),
        );
        assert_eq!(
            evaluations,
            vec![
                // input layer
                vec![Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4)],
                // mid layer
                vec![Fr::from(3), Fr::from(12)],
                // output layer
                vec![Fr::from(15)]
            ]
        );
    }
}
