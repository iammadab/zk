use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::{selector_from_usize, MultiLinearPolynomial};
use crate::polynomial::univariate_poly::UnivariatePolynomial;
use ark_ff::PrimeField;
use ark_std::iterable::Iterable;

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
pub fn evaluate_l_function<F: PrimeField>(polys: &[UnivariatePolynomial<F>], r: F) -> Vec<F> {
    polys.iter().map(|poly| poly.evaluate(&r)).collect()
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

    // how do we determine what variable belongs to each term in the multilinear polynomial
    // we can decompose the hashmap keys to know what variables we are dealing with
    // then we can perform a reduction to get index based values
    // use those index values for select from the l function
    // then just do univariate multiplication
    // what tools do we have at the disposal now?
    // .coefficients() + selector_from_usize() might be able to do the job
    // will need to test this? How?
    // - create a multilinear polynomial with a couple terms
    // - ensure that the variable mapping gotten back is correct

    // selector_from_usize might be enough

    // TODO: add better comments here
    // let q_poly = additive_identity;
    for (compressed_variables, coeff) in w.coefficients() {
        // let new_term_poly = multiplicative_identity
        let uncompressed_variables = selector_from_usize(compressed_variables, w.n_vars());
        for (i, is_present) in uncompressed_variables.iter().enumerate() {
            if *is_present {}
        }
    }

    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn test_l_function() {
        let b = vec![Fr::from(3), Fr::from(2)];
        let c = vec![Fr::from(1), Fr::from(200)];

        let l_functions = l(b.as_slice(), c.as_slice()).expect("should generate successfully");

        // l(0) = b
        assert_eq!(evaluate_l_function(&l_functions, Fr::from(0)), b);

        // l(1) = c
        assert_eq!(evaluate_l_function(&l_functions, Fr::from(1)), c);
    }
}
