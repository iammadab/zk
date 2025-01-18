use ark_ff::FftField;

// TODO: add documentation
pub fn fft<F: FftField>(coefficients: Vec<F>) -> Vec<F> {
    if coefficients.len() == 1 {
        return coefficients;
    }

    if !coefficients.len().is_power_of_two() {
        panic!("coefficients should be a power of 2");
    }

    let n = coefficients.len();

    let (even, odd) = split_even_odd(coefficients);

    let even_evaluations = fft(even);
    let odd_evaluations = fft(odd);

    // n-th root of unity
    let omega = F::get_root_of_unity(n as u64).unwrap();

    // recombination step
    let mut evaluations = vec![F::ZERO; n];
    for i in 0..n / 2 {
        evaluations[i] = even_evaluations[i] + omega.pow([i as u64]) * odd_evaluations[i];
        evaluations[i + n / 2] =
            even_evaluations[i] + omega.pow([(i + n / 2) as u64]) * odd_evaluations[i];
    }

    evaluations
}

// TODO: add documentation
pub fn ifft<F: FftField>(evaluations: Vec<F>) -> Vec<F> {
    todo!()
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
