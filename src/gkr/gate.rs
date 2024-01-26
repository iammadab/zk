use crate::polynomial::multilinear_poly::binary_string;

/// Represents a gate wiring (2 inputs 1 output)
#[derive(Clone, Debug)]
pub struct Gate {
    pub out: usize,
    pub in_a: usize,
    pub in_b: usize,
}

impl Gate {
    pub fn new(out: usize, in_a: usize, in_b: usize) -> Self {
        Self { out, in_a, in_b }
    }

    /// Return a binary string of the form out + in_a + in_b
    /// each binary string size is specified as input to the function
    pub fn to_bit_string(&self, out_var_count: usize, in_var_count: usize) -> String {
        let out_binary_string = binary_string(self.out, out_var_count);
        let in_a_binary_string = binary_string(self.in_a, in_var_count);
        let in_b_binary_string = binary_string(self.in_b, in_var_count);

        out_binary_string + &in_a_binary_string + &in_b_binary_string
    }
}
