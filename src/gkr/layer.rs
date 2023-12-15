use crate::gkr::gate::Gate;
use crate::polynomial::multilinear_poly::{bit_count_for_n_elem, MultiLinearPolynomial};
use ark_ff::PrimeField;

/// Holds the add and mul gates in a given layer
pub struct Layer {
    pub add_gates: Vec<Gate>,
    pub mul_gates: Vec<Gate>,
    len: usize,
}

impl Layer {
    /// Instantiate a new gate layer, calculate the total gate count
    // TODO: don't allow the creation of empty layers
    pub fn new(add_gates: Vec<Gate>, mul_gates: Vec<Gate>) -> Self {
        Self {
            len: add_gates.len() + mul_gates.len(),
            add_gates,
            mul_gates,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

/// Generate the add_i and mult_i multilinear extension polynomials given a layer
impl<F: PrimeField> From<&Layer> for [MultiLinearPolynomial<F>; 2] {
    fn from(layer: &Layer) -> Self {
        let layer_var_count = bit_count_for_n_elem(layer.len);
        // we assume input fan in of 2
        let input_var_count = bit_count_for_n_elem(layer.len * 2);

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
