use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_std::test_rng;
use criterion::{criterion_group, criterion_main, Criterion};
use field_tracker::{end_tscope, print_summary, start_tscope, Ft};
use polynomial::multilinear::evaluation_form::MultiLinearPolynomial;

/// Operation trackable field type
type FTr = Ft!(Fr);

pub fn n_points<F: PrimeField>(n: usize) -> Vec<F> {
    let mut rng = test_rng();
    (0..n).map(|_| F::rand(&mut rng)).collect()
}

fn ark_random_poly_evaluation_pair<F: PrimeField>(
    n_vars: usize,
) -> (DenseMultilinearExtension<F>, Vec<F>) {
    let total_n_points = 2_i32.pow(n_vars as u32);
    let poly_evaluations = n_points(total_n_points as usize);
    let to_eval = n_points(n_vars);
    (
        DenseMultilinearExtension::from_evaluations_vec(n_vars, poly_evaluations),
        to_eval,
    )
}

fn poly_eval_pair<F: PrimeField>(n_vars: usize) -> (MultiLinearPolynomial<F>, Vec<F>) {
    let total_n_points = 2_i32.pow(n_vars as u32);
    let poly_evaluations = n_points(total_n_points as usize);
    let to_eval = n_points(n_vars);
    (
        MultiLinearPolynomial::new(n_vars, poly_evaluations).unwrap(),
        to_eval,
    )
}

pub fn arkworks_benchmark(c: &mut Criterion) {
    c.bench_function("arkworks_evaluate_18_vars", |b| {
        let (poly, to_eval) = ark_random_poly_evaluation_pair::<Fr>(18);
        b.iter(|| poly.fix_variables(to_eval.as_slice()))
    });

    c.bench_function("arkworks_evaluate_19_vars", |b| {
        let (poly, to_eval) = ark_random_poly_evaluation_pair::<Fr>(19);
        b.iter(|| poly.fix_variables(to_eval.as_slice()))
    });

    c.bench_function("arkworks_evaluate_20_vars", |b| {
        let (poly, to_eval) = ark_random_poly_evaluation_pair::<Fr>(20);
        b.iter(|| poly.fix_variables(to_eval.as_slice()))
    });

    c.bench_function("arkworks_evaluate_21_vars", |b| {
        let (poly, to_eval) = ark_random_poly_evaluation_pair::<Fr>(21);
        b.iter(|| poly.fix_variables(to_eval.as_slice()))
    });
}

pub fn arkworks_field_op_benchmark(_c: &mut Criterion) {
    start_tscope!("arkworks_evaluate");
    start_tscope!("poly_eval 18var");
    let (poly, to_eval) = ark_random_poly_evaluation_pair::<FTr>(18);
    poly.fix_variables(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 19var");
    let (poly, to_eval) = ark_random_poly_evaluation_pair::<FTr>(19);
    poly.fix_variables(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 20var");
    let (poly, to_eval) = ark_random_poly_evaluation_pair::<FTr>(20);
    poly.fix_variables(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 21var");
    let (poly, to_eval) = ark_random_poly_evaluation_pair::<FTr>(21);
    poly.fix_variables(&to_eval.as_slice());
    end_tscope!();
    end_tscope!();
}

pub fn poly_eval_benchmark(c: &mut Criterion) {
    c.bench_function("evaluate_18_vars", |b| {
        let (poly, to_eval) = poly_eval_pair::<Fr>(18);
        b.iter(|| poly.evaluate(to_eval.as_slice()))
    });

    c.bench_function("evaluate_19_vars", |b| {
        let (poly, to_eval) = poly_eval_pair::<Fr>(19);
        b.iter(|| poly.evaluate(to_eval.as_slice()))
    });

    c.bench_function("evaluate_20_vars", |b| {
        let (poly, to_eval) = poly_eval_pair::<Fr>(20);
        b.iter(|| poly.evaluate(to_eval.as_slice()))
    });

    c.bench_function("evaluate_21_vars", |b| {
        let (poly, to_eval) = poly_eval_pair::<Fr>(21);
        b.iter(|| poly.evaluate(to_eval.as_slice()))
    });
}

pub fn poly_field_op_benchmark(_c: &mut Criterion) {
    start_tscope!("poly_evaluate");
    start_tscope!("poly_eval 18var");
    let (poly, to_eval) = poly_eval_pair::<FTr>(18);
    poly.evaluate(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 19var");
    let (poly, to_eval) = poly_eval_pair::<FTr>(19);
    poly.evaluate(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 20var");
    let (poly, to_eval) = poly_eval_pair::<FTr>(20);
    poly.evaluate(&to_eval.as_slice());
    end_tscope!();

    start_tscope!("poly_eval 21var");
    let (poly, to_eval) = poly_eval_pair::<FTr>(21);
    poly.evaluate(&to_eval.as_slice());
    end_tscope!();
    end_tscope!();

    print_summary!();
}

criterion_group!(
    benches,
    arkworks_field_op_benchmark,
    poly_field_op_benchmark,
    poly_eval_benchmark,
    arkworks_benchmark
);

criterion_main!(benches);
