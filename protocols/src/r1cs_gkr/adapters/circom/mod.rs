pub mod cli_functions;

// TODO: move into adpater.rs

use crate::r1cs_gkr::constraint::{Constraint, Term};
use crate::r1cs_gkr::program::R1CSProgram;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField};
use num_bigint::{BigInt, Sign};
use std::path::Path;

/// Holds utility methods for going from .r1cs to R1CSProgram
/// also performs witness generation from circuit input
struct CircomAdapter<E: Pairing> {
    builder: CircomBuilder<E>,
}

impl<F: PrimeField + Into<ark_ff::BigInt<4>>, E: Pairing<ScalarField = F>> CircomAdapter<E> {
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

    /// Given the circuit input, generate the intermediate witness values
    fn generate_witness(
        mut self,
        circuit_inputs: Vec<(impl ToString, F)>,
    ) -> Result<Vec<F>, &'static str> {
        // insert the inputs into the builder
        circuit_inputs
            .into_iter()
            .for_each(|(variable_name, value)| {
                self.builder
                    .push_input(variable_name, BigIntAdapter(value.into()))
            });

        // generate the witness
        let witness = self.builder.build()
            .map_err(|_| "failed to build the witness")?
            .witness
            .ok_or("invalid input, ensure you pass all inputs defined by the circom source (they must also satisfy any relevant constraint)")?;

        // remove the first witness input
        // circom adds a 1 at index 0, we also do this at a later
        // this is technically not part of the generated witness
        Ok(witness.into_iter().skip(1).collect())
    }
}

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

struct BigIntAdapter(ark_ff::BigInt<4>);

impl From<BigIntAdapter> for BigInt {
    fn from(value: BigIntAdapter) -> Self {
        BigInt::from_bytes_be(Sign::Plus, &value.0.to_bytes_be())
    }
}

#[cfg(test)]
mod tests {
    use crate::r1cs_gkr::adapters::circom::CircomAdapter;
    use crate::r1cs_gkr::program::test::eq_3a_plus_5b;
    use crate::r1cs_gkr::program::R1CSProgram;
    use crate::r1cs_gkr::proof::{prove_circom_gkr, verify_circom_gkr};
    use ark_bn254::Bn254;
    use ark_ec::pairing::Pairing;
    use std::path::PathBuf;

    type Fr = <Bn254 as Pairing>::ScalarField;

    #[test]
    fn test_circom_adapter_with_proof() {
        let mut test_artifacts = PathBuf::from(file!())
            .parent()
            .expect("should have parent")
            .strip_prefix("protocols")
            .expect("should have protocols prefix")
            .join("test_artifacts");
        let r1cs = test_artifacts.join("test_circuit.r1cs");
        let wtns = test_artifacts.join("test_circuit.wasm");

        let adapter = CircomAdapter::<Bn254>::new(r1cs, wtns);

        let program = R1CSProgram::from(&adapter);
        assert_eq!(program, eq_3a_plus_5b::<<Bn254 as Pairing>::ScalarField>());

        let witness = adapter
            .generate_witness(vec![
                ("a", Fr::from(2)),
                ("b", Fr::from(3)),
                ("c", Fr::from(31)),
            ])
            .unwrap();

        assert_eq!(
            witness,
            vec![
                Fr::from(2),
                Fr::from(3),
                Fr::from(31),
                Fr::from(6),
                Fr::from(15),
            ]
        );

        let proof = prove_circom_gkr(program.clone(), witness.clone()).unwrap();
        assert!(verify_circom_gkr(program, witness, proof).unwrap());
    }
}
