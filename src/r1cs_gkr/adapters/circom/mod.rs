use ark_circom::{CircomBuilder, CircomConfig};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use std::path::Path;

/// Holds utility methods for going from .r1cs to R1CSProgram
/// also performs witness generation from circuit input
struct CircomAdapter<E: Pairing> {
    builder: CircomBuilder<E>,
}

impl<E: Pairing> CircomAdapter<E> {
    /// Initialize a new circom adapater from the .r1cs and .wasm circom compiler output
    fn new(
        r1cs_file_path: impl AsRef<Path>,
        witness_generator_file_path: impl AsRef<Path>,
    ) -> Self {
        let cfg = CircomConfig::<E>::new(witness_generator_file_path, r1cs_file_path).unwrap();
        Self {
            builder: CircomBuilder::new(cfg)
        }
    }
}
