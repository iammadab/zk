use ark_ff::{Fp64, MontBackend, MontConfig};

mod dense_merkle_tree;
mod domain;
mod hasher;
mod sparse_merkle_tree;

/// p = 3 * 2^30 + 1
#[derive(MontConfig)]
#[modulus = "3221225473"]
#[generator = "5"]
struct FqConfig;
type Fq = Fp64<MontBackend<FqConfig, 1>>;
