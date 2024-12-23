use criterion::{black_box, criterion_group, criterion_main, Criterion};
use polynomial::multilinear::pairing_index_2::index_pair;

fn generate_pair_vector(n_vars: u8, index: u8) -> Vec<(usize, usize)> {
    // f(a, b, c);
    index_pair(n_vars, index).collect()
}

pub fn bench_pair_index(c: &mut Criterion) {
    c.bench_function("pair_index_18_var_12_index", |b| {
        b.iter(|| black_box(generate_pair_vector(18, 12)));
    });
    c.bench_function("pair_index_19_var_12_index", |b| {
        b.iter(|| black_box(generate_pair_vector(19, 12)));
    });
    c.bench_function("pair_index_20_var_12_index", |b| {
        b.iter(|| black_box(generate_pair_vector(20, 12)));
    });
    c.bench_function("pair_index_21_var_12_index", |b| {
        b.iter(|| black_box(generate_pair_vector(21, 12)));
    });
}

criterion_group!(benches, bench_pair_index,);
criterion_main!(benches);
