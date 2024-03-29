use crate::gkr::gkr_prove_small_input;
use crate::r1cs_gkr::quadratic_r1cs_gkr_prove;
use stat::{end_timer, start_timer};

mod gkr;
mod r1cs_gkr;

fn run<F: Fn() -> ()>(label: &'static str, to_run: F) {
    start_timer!(label);
    to_run();
    end_timer!();
}

fn main() {
    // run("gkr_prove_small_input", gkr_prove_small_input);
    run("gkr_prove_quadratic_circuit", quadratic_r1cs_gkr_prove);
}
