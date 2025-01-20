use ark_ff::FftField;

// TODO: add documentation
pub fn fft<F: FftField>(coefficients: Vec<F>) -> Vec<F> {
    // n-th root of unity
    let omega = F::get_root_of_unity(coefficients.len() as u64).unwrap();
    fft_internal(coefficients, omega)
}

// TODO: add documentation
pub fn ifft<F: FftField>(evaluations: Vec<F>) -> Vec<F> {
    let n = evaluations.len() as u64;
    // n-th root of unity
    let omega = F::get_root_of_unity(n).unwrap().inverse().unwrap();
    fft_internal(evaluations, omega)
        .into_iter()
        .map(|v| v * F::from(n).inverse().unwrap())
        .collect()
}

pub fn fft_internal<F: FftField>(values: Vec<F>, omega: F) -> Vec<F> {
    if values.len() == 1 {
        return values;
    }

    // TODO: can defer this check to the caller
    // TODO: better help text
    if !values.len().is_power_of_two() {
        panic!("values must be a power of 2");
    }

    let n = values.len();

    let (even, odd) = split_even_odd(values);

    let even_evals = fft_internal(even, omega.square());
    let odd_evals = fft_internal(odd, omega.square());

    let mut evaluations = vec![F::ZERO; n];
    for i in 0..n / 2 {
        evaluations[i] = even_evals[i] + omega.pow([i as u64]) * odd_evals[i];
        evaluations[i + n / 2] = even_evals[i] + omega.pow([(i + n / 2) as u64]) * odd_evals[i];
    }

    evaluations
}

fn split_even_odd<T>(data: Vec<T>) -> (Vec<T>, Vec<T>) {
    let mut even = Vec::with_capacity(data.len() / 2);
    let mut odd = Vec::with_capacity(data.len() / 2);

    for (i, val) in data.into_iter().enumerate() {
        if i % 2 == 0 {
            even.push(val)
        } else {
            odd.push(val);
        }
    }

    (even, odd)
}

#[cfg(test)]
mod tests {
    use ark_ff::{Fp64, MontBackend, MontConfig};

    use super::*;

    //#[derive(MontConfig)]
    //#[modulus = "97"]
    //#[generator = "5"]
    //pub struct FqConfig;
    //pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

    use ark_bls12_377::Fr;
    type Fq = Fr;

    #[test]
    fn test_fft() {
        let a = vec![Fq::from(0), Fq::from(2), Fq::from(34), Fq::from(3434)];
        assert_eq!(ifft(fft(a.clone())), a);
    }
}
