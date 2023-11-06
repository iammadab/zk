// TODO: add documentation describing reed solomon fingerprinting

#[cfg(test)]
mod tests {
    use crate::polynomial::Polynomial;
    use ark_bls12_381::Fr;
    use ark_ff::{Fp64, MontBackend, MontConfig, UniformRand};
    use rand::random;

    fn fr_from_vec(values: Vec<i64>) -> Vec<Fr> {
        values.into_iter().map(Fr::from).collect()
    }

    #[test]
    fn reed_solomon_fingerprinting() {
        for i in 0..10000 {
            // randomness
            // TODO: might need a cryptographically secure hash function
            let mut rng = rand::thread_rng();

            // same vector
            // TODO: implement random field vector
            let a = vec![
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
            ];
            let b = a.clone();

            // Reed solomon encoding (take the vector as co-efficients of a polynomial)
            let p_a = Polynomial::new(a);
            let p_b = Polynomial::new(b);

            let random_field_element = Fr::rand(&mut rng);

            let a_eval = p_a.evaluate(&random_field_element);
            let b_eval = p_b.evaluate(&random_field_element);

            // evaluations should be the same
            assert_eq!(a_eval, b_eval);

            // different vector
            let a = vec![
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
            ];
            let b = vec![
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
            ];

            let p_a = Polynomial::new(a);
            let p_b = Polynomial::new(b);

            let random_field_element = Fr::rand(&mut rng);

            let a_eval = p_a.evaluate(&random_field_element);
            let b_eval = p_b.evaluate(&random_field_element);

            // evaluations should be different
            assert_ne!(a_eval, b_eval);
        }
    }
}
