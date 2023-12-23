use crate::polynomial::univariate_poly::UnivariatePolynomial;
use ark_ff::PrimeField;
use ark_std::iterable::Iterable;

// TODO: rename this function
// TODO: add documentation
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

// TODO: add documentation
// TODO: did you rename the other function
pub fn evaluate_l_function<F: PrimeField>(polys: &[UnivariatePolynomial<F>], r: F) -> Vec<F> {
    polys.iter().map(|poly| poly.evaluate(&r)).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    // TODO: did you rename the function eventually?
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
