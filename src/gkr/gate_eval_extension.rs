use crate::multilinear_poly::MultiLinearPolynomial;
use ark_ff::PrimeField;

/// Multivariate Extension structure for gate evaluations in a circuit layer
/// Given three values a, b, c the structure will:
/// - first check if a, b, c is a valid gate in the current layer
/// - then return the evaluation of that gate
/// it is also an extension structure because if a, b, c don't belong in the
/// boolean hypercube, a field element is returned.
struct GateEvalExtension<F: PrimeField> {
    r: Vec<F>,
    add_mle: MultiLinearPolynomial<F>,
    mul_mle: MultiLinearPolynomial<F>,
    w_mle: MultiLinearPolynomial<F>,
}

impl<F: PrimeField> GateEvalExtension<F> {
    fn new(
        r: Vec<F>,
        add_mle: MultiLinearPolynomial<F>,
        mul_mle: MultiLinearPolynomial<F>,
        w_mle: MultiLinearPolynomial<F>,
    ) -> Result<Self, &'static str> {
        // add_mle and mul_mle must have the same variable length
        // proxy signal that they come from the same layer
        if add_mle.n_vars() != mul_mle.n_vars() {
            // only reason they should be different is if one of them has 0 variables
            // this can happen if that gate doesn't exist on the layer at all
            // otherwise then an invariant has been broken
            if add_mle.n_vars() != 0 || mul_mle.n_vars() != 0 {
                return Err("add_mle and mul_mle must come from the same layer");
            }
        }

        // verify the relationship between r, selector_mle and w_mle
        // we want selector_mle(r, b, c)
        // where the size of b and c are len(w_mle) each
        // so total number of variables in selector_mle = len(r) + 2*len(w_mle)

        // have to get max because it's possible for one of them to be 0
        let selector_var_count = add_mle.n_vars().max(mul_mle.n_vars());

        if selector_var_count < 2 * w_mle.n_vars() {
            return Err("selector mle is less than 2 * w_mle, invalid mle's");
        }

        if r.len() != (selector_var_count - (2 * w_mle.n_vars())) {
            return Err("invalid r input length");
        }

        Ok(Self {
            r,
            add_mle,
            mul_mle,
            w_mle,
        })
    }
}

impl<F: PrimeField> GateEvalExtension<F> {
    fn evaluate(&self, b: &[F], c: &[F]) -> Result<F, &'static str> {
        if b.len() != self.w_mle.n_vars() || c.len() != self.w_mle.n_vars() {
            return Err("invalid variable length, b and c should each be the same size as w_mle");
        }

        let mut bc = b.to_vec();
        bc.extend(c.to_vec());

        let add_result = self.add_mle.evaluate(bc.as_slice()).unwrap()
            * (self.w_mle.evaluate(b).unwrap() + self.w_mle.evaluate(c).unwrap());
        let mul_result = self.mul_mle.evaluate(bc.as_slice()).unwrap()
            * (self.w_mle.evaluate(b).unwrap() * self.w_mle.evaluate(c).unwrap());

        Ok(add_result + mul_result)
    }
}
