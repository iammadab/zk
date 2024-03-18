use ark_ff::PrimeField;
use polynomial::multilinear_poly::{selector_from_usize, MultiLinearPolynomial};
use polynomial::univariate_poly::UnivariatePolynomial;
use polynomial::Polynomial;

/// Generate a unique line such that l(0) = b and l(1) = c
pub fn l<F: PrimeField>(b: &[F], c: &[F]) -> Result<Vec<UnivariatePolynomial<F>>, &'static str> {
    if b.len() != c.len() {
        return Err("b and c should be the same length");
    }

    // for each pair (b, c) create a straight line t such that
    // t(0) = b and t(1) = c
    // y = mx + b
    // m = (y2 - y1) / (x2 - x1)
    // m = (c - b) / (1 - 0) = (c - b)
    // i.e y = (c - b)x + b
    Ok(b.iter()
        .zip(c.iter())
        .map(|(b, c)| UnivariatePolynomial::new(vec![*b, *c - b]))
        .collect())
}

/// Evaluate a list of univariate polynomial at single point r
pub fn evaluate_l_function<F: PrimeField>(polys: &[UnivariatePolynomial<F>], r: &F) -> Vec<F> {
    polys.iter().map(|poly| poly.evaluate(r)).collect()
}

/// Restrict the domain of the w polynomial to the output of l
/// i.e q(x) = w(l(x))
pub fn q<F: PrimeField>(
    l_functions: &[UnivariatePolynomial<F>],
    w: MultiLinearPolynomial<F>,
) -> Result<UnivariatePolynomial<F>, &'static str> {
    // there should be an l function for each variable in w
    if l_functions.len() != w.n_vars() {
        return Err("output of l should match the number of variables for w");
    }

    // TODO: might be able to optimize this with distributive property
    //  look into univariate poly evaluate (this is equivalent to evaluating a poly
    //  with another poly)
    // compose l functions into w
    // for each variable in w, there is a corresponding l function
    // the goal of this step is to replace each variable with the corresponding l function
    // e.g. w(a, b) = 2ab
    // la = 2x
    // lb = 4x
    // q(x) = 2 * la * lb = 2 * 2x * 4x = 16x^2
    let mut q_poly = UnivariatePolynomial::<F>::additive_identity();
    for (compressed_variables, coeff) in w.coefficients() {
        let mut restricted_term = UnivariatePolynomial::new(vec![coeff]);
        let uncompressed_variables = selector_from_usize(compressed_variables, w.n_vars());
        for (i, is_present) in uncompressed_variables.iter().enumerate() {
            if *is_present {
                restricted_term = &restricted_term * &l_functions[i];
            }
        }
        q_poly = &q_poly + &restricted_term;
    }

    Ok(q_poly)
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_std::{test_rng, UniformRand};

    #[test]
    fn test_l_function() {
        let b = vec![Fr::from(3), Fr::from(2)];
        let c = vec![Fr::from(1), Fr::from(200)];

        let l_functions = l(b.as_slice(), c.as_slice()).expect("should generate successfully");

        // l(0) = b
        assert_eq!(evaluate_l_function(&l_functions, &Fr::from(0)), b);

        // l(1) = c
        assert_eq!(evaluate_l_function(&l_functions, &Fr::from(1)), c);
    }

    #[test]
    fn test_q_poly() {
        // p = 2ab + 3bc
        let p = MultiLinearPolynomial::new(
            3,
            vec![
                (Fr::from(2), vec![true, true, false]),
                (Fr::from(3), vec![false, true, true]),
            ],
        )
        .unwrap();

        // build l function
        let b = vec![Fr::from(1), Fr::from(0), Fr::from(10)];
        let c = vec![Fr::from(23), Fr::from(12), Fr::from(6)];
        let l_functions = l(b.as_slice(), c.as_slice()).unwrap();

        // build q function
        let q = q(l_functions.as_slice(), p.clone()).unwrap();

        // q(x) = w(l(x))
        let x = Fr::rand(&mut test_rng());
        assert_eq!(
            q.evaluate(&x),
            p.evaluate_slice(evaluate_l_function(l_functions.as_slice(), &x).as_slice())
                .unwrap()
        );
    }
}
