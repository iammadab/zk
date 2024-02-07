use crate::r1cs_gkr::constraint::{Constraint, Term};
use crate::r1cs_gkr::program::R1CSProgram;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use num_bigint::{BigInt, BigUint};
use std::path::Path;

/// Holds utility methods for going from .r1cs to R1CSProgram
/// also performs witness generation from circuit input
struct CircomAdapter<E: Pairing> {
    builder: CircomBuilder<E>,
}

impl<F: PrimeField<BigInt = num_bigint::BigInt>, E: Pairing<ScalarField = F>> CircomAdapter<E> {
    /// Initialize a new circom adapater from the .r1cs and .wasm circom compiler output
    fn new(
        r1cs_file_path: impl AsRef<Path>,
        witness_generator_file_path: impl AsRef<Path>,
    ) -> Self {
        let cfg = CircomConfig::<E>::new(witness_generator_file_path, r1cs_file_path).unwrap();
        Self {
            builder: CircomBuilder::new(cfg),
        }
    }

    // TODO: test this
    /// Given the circuit input, generate the intermediate witness values
    fn generate_witness(mut self, circuit_inputs: Vec<(impl ToString, F)>) -> Vec<F> {
        // insert the inputs into the builder
        circuit_inputs
            .into_iter()
            .for_each(|(variable_name, value)| self.builder.push_input(variable_name, value));
        // generate the witness
        self.builder.build().unwrap().witness.unwrap()
    }
}

// TODO: test this
/// Generate an R1CS Program from the .r1cs compiled source
impl<F: PrimeField, E: Pairing<ScalarField = F>> From<&CircomAdapter<E>> for R1CSProgram<F> {
    fn from(adapter: &CircomAdapter<E>) -> Self {
        let circom_circuit = adapter.builder.setup();
        let constraints = circom_circuit
            .r1cs
            .constraints
            .into_iter()
            .map(|r1cs_constraint| {
                let a = r1cs_constraint
                    .0
                    .into_iter()
                    .map(|(index, value)| Term(index, value))
                    .collect();
                let b = r1cs_constraint
                    .1
                    .into_iter()
                    .map(|(index, value)| Term(index, value))
                    .collect();
                let c = r1cs_constraint
                    .2
                    .into_iter()
                    .map(|(index, value)| Term(index, value))
                    .collect();

                Constraint::new(a, b, c)
            })
            .collect();
        R1CSProgram::new(constraints)
    }
}
