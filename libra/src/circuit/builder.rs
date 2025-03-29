#![allow(unused)]

use super::layered_circuit::{Layer, LayeredCircuit};

const INPUT_LAYER_ID: usize = 0;

// (layer, id)
type Node = (usize, usize);

#[derive(Default)]
struct Builder {
    curr_input_id: usize,
    layered_circuit: LayeredCircuit,
}

impl Builder {
    fn next_input_id(&mut self) -> usize {
        self.curr_input_id += 1;
        self.curr_input_id - 1
    }

    fn input(&mut self) -> Node {
        let id = self.next_input_id();
        (INPUT_LAYER_ID, id)
    }

    fn input_n(&mut self, n: usize) -> Vec<Node> {
        (0..n).map(|_| self.input()).collect()
    }

    fn add(&mut self, left: &Node, right: &Node) -> Node {
        // ensure that both inputs come from the same layer
        assert_eq!(left.0, right.0);
        self.insert_in_layer(left.0 + 1, GateInfo::Add(left.1, right.1))
    }

    fn mul(&mut self, left: &Node, right: &Node) -> Node {
        // ensure that both inputs come from the same layer
        assert_eq!(left.0, right.0);
        self.insert_in_layer(left.0 + 1, GateInfo::Mul(left.1, right.1))
    }

    fn insert_in_layer(&mut self, layer_id: usize, gate_info: GateInfo) -> Node {
        assert!(layer_id - 1 <= self.layered_circuit.layers.len());

        // if we haven't seen an element from this layer we first create the layer
        if layer_id - 1 == self.layered_circuit.layers.len() {
            self.layered_circuit.layers.push(Layer::default());
        }

        let id = self.layered_circuit.layers[layer_id - 1].len();

        match gate_info {
            GateInfo::Add(l, r) => {
                self.layered_circuit.layers[layer_id - 1].add_gate([id, l, r]);
            }
            GateInfo::Mul(l, r) => {
                self.layered_circuit.layers[layer_id - 1].mul_gate([id, l, r]);
            }
        }

        (layer_id, id)
    }

    fn to_layered_circuit(self) -> Option<LayeredCircuit> {
        let mut circuit = self.layered_circuit;
        // TODO: do circuit validation here to ensure that we have a valid layered circuit
        circuit.layers.reverse();
        Some(circuit)
    }
}

enum GateInfo {
    // (left, right)
    Add(usize, usize),
    // (left, right)
    Mul(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::Builder;
    use ark_bn254::Fr;

    #[test]
    fn test_circuit_builder() {
        // (a + b) + (c * d)
        let mut builder = Builder::default();
        let [a, b, c, d] = builder.input_n(4)[..] else {
            panic!("")
        };

        let ab = builder.add(&a, &b);
        let cd = builder.mul(&c, &d);
        builder.add(&ab, &cd);

        assert!(builder.to_layered_circuit().is_some());
    }

    #[test]
    fn test_should_not_return_layered_circuit() {
        // (a + b) + (c * d) | e
        // e is not consumed hence serves as an invalid output
        let mut builder = Builder::default();
        let [a, b, c, d, e] = builder.input_n(5)[..] else {
            panic!("")
        };
        let ab = builder.add(&a, &b);
        let cd = builder.mul(&c, &d);
        builder.add(&ab, &cd);

        assert!(builder.to_layered_circuit().is_none());
    }
}
