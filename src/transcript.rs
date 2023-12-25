use ark_ff::{BigInteger, PrimeField};
use sha3::{Digest, Keccak256};

// TODO: implement better transcript
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
        F::from_be_bytes_mod_order(&challenge)
    }

    pub(crate) fn sample_n_field_elements<F: PrimeField>(&mut self, n: usize) -> Vec<F> {
        // TODO: is this secure??
        (0..n)
            .map(|_| F::from_be_bytes_mod_order(self.sample_challenge().as_slice()))
            .collect()
    }
}
