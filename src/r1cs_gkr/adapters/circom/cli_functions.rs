use crate::gkr::gkr::{GKRProof, GKRVerify};
use crate::r1cs_gkr::adapters::circom::CircomAdapter;
use crate::r1cs_gkr::program::R1CSProgram;
use crate::r1cs_gkr::proof::{prove_circom_gkr, verify_circom_gkr};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInt, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::Serialize;
use serde_json::{Number, Value};
use std::fmt::format;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fs, process};

// Refactor approach?
// need dedicated functions for getting common file paths
// file paths I care about are:
//   r1cs, wasm, input, witness
// will then create dedicated function for reading the contents of those files
// this will allow proper cleaning of the cli functions
// error handling should also be built in i.e. it should print to stderr directly
//  - need an exit with message function

// what does the cli function struct need?
// guess the source file path

// TODO: add documentation
//  move to a better location
#[derive(Serialize)]
struct Witness {
    witness: Vec<String>,
}

struct CLIFunctions<'a, F> {
    source_file_path: &'a Path,
    _marker: PhantomData<F>,
}

impl<'a, F: PrimeField> CLIFunctions<'a, F> {
    /// Create new clifunctions from source file path
    fn new(source_file_path: &'a Path) -> Self {
        Self {
            source_file_path,
            _marker: PhantomData,
        }
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
        let base_folder_path = source_folder.join(PathBuf::from(base_folder_name));
        base_folder_path
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

    /// Create a new input.json file and write the empty object "{}"
    fn write_empty_input(&self) -> Result<(), &'static str> {
        let mut input_file =
            File::create(self.input_path()).map_err(|_| "failed to create input.json file")?;
        input_file
            .write_all(b"{}")
            .map_err(|_| "failed to write empty object to input.json")
    }

    /// Create new witness.jso file and write empty witness array "{ witness: [] }"
    fn write_empty_witness(&self) -> Result<(), &'static str> {
        let mut witness_file =
            File::create(self.witness_path()).map_err(|_| "failed to create witness.json")?;
        witness_file
            .write_all(b"{\"witness\": []}")
            .map_err(|_| "failed to write empty witness array to witness.json")
    }

    /// Read and process the input.json file
    fn read_input(&self) -> Result<Vec<(String, F)>, &'static str> {
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
            .collect::<Result<Vec<(String, F)>, &'static str>>()
    }

    /// Compiles the circom source to .r1cs and .wasm
    fn compile(&self) -> Result<(), &'static str> {
        if !self.source_file_path.exists() {
            return Err("source file not found");
        }

        if !self.source_file_path.to_string_lossy().ends_with(".circom") {
            return Err("source file must be a circom file");
        }

        if !self.base_folder().exists() {
            fs::create_dir(&self.base_folder()).map_err(|_| "failed to create base folder")?;
        }

        // compile the circom program
        let _ = Command::new("circom")
            .arg(self.source_file_path)
            .arg("--r1cs")
            .arg("--wasm")
            .arg("--O0")
            .arg("--output")
            .arg(self.base_folder())
            .output()
            .map_err(|_| "issue compiling source with circom compiler")?;

        self.write_empty_input()?;
        self.write_empty_witness()?;

        Ok(())
    }

    /// Generate circom witness from input
    fn generate_witness(&self) -> Result<(), &'static str> {
        // if no r1cs, witness generator or input file, perform compilation step
        if !self.r1cs_path().exists() || !self.wasm_path().exists() || !self.input_path().exists() {
            self.compile()?
        }

        let input = self.read_input()?;

        // let input = self.read_input()?;
        // let adapter = CircomAdapter::<E>::new(self.r1cs_path(), self.wasm_path());
        // let witness = adapter.generate_witness(input).map_err(|_| "failed to generate witness from input, ensure you supplied the correct input")?;
        // let witness_struct = serialize_witness(witness);
        // let serialized_witness = serde_json::to_string()

        todo!()
    }

    //
    //
    //     // TODO: add documentation
    // // TODO: fix considers 0 as empty string
    //     fn generate_witness<F: PrimeField + Into<ark_ff::BigInt<4>>, E: Pairing<ScalarField=F>>(
    //         source_file_path: &Path,
    //     ) {
    //         let file_name = file_name(source_file_path);
    //         let base_folder_path = base_folder(source_file_path);
    //         let r1cs_file = base_folder_path.join(format!("{}.r1cs", file_name));
    //         let wtns_generator_file = base_folder_path.join(format!("{}_js/{}.wasm", file_name, file_name));
    //         let input_file = base_folder_path.join("input.json");
    //
    //         // if no r1cs file, witness generator or input file then perform compilation step
    //         if !r1cs_file.exists() || !wtns_generator_file.exists() || !input_file.exists() {
    //             compile(source_file_path)
    //         }
    //
    //         // TODO: handle errors here
    //         let input = read_input::<F>(&input_file);
    //
    //         let adapter = CircomAdapter::<E>::new(r1cs_file, wtns_generator_file);
    //
    //         let witness = adapter.generate_witness(input).unwrap();
    //
    //         let witness_as_strings = witness.into_iter().map(|v| v.to_string());
    //
    //         let witness_struct = Witness {
    //             witness: witness_as_strings.collect(),
    //         };
    //
    //         let serialized = serde_json::to_string(&witness_struct).unwrap();
    //
    //         let witness_path = base_folder_path.join("witness.json");
    //         let mut witness_file = File::create(witness_path).expect("failed to create witness file");
    //         witness_file
    //             .write_all(serialized.as_bytes())
    //             .expect("failed to write json file");
    //     }
    //
    //     // TODO: add documentation
    // // TODO: handle errors
    //     fn read_input<F: PrimeField>(input_file_path: &Path) -> Vec<(String, F)> {
    //         let file = File::open(input_file_path).unwrap();
    //         let reader = BufReader::new(file);
    //
    //         let json_data: Value = serde_json::from_reader(reader).unwrap();
    //         let json_object = json_data.as_object().unwrap();
    //
    //         let mut inputs = vec![];
    //
    //         for (key, value) in json_object {
    //             if !value.is_number() {
    //                 panic!("hello")
    //             } else {
    //                 let value_as_field_element =
    //                     F::from(value.as_number().unwrap().as_u64().unwrap() as u128);
    //                 inputs.push((key.to_owned(), value_as_field_element))
    //             }
    //         }
    //
    //         inputs
    //     }
    //
    //     fn read_witness<F: PrimeField>(witness_file_path: &Path) -> Vec<F> {
    //         let file = File::open(witness_file_path).unwrap();
    //         let reader = BufReader::new(file);
    //
    //         let json_data: Value = serde_json::from_reader(reader).unwrap();
    //         let json_object = json_data.as_object().unwrap();
    //
    //         let mut witness_string_array = json_object.get("witness").unwrap().as_array().unwrap().to_owned();
    //
    //         let witness_field_elements = witness_string_array.into_iter().map(|val| {
    //             let m = num_bigint::BigInt::from_str(val.as_str().unwrap()).unwrap();
    //             F::from_be_bytes_mod_order(m.to_bytes_be().1.as_slice())
    //         });
    //
    //         witness_field_elements.collect()
    //     }
    //
    //     fn prove<F: PrimeField + Into<ark_ff::BigInt<4>>, E: Pairing<ScalarField=F>>(source_file_path: &Path) {
    //         // read witness
    //         // read .r1cs
    //         // use .r1cs to build circuit adapter to build r1csprogram
    //         // you can prove with r1cs program and witness
    //
    //         let file_name = file_name(source_file_path);
    //         let base_folder_path = base_folder(source_file_path);
    //         let witness_path = base_folder_path.join("witness.json");
    //         let r1cs_path = base_folder_path.join(format!("{}.r1cs", file_name));
    //         let wtns_generator_file = base_folder_path.join(format!("{}_js/{}.wasm", file_name, file_name));
    //
    //         let witness: Vec<F> = read_witness(&witness_path);
    //
    //         let adapter = CircomAdapter::<E>::new(r1cs_path, wtns_generator_file);
    //         let program: R1CSProgram<F> = (&adapter).into();
    //
    //         let proof = prove_circom_gkr(program, witness).unwrap();
    //         let mut serialized_proof = vec![];
    //         proof.serialize_uncompressed(&mut serialized_proof);
    //
    //         let mut proof_path = base_folder_path.join("proof.bin");
    //         let mut proof_file = File::create(proof_path).expect("failed to create proof path");
    //         proof_file.write_all(serialized_proof.as_slice()).expect("failed to write proof to file");
    //     }
    //
    //     fn verify<F: PrimeField + Into<ark_ff::BigInt<4>>, E: Pairing<ScalarField=F>>(source_file_path: &Path) {
    //         let file_name = file_name(source_file_path);
    //         let base_folder_path = base_folder(source_file_path);
    //         let proof_path = base_folder_path.join("proof.bin");
    //
    //         let wtns_generator_file = base_folder_path.join(format!("{}_js/{}.wasm", file_name, file_name));
    //         let r1cs_path = base_folder_path.join(format!("{}.r1cs", file_name));
    //
    //         let adapter = CircomAdapter::<E>::new(r1cs_path, wtns_generator_file);
    //         let program: R1CSProgram<F> = (&adapter).into();
    //
    //         let witness_path = base_folder_path.join("witness.json");
    //         let witness: Vec<F> = read_witness(&witness_path);
    //         let mut proof_file = File::open(proof_path).unwrap();
    //         let mut proof_data = vec![];
    //         proof_file.read_to_end(&mut proof_data);
    //         let gkr_proof: GKRProof<F> = GKRProof::deserialize_uncompressed(Cursor::new(proof_data)).unwrap();
    //
    //         dbg!(verify_circom_gkr(program, witness, gkr_proof));
    //     }
}

/// Attempt to convert a json value to a field element
/// return an error if not possible
fn json_value_to_field_element<F: PrimeField>(val: &Value) -> Result<F, &'static str> {
    let val_str = val
        .as_str()
        .ok_or("invalid input.json value: expected number")?;
    let val_big_int = num_bigint::BigInt::from_str(val_str)
        .map_err(|_| "invalid input.json value: expected number")?;
    Ok(F::from_be_bytes_mod_order(
        val_big_int.to_bytes_be().1.as_slice(),
    ))
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
        let cli_functions = CLIFunctions::<Fr>::new(&source_file_path);

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
    }

    #[test]
    fn fake_test() {
        let source_path =
            PathBuf::from("src/r1cs_gkr/adapters/circom/test_artifacts/program.circom");
        let cli_functions = CLIFunctions::<Fr>::new(&source_path);
        // cli_functions.compile().unwrap();
        cli_functions.generate_witness().unwrap();
    }
}

// TODO: test vectors (cli tool should handle the following error in a nice way)
//  point to a file that does not exist (done)
//  point to a non-circom file (done)
//  invalid input (bad data)
//  invalid input (anything else e.g. incomplete)
//  invalid witness
//  incorrect witness
//  invalid proof
//  incorrect proof
