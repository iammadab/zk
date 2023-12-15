use crate::polynomial::multilinear_poly::binary_string;

/// Represents a gate wiring (2 inputs 1 output)
pub struct Gate {
    pub out: usize,
    pub in_a: usize,
    pub in_b: usize,
}

impl Gate {
    pub fn new(out: usize, in_a: usize, in_b: usize) -> Self {
        Self { out, in_a, in_b }
    }

    // TODO: add documentation
    // Should return the bit representation for a, b, and c as a long string
    // will need to pass the size of the bits, making some assumption about
    // the structure of the circuit
    pub fn to_bit_string(&self, out_var_count: usize, in_var_count: usize) -> String {
        let out_binary_string = binary_string(self.out, out_var_count);
        let in_a_binary_string = binary_string(self.in_a, in_var_count);
        let in_b_binary_string = binary_string(self.in_b, in_var_count);

        out_binary_string + &in_a_binary_string + &in_b_binary_string
    }
}
