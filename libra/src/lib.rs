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

#[cfg(test)]
mod test {
    use crate::eq_table;
    use ark_bn254::Fr;

    #[test]
    fn test_eq_table_generation() {
        assert_eq!(eq_table(&[Fr::from(3)]), vec![Fr::from(-2), Fr::from(3)]);
        assert_eq!(
            eq_table(&[Fr::from(3), Fr::from(5)]),
            vec![Fr::from(8), Fr::from(-10), Fr::from(-12), Fr::from(15)]
        );
    }
}
