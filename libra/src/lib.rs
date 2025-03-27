use ark_ff::PrimeField;
use polynomial::multilinear::pairing_index::index_pair;

fn eq_table<F: PrimeField>(r: &[F]) -> Vec<F> {
    let mut result = vec![F::one(); 1 << r.len()];
    for (i, val) in r.iter().enumerate() {
        for (l, r) in index_pair(r.len() as u8, i as u8) {
            result[l] *= F::one() - val;
            result[r] *= val;
        }
    }
    result
}
