use clap::{command, Arg, ArgMatches, Command};
use protocols::r1cs_gkr::adapters::circom::cli_functions::CLIFunctions;
use std::path::PathBuf;

const SOURCE: &'static str = "source_file";

fn main() {
    let source_file_arg = Arg::new(SOURCE).required(true);

    let match_result = command!()
        .subcommand(
            Command::new("compile")
                .about("Uses circom cli to compile the circom source code")
                .arg(source_file_arg.clone()),
        )
        .subcommand(
            Command::new("generate-witness")
                .about("Executes the circom program with inputs from the input.json to generate intermediate witness values")
                .arg(source_file_arg.clone()),
        )
        .subcommand(Command::new("prove").about("Generates gkr proof to prove constraint satisfaction").arg(source_file_arg.clone()))
        .subcommand(
            Command::new("verify")
                .about("Verify generated gkr proof for constraint satisfaction")
                .arg(source_file_arg.clone()),
        )
        .subcommand(Command::new("prove-verify").about("Prove and verify in one step").arg(source_file_arg))
        .get_matches();

    match match_result.subcommand_name() {
        Some("compile") => run_cli_function(&match_result, "compile"),
        Some("generate-witness") => run_cli_function(&match_result, "generate-witness"),
        Some("prove") => run_cli_function(&match_result, "prove"),
        Some("verify") => run_cli_function(&match_result, "verify"),
        Some("prove-verify") => run_cli_function(&match_result, "prove-verify"),
        _ => {}
    }
}

fn get_source_file(matches: &ArgMatches) -> PathBuf {
    matches.get_raw(SOURCE).unwrap().collect::<Vec<_>>()[0]
        .clone()
        .into()
}

fn run_cli_function(match_result: &ArgMatches, command: &str) {
    let source_path = get_source_file(match_result.subcommand_matches(command).unwrap());
    let cli_functions = CLIFunctions::new(&source_path);
    let execution_result = match command {
        "compile" => cli_functions.compile(),
        "generate-witness" => cli_functions.generate_witness(),
        "prove" => cli_functions.prove(),
        "verify" => cli_functions.verify(),
        "prove-verify" => cli_functions.prove().and_then(|()| cli_functions.verify()),
        _ => unreachable!(),
    };

    if let Err(err_msg) = execution_result {
        println!("error: {}", err_msg);
    }
}
