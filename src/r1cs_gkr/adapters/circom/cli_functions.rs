use crate::r1cs_gkr::adapters::circom::CircomAdapter;
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, process};

// TODO: add documentation
fn file_name(source_file_path: &Path) -> String {
    source_file_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

// TODO: add documentation
fn base_folder(source_file_path: &Path) -> PathBuf {
    let source_folder = source_file_path.parent().unwrap();
    let base_folder_name = file_name(source_file_path) + "_gkr";
    let base_folder_path = source_folder.join(PathBuf::from(base_folder_name));
    base_folder_path
}

// TODO: add documentation
// TODO: return errors rather than unwrapping
fn compile(source_file_path: &Path) {
    // TODO: ensure the source_file_path ends in .circom
    let base_folder_path = base_folder(source_file_path);

    // create the base folder if it doesn't exist
    if !base_folder_path.exists() {
        fs::create_dir(&base_folder_path).expect("failed to create base folder");
    }

    // compile the circom program
    let _ = Command::new("circom")
        .arg(source_file_path)
        .arg("--r1cs")
        .arg("--wasm")
        .arg("--O0")
        .arg("--output")
        .arg(&base_folder_path)
        .output()
        .expect("circom command failed");

    dbg!("creating json");
    // create input.json file
    let input_path = base_folder_path.join("input.json");
    let mut input_file = File::create(input_path).expect("failed to create input file");
    input_file
        .write_all(b"{}")
        .expect("failed to write json file");

    // create witness.json file
    let witness_path = base_folder_path.join("witness.json");
    let mut witness_file = File::create(witness_path).expect("failed to create witness file");
    witness_file
        .write_all(b"{\"witness\": []}")
        .expect("failed to write json file");
}

// TODO: add documentation
//  move to a better location
#[derive(Serialize)]
struct Witness {
    witness: Vec<String>,
}

// TODO: add documentation
fn generate_witness<F: PrimeField + Into<ark_ff::BigInt<4>>, E: Pairing<ScalarField = F>>(
    source_file_path: &Path,
) {
    let file_name = file_name(source_file_path);
    let base_folder_path = base_folder(source_file_path);
    let r1cs_file = base_folder_path.join(format!("{}.r1cs", file_name));
    let wtns_generator_file = base_folder_path.join(format!("{}_js/{}.wasm", file_name, file_name));
    let input_file = base_folder_path.join("input.json");

    // if no r1cs file, witness generator or input file then perform compilation step
    if !r1cs_file.exists() || !wtns_generator_file.exists() || !input_file.exists() {
        compile(source_file_path)
    }

    // TODO: handle errors here
    let input = read_input::<F>(&input_file);

    let adapter = CircomAdapter::<E>::new(r1cs_file, wtns_generator_file);

    let witness = adapter.generate_witness(input).unwrap();

    let witness_as_strings = witness.into_iter().map(|v| v.to_string());

    let witness_struct = Witness {
        witness: witness_as_strings.collect(),
    };

    let serialized = serde_json::to_string(&witness_struct).unwrap();

    let witness_path = base_folder_path.join("witness.json");
    let mut witness_file = File::create(witness_path).expect("failed to create witness file");
    witness_file
        .write_all(serialized.as_bytes())
        .expect("failed to write json file");
}

// TODO: add documentation
// TODO: handle errors
fn read_input<F: PrimeField>(input_file_path: &Path) -> Vec<(String, F)> {
    let file = File::open(input_file_path).unwrap();
    let reader = BufReader::new(file);

    let json_data: Value = serde_json::from_reader(reader).unwrap();
    let json_object = json_data.as_object().unwrap();

    let mut inputs = vec![];

    for (key, value) in json_object {
        if !value.is_number() {
            panic!("hello")
        } else {
            let value_as_field_element =
                F::from(value.as_number().unwrap().as_u64().unwrap() as u128);
            inputs.push((key.to_owned(), value_as_field_element))
        }
    }

    inputs
}

fn prove(source_file_path: &Path) {
    // read witness
    // read .r1cs
    // use .r1cs to build circuit adapter to build r1csprogram
    // you can prove with r1cs program and witness
    todo!()
}

fn verify(source_file_path: &Path) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::{Bn254, Fr};
    use std::path::PathBuf;

    #[test]
    // TODO: test with temp folders
    fn test_cli_functions() {
        let m = PathBuf::from(
            "/Users/madab/Documents/projects/2023/thaler/src/r1cs_gkr/adapters/circom/test.circom",
        );
        // compile(&m);
        generate_witness::<Fr, Bn254>(&m);
    }
}

// TODO: test vectors (cli tool should handle the following error in a nice way)
//  point to a file that does not exist
//  point to a non-circom file
//  invalid input (bad data)
//  invalid input (anything else e.g. incomplete)
//  invalid witness
//  incorrect witness
//  invalid proof
//  incorrect proof
