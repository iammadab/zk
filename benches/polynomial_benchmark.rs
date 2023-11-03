use ark_ff::{Fp64, MontBackend, MontConfig};
use criterion::{criterion_group, criterion_main, Criterion};
use thaler::polynomial;
use thaler::polynomial::Polynomial;

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

fn polynomial_evaluation_benchmark(c: &mut Criterion) {
    let poly = Polynomial::new(vec![
        Fq::from(12),
        Fq::from(25),
        Fq::from(18),
        Fq::from(24),
        Fq::from(12),
        Fq::from(8),
    ]);
    c.bench_function("polynomial evaluation", |b| {
        b.iter(|| poly.evaluate(&Fq::from(16)));
    });
}

criterion_group!(benches, polynomial_evaluation_benchmark);
criterion_main!(benches);
