use ark_ff::PrimeField;
use sha3::{Digest, Keccak256};

// TODO: implement better transcript
pub struct Transcript {
    hasher: Keccak256,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            hasher: Keccak256::new(),
        }
    }

    pub fn append(&mut self, new_data: &[u8]) {
        self.hasher.update(new_data);
    }

    fn sample_challenge(&mut self) -> [u8; 32] {
        let mut result_hash = [0; 32];
        result_hash.copy_from_slice(&self.hasher.finalize_reset());
        self.hasher.update(result_hash);
        result_hash
    }

    pub fn sample_field_element<F: PrimeField>(&mut self) -> F {
        let challenge = self.sample_challenge();
        F::from_be_bytes_mod_order(&challenge)
    }

    pub fn sample_n_field_elements<F: PrimeField>(&mut self, n: usize) -> Vec<F> {
        (0..n).map(|_| self.sample_field_element()).collect()
    }
}
