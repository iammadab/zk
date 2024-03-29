use ark_bls12_381::Fr;
use gkr::circuit::Circuit;
use gkr::gate::Gate;
use gkr::layer::Layer;
use gkr::protocol::prove;
use stat::{end_timer, start_timer};

pub fn gkr_prove_small_input() {
    start_timer!("build circuit");
    let circuit = test_circuit();
    end_timer!();

    start_timer!("evaluate circuit");
    let input = vec![
        Fr::from(1),
        Fr::from(2),
        Fr::from(3),
        Fr::from(4),
        Fr::from(5),
        Fr::from(6),
        Fr::from(7),
        Fr::from(8),
    ];
    let circut_eval = circuit.evaluate(input.clone()).unwrap();
    end_timer!();

    start_timer!("prove");
    prove(test_circuit(), circut_eval).unwrap();
    end_timer!();
}

fn test_circuit() -> Circuit {
    let layer_0 = Layer::new(vec![Gate::new(0, 0, 1)], vec![]);
    let layer_1 = Layer::new(vec![Gate::new(0, 0, 1)], vec![Gate::new(1, 2, 3)]);
    let layer_2 = Layer::new(
        vec![Gate::new(2, 4, 5), Gate::new(3, 6, 7)],
        vec![Gate::new(0, 0, 1), Gate::new(1, 2, 3)],
    );
    Circuit::new(vec![layer_0, layer_1, layer_2]).unwrap()
}
