use ark_bls12_381::Fr;
use gkr::protocol::prove;
use r1cs_gkr::constraint::{Constraint, Term};
use r1cs_gkr::program::R1CSProgram;
use r1cs_gkr::proof::compile_program_and_constrain_witness;
use stat::{end_timer, start_timer};
use ark_std::{test_rng, UniformRand};

pub fn quadratic_checker_circuit() -> R1CSProgram<Fr> {
    // constraints
    // -x * x = -x_squared
    // -a * x_squared = -first_term
    // -b * x = -second_term
    // 0 = first_term + second_term - partial_sum
    // 0 = c + partial_sum - in[0]
    // 0 = res - in[1]
    // 0 = -out + equal_out
    // 0 = -in[0] + in[1] - isz_in
    // 0 = -equal_out + isz_out
    // isz_in . isz_inv = 1 - isz_out
    // isz_in . isz_out = 0

    // symbol index values
    // out = 1
    // x = 2
    // a = 3
    // b = 4
    // c = 5
    // res = 6
    // x_squared = 7
    // first_term = 8
    // second_term = 9
    // partial_sum = 10
    // equal_out = 11
    // in[0] = 12
    // in[1] = 13
    // isz_out = 14
    // isz_in = 15
    // isz_inv = 16

    let constraints = vec![
        Constraint::new(
            vec![Term(2, Fr::from(-1))],
            vec![Term(2, Fr::from(1))],
            vec![Term(7, Fr::from(-1))],
        ),
        Constraint::new(
            vec![Term(3, Fr::from(-1))],
            vec![Term(7, Fr::from(1))],
            vec![Term(8, Fr::from(-1))],
        ),
        Constraint::new(
            vec![Term(4, Fr::from(-1))],
            vec![Term(2, Fr::from(1))],
            vec![Term(9, Fr::from(-1))],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![
                Term(8, Fr::from(1)),
                Term(9, Fr::from(1)),
                Term(10, Fr::from(-1)),
            ],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![
                Term(5, Fr::from(1)),
                Term(10, Fr::from(1)),
                Term(12, Fr::from(-1)),
            ],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![Term(6, Fr::from(1)), Term(13, Fr::from(-1))],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![Term(1, Fr::from(-1)), Term(11, Fr::from(1))],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![
                Term(12, Fr::from(-1)),
                Term(13, Fr::from(1)),
                Term(15, Fr::from(-1)),
            ],
        ),
        Constraint::new(
            vec![],
            vec![],
            vec![Term(11, Fr::from(-1)), Term(14, Fr::from(1))],
        ),
        Constraint::new(
            vec![Term(15, Fr::from(1))],
            vec![Term(16, Fr::from(1))],
            vec![Term(0, Fr::from(1)), Term(14, Fr::from(-1))],
        ),
        Constraint::new(
            vec![Term(15, Fr::from(1))],
            vec![Term(14, Fr::from(1))],
            vec![],
        ),
    ];

    R1CSProgram::new(constraints)
}

pub fn quadratic_r1cs_gkr_prove() {
    let quadratic_progam = quadratic_checker_circuit();
    let mut witness = vec![];

    for _ in 0..16 {
        witness.push(Fr::rand(&mut test_rng()));
    }

    start_timer!("compile r1cs program to gkr circuit + witness construction");
    let (circuit, constrainted_witness) =
        compile_program_and_constrain_witness(quadratic_progam, witness).unwrap();
    end_timer!();

    start_timer!("evaluate the circuit");
    let evaluations = circuit.evaluate(constrainted_witness).unwrap();
    end_timer!();

    start_timer!("prove quadratic program");
    prove(circuit, evaluations).unwrap();
    end_timer!();
}
