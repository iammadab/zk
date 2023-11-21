use ark_ff::PrimeField;
use sha3::{Digest, Keccak256};

pub struct Transcript {
    hasher: Keccak256,
}

impl Transcript {
    pub(crate) fn new() -> Self {
        Self {
            hasher: Keccak256::new(),
        }
    }

    pub(crate) fn append(&mut self, new_data: &[u8]) {
        self.hasher.update(&mut new_data.clone());
    }

    fn sample_challenge(&mut self) -> [u8; 32] {
        let mut result_hash = [0; 32];
        result_hash.copy_from_slice(&self.hasher.finalize_reset());
        self.hasher.update(result_hash);
        result_hash
    }

    pub(crate) fn sample_field_element<F: PrimeField>(&mut self) -> F {
        let challenge = self.sample_challenge();
        F::from_random_bytes(&challenge).unwrap()
    }
}