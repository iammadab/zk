use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, process};
use std::fs::File;
use std::io::Write;

// TODO: add documentation
// TODO: return errors rather than unwrapping
fn compile(source_file_path: &Path) {
    // TODO: ensure the source_file_path ends in .circom
    let source_folder = source_file_path.parent().unwrap();
    let base_folder_name = source_file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        + "_gkr";
    let base_folder_path = source_folder.join(PathBuf::from(base_folder_name));

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

    // create input.json file
    let input_path = base_folder_path.join("input.json");
    let mut input_file = File::create(input_path).expect("failed to create input file");
    input_file.write_all(b"{}").expect("failed to write json file");

    // create witness.json file
    let witness_path = base_folder_path.join("witness.json");
    let mut witness_file = File::create(witness_path).expect("failed to create witness file");
    witness_file.write_all(b"{\"data\": []}").expect("failed to write json file");
}

// TODO: add documentation
fn generate_witness(source_file_path: &Path) {



    todo!()
}

fn prove(source_file_path: &Path) {
    todo!()
}

fn verify(source_file_path: &Path) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    // TODO: test with temp folders
    fn test_cli_functions() {
        let m = PathBuf::from(
            "/Users/madab/Documents/projects/2023/thaler/src/r1cs_gkr/adapters/circom/test.circom",
        );
        compile(&m);
        generate_witness(&m);
    }
}
