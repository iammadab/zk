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
        assert!(layer_id <= self.layered_circuit.layers.len());

        // if we haven't seen an element from this layer we first create the layer
        if layer_id == self.layered_circuit.layers.len() {
            self.layered_circuit.layers.push(Layer::default());
        }

        let id = self.layered_circuit.layers[layer_id].len();

        match gate_info {
            GateInfo::Add(l, r) => {
                self.layered_circuit.layers[layer_id].add_gate([id, l, r]);
            }
            GateInfo::Mul(l, r) => {
                self.layered_circuit.layers[layer_id].mul_gate([id, l, r]);
            }
        }

        (layer_id, id)
    }
}

enum GateInfo {
    // (left, right)
    Add(usize, usize),
    // (left, right)
    Mul(usize, usize),
}
