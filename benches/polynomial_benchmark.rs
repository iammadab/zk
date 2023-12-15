use ark_ff::{Fp64, MontBackend, MontConfig};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::str::CharIndices;
use thaler::polynomial::univariate_poly::UnivariatePolynomial;

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

fn poly_a() -> UnivariatePolynomial<Fq> {
    UnivariatePolynomial::new(vec![
        Fq::from(12),
        Fq::from(25),
        Fq::from(18),
        Fq::from(24),
        Fq::from(12),
        Fq::from(8),
    ])
}

fn poly_b() -> UnivariatePolynomial<Fq> {
    UnivariatePolynomial::new(vec![Fq::from(4), Fq::from(3), Fq::from(2)])
}

fn polynomial_evaluation_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial evaluation", |b| {
        let poly = poly_a();
        b.iter(|| poly.evaluate(&Fq::from(16)));
    });
}

fn polynomial_addition_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial addition", |b| {
        let poly_a = poly_a();
        let poly_b = poly_b();
        b.iter(|| &poly_a + &poly_b)
    });
}

fn polynomial_multiplication_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial multiplication", |b| {
        let poly_a = poly_a();
        let poly_b = poly_b();
        b.iter(|| &poly_a * &poly_b);
    });
}

fn polynomial_interpolation_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial interpolation", |b| {
        b.iter(|| {
            UnivariatePolynomial::interpolate_xy(
                vec![
                    Fq::from(0),
                    Fq::from(1),
                    Fq::from(3),
                    Fq::from(4),
                    Fq::from(5),
                    Fq::from(8),
                ],
                vec![
                    Fq::from(12),
                    Fq::from(48),
                    Fq::from(3158),
                    Fq::from(11772),
                    Fq::from(33452),
                    Fq::from(315020),
                ],
            );
        })
    });
}

criterion_group!(
    benches,
    polynomial_evaluation_benchmark,
    polynomial_addition_benchmark,
    polynomial_multiplication_benchmark,
    polynomial_interpolation_benchmark
);
criterion_main!(benches);
