use super::layered_circuit::{Layer, LayeredCircuit};

const INPUT_LAYER_ID: usize = 0;

// (layer, id, validation_id)
type Node = (usize, usize, usize);

#[derive(Default)]
struct Builder {
    curr_input_id: usize,
    curr_validation_id: usize,
    output_consumed: Vec<bool>,
    layered_circuit: LayeredCircuit,
}

impl Builder {
    fn next_input_id(&mut self) -> usize {
        self.curr_input_id += 1;
        self.curr_input_id - 1
    }

    fn next_validation_id(&mut self) -> usize {
        self.curr_validation_id += 1;
        self.output_consumed.push(false);
        self.curr_validation_id - 1
    }

    fn input(&mut self) -> Node {
        (
            INPUT_LAYER_ID,
            self.next_input_id(),
            self.next_validation_id(),
        )
    }

    fn input_n(&mut self, n: usize) -> Vec<Node> {
        (0..n).map(|_| self.input()).collect()
    }

    fn add(&mut self, left: &Node, right: &Node) -> Node {
        // ensure that both inputs come from the same layer
        assert_eq!(left.0, right.0);

        // mark the inputs as consumed
        self.output_consumed[left.2] = true;
        self.output_consumed[right.2] = true;

        self.insert_in_layer(left.0 + 1, GateInfo::Add(left.1, right.1))
    }

    fn mul(&mut self, left: &Node, right: &Node) -> Node {
        // ensure that both inputs come from the same layer
        assert_eq!(left.0, right.0);

        // mark the inputs as consumed
        self.output_consumed[left.2] = true;
        self.output_consumed[right.2] = true;

        // insert inputs into the appropriate layer
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

        (layer_id, id, self.next_validation_id())
    }

    fn to_layered_circuit(self) -> Option<LayeredCircuit> {
        let mut circuit = self.layered_circuit;

        let no_of_unused = self.output_consumed.into_iter().filter(|v| !v).count();

        // every created element, input, gates, start out with a consumed state of false
        // when an element is used as input to the creation of another element the inputs
        // output state changes to true.
        // hence the number of unconsumed state is an accurate representation of unconsumed nodes
        // for layered circuit, we can have more than one unconsumed nodes but they must
        // all be on the output layer.
        // hence checking that the number of unconsumed states equals the number of nodes in the
        // output layer should be a complete way to validate accurate layered circuit construction
        // given our other type constraints.
        if circuit.layers.last().unwrap().len() != no_of_unused {
            None
        } else {
            circuit.layers.reverse();
            Some(circuit)
        }
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
