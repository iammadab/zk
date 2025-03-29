use ark_ff::PrimeField;
use polynomial::multilinear::evaluation_form::MultiLinearPolynomial;
use polynomial::multilinear::pairing_index::index_pair;

pub(crate) fn eq_table<F: PrimeField>(r: &[F]) -> Vec<F> {
    let mut result = vec![F::one(); 1 << r.len()];
    for (i, val) in r.iter().enumerate() {
        for (l, r) in index_pair(r.len() as u8, i as u8) {
            result[l] *= F::one() - val;
            result[r] *= val;
        }
    }
    result
}

pub(crate) fn phase_one_table<F: PrimeField>(
    f1_sparse: &[[usize; 3]],
    g_eq: &[F],
    f3_dense: &[F],
) -> Vec<F> {
    let mut result = vec![F::zero(); f3_dense.len()];
    for [z, x, y] in f1_sparse.iter() {
        result[*x] += g_eq[*z] * f3_dense[*y];
    }
    result
}

pub(crate) fn phase_two_table<F: PrimeField>(
    f1_sparse: &[[usize; 3]],
    g_eq: &[F],
    x_eq: &[F],
) -> Vec<F> {
    let mut result = vec![F::zero(); x_eq.len()];
    for [z, x, y] in f1_sparse.iter() {
        result[*y] += g_eq[*z] * x_eq[*x];
    }
    result
}

pub(crate) fn w_i<F: PrimeField>(
    evaluations: &[Vec<F>],
    layer_id: usize,
) -> MultiLinearPolynomial<F> {
    let values = if layer_id == 0 {
        if evaluations[0].len() == 1 {
            vec![evaluations[0][0], F::zero()]
        } else {
            evaluations[0].clone()
        }
    } else {
        evaluations[layer_id].clone()
    };

    MultiLinearPolynomial::new_with_pad(values, None)
}

#[cfg(test)]
mod test {
    use crate::util::eq_table;
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
