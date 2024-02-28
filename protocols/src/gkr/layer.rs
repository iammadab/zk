use crate::gkr::gate::Gate;
use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::{bit_count_for_n_elem, MultiLinearPolynomial};
use ark_ff::PrimeField;

/// Holds the add and mul gates in a given layer
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub add_gates: Vec<Gate>,
    pub mul_gates: Vec<Gate>,
    // Takes two inputs a, b and computes (a + b)^98
    pub exp_98_gates: Vec<Gate>,
    len: usize,
}

impl Layer {
    /// Instantiate a new gate layer, calculate the total gate count
    // TODO: don't allow the creation of empty layers
    pub fn new(add_gates: Vec<Gate>, mul_gates: Vec<Gate>, exp_98_gates: Vec<Gate>) -> Self {
        Self {
            len: add_gates.len() + mul_gates.len() + exp_98_gates.len(),
            add_gates,
            mul_gates,
            exp_98_gates,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Return the maximum index for an input wiring into any gate in the given layer
    pub fn max_input_index(&self) -> isize {
        let max_add_gate_index = self
            .add_gates
            .iter()
            .map(|gate| gate.in_a.max(gate.in_b) as isize)
            .max()
            .unwrap_or(-1);
        let max_mul_gate_index = self
            .mul_gates
            .iter()
            .map(|gate| gate.in_a.max(gate.in_b) as isize)
            .max()
            .unwrap_or(-1);
        let max_exp_98_gate_index = self
            .exp_98_gates
            .iter()
            .map(|gate| gate.in_a.max(gate.in_b) as isize)
            .max()
            .unwrap_or(-1);
        max_add_gate_index
            .max(max_mul_gate_index)
            .max(max_exp_98_gate_index)
    }

    /// Generate the add_i and mult_i multilinear extension polynomials for the current layer
    /// also take the size of the next layer
    pub fn add_mul_mle<F: PrimeField>(
        &self,
        next_layer_count: usize,
    ) -> [MultiLinearPolynomial<F>; 3] {
        let layer_var_count = bit_count_for_n_elem(self.len);
        let next_layer_count = bit_count_for_n_elem(next_layer_count);

        let add_mle = self.add_gates.iter().fold(
            MultiLinearPolynomial::<F>::additive_identity(),
            |acc, gate| {
                let gate_bits = gate.to_bit_string(layer_var_count, next_layer_count);
                let gate_bit_checker = MultiLinearPolynomial::<F>::bit_string_checker(gate_bits);

                (&acc + &gate_bit_checker).unwrap()
            },
        );

        let mult_mle = self.mul_gates.iter().fold(
            MultiLinearPolynomial::<F>::additive_identity(),
            |acc, gate| {
                let gate_bits = gate.to_bit_string(layer_var_count, next_layer_count);
                let gate_bit_checker = MultiLinearPolynomial::<F>::bit_string_checker(gate_bits);

                (&acc + &gate_bit_checker).unwrap()
            },
        );

        // construct selector poly for exp_98 gates
        let exp_98_mle = self.exp_98_gates.iter().fold(
            MultiLinearPolynomial::<F>::additive_identity(),
            |acc, gate| {
                let gate_bits = gate.to_bit_string(layer_var_count, next_layer_count);
                let gate_bit_checker = MultiLinearPolynomial::<F>::bit_string_checker(gate_bits);
                (&acc + &gate_bit_checker).unwrap()
            },
        );

        [add_mle, mult_mle, exp_98_mle]
    }
}

#[cfg(test)]
mod test {
    use crate::gkr::gate::Gate;
    use crate::gkr::layer::Layer;

    #[test]
    fn test_max_input_index() {
        let layer = Layer::new(
            vec![],
            vec![
                Gate::new(0, 1, 0),
                Gate::new(1, 1, 6),
                Gate::new(2, 2, 0),
                Gate::new(3, 0, 5),
                Gate::new(4, 2, 0),
                Gate::new(5, 1, 0),
                Gate::new(6, 3, 0),
                Gate::new(7, 0, 5),
            ],
            vec![],
        );
        assert_eq!(layer.max_input_index(), 6);

        let empty_layer = Layer::new(vec![], vec![], vec![]);
        assert_eq!(empty_layer.max_input_index(), -1);
    }
}
