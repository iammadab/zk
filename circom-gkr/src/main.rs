use std::path::PathBuf;
use clap::{command, Arg, Command, ArgMatches};
use crypto::r1cs_gkr::adapters::circom::cli_functions::CLIFunctions;

const SOURCE: &'static str = "source_file";

fn main() {
    let source_file_arg = Arg::new(SOURCE).required(true);

    let match_result = command!()
        .subcommand(
            Command::new("compile")
                .about("uses circom cli to compile the circom source code")
                .arg(source_file_arg.clone()),
        )
        .subcommand(
            Command::new("generate-witness")
                .about("executes the circom program with inputs from the input.json to generate intermediate witness values")
                .arg(source_file_arg.clone()),
        )
        .subcommand(Command::new("prove").about("generates gkr proof to prove constraint satisfaction").arg(source_file_arg.clone()))
        .subcommand(
            Command::new("verify")
                .about("verify generated gkr proof for constraint satisfaction")
                .arg(source_file_arg.clone()),
        )
        .subcommand(Command::new("prove-verify").about("prove and verify in one step").arg(source_file_arg))
        .get_matches();

    match match_result.subcommand_name() {
        Some("compile") => {
            let source_path = get_source_file(match_result.subcommand_matches("compile").unwrap());
            // let cli_functions = CLIFunctions::new(&source_path);
            dbg!(source_path);
            todo!()
        },
        Some("generate-witness") => {todo!()},
        _ => {}
    }
}

fn get_source_file(matches: &ArgMatches) -> PathBuf {
    matches.get_raw(SOURCE).unwrap().collect::<Vec<_>>()[0].clone().into()
}
