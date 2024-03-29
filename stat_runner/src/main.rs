use crate::gkr::gkr_prove_small_input;
use stat::{end_timer, start_timer};

mod gkr;

fn run<F: Fn() -> ()>(label: &'static str, to_run: F) {
    start_timer!(label);
    to_run();
    end_timer!();
}

fn main() {
    run("gkr_prove_small_input", gkr_prove_small_input);
}
