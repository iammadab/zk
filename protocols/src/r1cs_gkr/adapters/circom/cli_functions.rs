use crate::gkr::protocol::Proof as GKRProof;
use crate::r1cs_gkr::adapters::circom::CircomAdapter;
use crate::r1cs_gkr::program::R1CSProgram;
use crate::r1cs_gkr::proof::{prove_circom_gkr, verify_circom_gkr};
use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{stderr, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

pub struct CLIFunctions<'a> {
    source_file_path: &'a Path,
}

impl<'a> CLIFunctions<'a> {
    /// Create new clifunctions from source file path
    pub fn new(source_file_path: &'a Path) -> Self {
        Self { source_file_path }
    }

    /// Returns the circom file name
    /// e.g. program.circom ---returns--> program
    fn file_name(&self) -> String {
        self.source_file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    /// Returns the base folder that all circom gkr related files will be stored
    fn base_folder(&self) -> PathBuf {
        let source_folder = self.source_file_path.parent().unwrap();
        let base_folder_name = self.file_name() + "_gkr";
        source_folder.join(PathBuf::from(base_folder_name))
    }

    /// Returns the path to the R1CS file
    fn r1cs_path(&self) -> PathBuf {
        self.base_folder()
            .join(format!("{}.r1cs", self.file_name()))
    }

    /// Returns the path to the wasm witness generator
    fn wasm_path(&self) -> PathBuf {
        self.base_folder()
            .join(format!("{}_js/{}.wasm", self.file_name(), self.file_name()))
    }

    /// Returns the path to the input.json file
    fn input_path(&self) -> PathBuf {
        self.base_folder().join("input.json")
    }

    /// Returns the path to the witness.json file
    fn witness_path(&self) -> PathBuf {
        self.base_folder().join("witness.json")
    }

    /// Returns the path to the proof binary
    fn proof_path(&self) -> PathBuf {
        self.base_folder().join("proof.bin")
    }

    /// Returns the path to the proof json file
    fn proof_path_json(&self) -> PathBuf {
        self.base_folder().join("proof.json")
    }

    /// Create a new input.json file and write the empty object "{}"
    fn write_empty_input(&self) -> Result<(), &'static str> {
        write_file(&self.input_path(), b"{}")
    }

    /// Create new witness.jso file and write empty witness array "[]"
    fn write_empty_witness(&self) -> Result<(), &'static str> {
        write_file(&self.witness_path(), b"[]")
    }

    // TODO: deal with array based inputs
    //  create issue
    /// Read and process the input.json file
    fn read_input(&self) -> Result<Vec<(String, Fr)>, &'static str> {
        let file = File::open(self.input_path()).map_err(|_| "failed to open input file")?;
        let reader = BufReader::new(file);

        let json_data: Value =
            serde_json::from_reader(reader).map_err(|_| "corrupted data in input.json")?;
        let json_object = json_data
            .as_object()
            .ok_or("expect input.json to contain a json object")?;

        json_object
            .into_iter()
            .map(|(key, val)| json_value_to_field_element(val).map(|fe| (key.to_owned(), fe)))
            .collect::<Result<Vec<(String, Fr)>, &'static str>>()
    }

    /// Read and process the witness.json file
    fn read_witness(&self) -> Result<Vec<Fr>, &'static str> {
        let file = File::open(self.witness_path()).map_err(|_| "failed to open witness file")?;
        let reader = BufReader::new(file);

        let json_data: Value =
            serde_json::from_reader(reader).map_err(|_| "corrupted data in witness.json")?;
        let json_object = json_data
            .as_array()
            .ok_or("expect witness.json to contain a json object")?;

        json_object
            .iter()
            .map(json_value_to_field_element)
            .collect::<Result<Vec<Fr>, &'static str>>()
    }

    /// Read and process the proof.bin file
    fn read_proof(&self) -> Result<GKRProof<Fr>, &'static str> {
        let mut proof_file =
            File::open(self.proof_path()).map_err(|_| "failed to open proof file")?;
        let mut proof_data = vec![];
        proof_file
            .read_to_end(&mut proof_data)
            .map_err(|_| "failed to read proof file")?;
        GKRProof::deserialize_uncompressed(Cursor::new(proof_data))
            .map_err(|_| "failed to deserialize proof")
    }

    /// Compiles the circom source to .r1cs and .wasm
    pub fn compile(&self) -> Result<(), &'static str> {
        if !self.source_file_path.exists() {
            return Err("source file not found");
        }

        if !self.source_file_path.to_string_lossy().ends_with(".circom") {
            return Err("source file must be a circom file");
        }

        if !self.base_folder().exists() {
            fs::create_dir(self.base_folder()).map_err(|_| "failed to create base folder")?;
        }

        // compile the circom program
        let _ = Command::new("circom")
            .arg(self.source_file_path)
            .arg("--r1cs")
            .arg("--wasm")
            .arg("--O0")
            .arg("--output")
            .arg(&self.base_folder())
            .stderr(stderr())
            .output()
            .map_err(|_| "issue compiling source with circom compiler")?;

        self.write_empty_input()?;
        self.write_empty_witness()?;

        println!("compilation successful");
        println!("generated empty input.json");
        println!("generated empty witness.json");

        Ok(())
    }

    /// Ensures that the compilation stage has occurred previously
    fn guard(&self) -> Result<(), &'static str> {
        // if no r1cs, witness generator or input file, perform compilation step
        if !self.r1cs_path().exists() || !self.wasm_path().exists() || !self.input_path().exists() {
            self.compile()?;

            if !self.witness_path().exists() {
                return Err("insert input for witness generation");
            }
        }

        Ok(())
    }

    /// Converts the circom code to an R1CSProgram, also returns the witness
    fn get_program_and_witness(&self) -> Result<(R1CSProgram<Fr>, Vec<Fr>), &'static str> {
        let adapter = CircomAdapter::<Bn254>::new(self.r1cs_path(), self.wasm_path());
        let program: R1CSProgram<Fr> = (&adapter).into();
        let witness = self.read_witness()?;

        Ok((program, witness))
    }

    /// Generate circom witness from input
    pub fn generate_witness(&self) -> Result<(), &'static str> {
        // if no r1cs, witness generator or input file, perform compilation step
        if !self.r1cs_path().exists() || !self.wasm_path().exists() || !self.input_path().exists() {
            self.compile()?
        }

        // read and process the contents of the input.json file
        let input = self.read_input()?;

        let adapter = CircomAdapter::<Bn254>::new(self.r1cs_path(), self.wasm_path());
        let witness = adapter.generate_witness(input).map_err(|_| {
            "failed to generate witness from input, ensure you supplied the correct witness"
        })?;
        let witness_as_string = witness
            .into_iter()
            .map(|witness| witness.to_string())
            .collect::<Vec<String>>();

        write_file(
            &self.witness_path(),
            serde_json::to_string(&Value::from(witness_as_string))
                .expect("this should not fail")
                .as_bytes(),
        )?;

        println!("generated witness values");
        println!("written to witness.json");

        Ok(())
    }

    /// Convert circom program to a gkr circuit and compute a proof with the witness
    pub fn prove(&self) -> Result<(), &'static str> {
        // ensure we have the pre-requisites for proving
        self.guard()?;

        let (program, witness) = self.get_program_and_witness()?;

        let proof = prove_circom_gkr(program, witness)?;

        let mut serialized_proof = vec![];
        proof
            .serialize_uncompressed(&mut serialized_proof)
            .map_err(|_| "failed to seralize proof")?;

        let _ = write_file(&self.proof_path(), serialized_proof.as_slice());

        let proof_string = format!("{{\"proof\": \"{}\"}}", hex::encode(&serialized_proof));
        write_file(&self.proof_path_json(), proof_string.as_bytes())
    }

    /// Verify generate proof, for given program and witness
    pub fn verify(&self) -> Result<(), &'static str> {
        // ensure we have the pre-requisites for proving
        self.guard()?;

        let (program, witness) = self.get_program_and_witness()?;
        let proof = self.read_proof()?;

        if verify_circom_gkr(program, witness, proof)? {
            println!("verification successful");
        } else {
            println!("verification failed");
        }

        Ok(())
    }
}

/// Attempt to convert a json value to a field element
/// return an error if not possible
fn json_value_to_field_element<F: PrimeField>(val: &Value) -> Result<F, &'static str> {
    let val_str = val
        .as_str()
        .ok_or("invalid input.json expected number strings for value e.g. {\"a\": \"1\"}")?;

    if val_str.is_empty() {
        return Ok(F::zero());
    }

    let val_big_int = num_bigint::BigInt::from_str(val_str)
        .map_err(|_| "invalid input.json value: expected number")?;

    Ok(F::from_be_bytes_mod_order(
        val_big_int.to_bytes_be().1.as_slice(),
    ))
}

/// Attempt to open a file at a specific path and overwrite it with some data
fn write_file(file_path: &Path, data: &[u8]) -> Result<(), &'static str> {
    let mut file = File::create(file_path).map_err(|_| "failed to create file")?;
    file.write_all(data).map_err(|_| "failed to write to file")
}

#[cfg(test)]
mod tests {
    use crate::r1cs_gkr::adapters::circom::cli_functions::{
        json_value_to_field_element, CLIFunctions,
    };
    use ark_bn254::Fr;
    use serde_json::Value;
    use std::path::PathBuf;

    #[test]
    fn test_file_path_constructors() {
        let test_artifacts = "src/r1cs_gkr/adapters/circom/test_artifacts".to_string();
        let base_folder = test_artifacts.clone() + "/program_gkr";
        let source_path = test_artifacts + "/program.circom";

        let source_file_path = PathBuf::from(source_path);
        let cli_functions = CLIFunctions::new(&source_file_path);

        assert_eq!(cli_functions.file_name(), "program");
        assert_eq!(
            cli_functions.base_folder().to_string_lossy(),
            base_folder.clone()
        );
        assert_eq!(
            cli_functions.r1cs_path().to_string_lossy(),
            base_folder.clone() + "/program.r1cs"
        );
        assert_eq!(
            cli_functions.wasm_path().to_string_lossy(),
            base_folder.clone() + "/program_js/program.wasm"
        );
        assert_eq!(
            cli_functions.input_path().to_string_lossy(),
            base_folder.clone() + "/input.json"
        );
        assert_eq!(
            cli_functions.witness_path().to_string_lossy(),
            base_folder.clone() + "/witness.json"
        );
        assert_eq!(
            cli_functions.proof_path().to_string_lossy(),
            base_folder.clone() + "/proof.bin"
        );
    }

    #[test]
    fn test_json_value_to_field_element() {
        let val_as_string = Value::from(
            "9824591917714408054315164033316029513904852844852789288956910420447629608399",
        );
        let field_element = json_value_to_field_element::<Fr>(&val_as_string).unwrap();
        assert_eq!(val_as_string, field_element.to_string());

        assert_eq!(
            Fr::from(0),
            json_value_to_field_element(&Value::from("")).unwrap()
        );
    }
}
