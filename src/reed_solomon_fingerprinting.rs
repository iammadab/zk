// Alice and Bob have a large file represented as a vector A, B respectively
// each consisting of n values.
// The goal is to determine if both files are equal while minimizing communication cost
// The naive solution of just sending the set A to Bob works but communication cost is too high
//
// to solve this, Alice and Bob convert their vectors to polynomials
// alice chooses a random point in some large field and evaluates her polynomial at that point
// alice send (r, p_a(r)) to bob i.e the random point and the evaluation at that random point
// bob evaluates his polynomial p_b at  r  and checks if the evaluations match.
// if they do, then they have the same file if they don't then they have different files
//
// This is based on the fact that 2 polynomials of at most degree d can only agree at
// d points, unless the polynomials are the same.
// If the evaluation domain is a lot larger than d then the probability of picking one of
// the d points is  d / evaluation_domain. (this is known as the soundness error)
// i.e probability of getting deceived.

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
