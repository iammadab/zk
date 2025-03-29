use ark_ff::PrimeField;

#[derive(Default)]
pub(crate) struct LayeredCircuit {
    // we assume that the output layer is at index 0
    pub(crate) layers: Vec<Layer>,
}

impl LayeredCircuit {
    pub(crate) fn evaluate<F: PrimeField>(&self, inputs: &[F]) -> Vec<Vec<F>> {
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

    fn addi(&self, layer_id: usize) -> &[[usize; 3]] {
        &self.layers[layer_id].add_gates
    }

    fn muli(&self, layer_id: usize) -> &[[usize; 3]] {
        &self.layers[layer_id].mul_gates
    }
}

#[derive(Default)]
pub(crate) struct Layer {
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

    pub(crate) fn add_gate(&mut self, gate_info: [usize; 3]) {
        self.add_gates.push(gate_info);
        self.len += 1;
    }

    pub(crate) fn mul_gate(&mut self, gate_info: [usize; 3]) {
        self.mul_gates.push(gate_info);
        self.len += 1;
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::layered_circuit::{Layer, LayeredCircuit};
    use ark_bn254::Fr;

    fn circuit() -> LayeredCircuit {
        LayeredCircuit {
            layers: vec![
                Layer::new(vec![[0, 0, 1]], vec![]),
                Layer::new(vec![[0, 0, 1]], vec![[1, 2, 3]]),
            ],
        }
    }

    #[test]
    fn test_evaluate_layered_circuit() {
        let evaluations = circuit().evaluate(
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

    #[test]
    fn test_add_i_mul_i() {
        assert_eq!(circuit().addi(0), vec![[0, 0, 1]]);
        assert!(circuit().muli(0).is_empty());
        assert_eq!(circuit().addi(1), vec![[0, 0, 1]]);
        assert_eq!(circuit().muli(1), vec![[1, 2, 3]]);
    }
}
