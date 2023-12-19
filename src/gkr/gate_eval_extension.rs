use crate::gkr::gate::Gate;
use crate::polynomial::multilinear_extension::MultiLinearExtension;
use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
use ark_ff::{BigInteger, PrimeField};
use std::ops::Add;

#[derive(Clone, Debug, PartialEq)]
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
    w_b_mle: MultiLinearPolynomial<F>,
    w_c_mle: MultiLinearPolynomial<F>,
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
            w_b_mle: w_mle.clone(),
            w_c_mle: w_mle,
        })
    }

    /// Return the number of variables for b in f(b, c)
    fn b_vars(&self) -> usize {
        self.w_b_mle.n_vars()
    }

    /// Return the number of variables for c in f(b, c)
    fn c_vars(&self) -> usize {
        self.w_c_mle.n_vars()
    }
}

impl<F: PrimeField> MultiLinearExtension<F> for GateEvalExtension<F> {
    fn n_vars(&self) -> usize {
        // n vars = |b| + |c|
        self.b_vars() + self.c_vars()
    }

    fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars() {
            return Err("invalid assignment length, should be twice the size of w_mle.n_vars()");
        }

        let mut rbc = self.r.clone();
        rbc.extend(assignments.to_vec());

        let mid = self.w_b_mle.n_vars();
        let b_val = self.w_b_mle.evaluate(&assignments[..mid]).unwrap();
        let c_val = self.w_c_mle.evaluate(&assignments[mid..]).unwrap();

        let add_result = self.add_mle.evaluate(rbc.as_slice()).unwrap() * (b_val + c_val);
        let mul_result = self.mul_mle.evaluate(rbc.as_slice()).unwrap() * (b_val * c_val);

        Ok(add_result + mul_result)
    }

    fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        // partial evaluate add_mle and mul_mle
        // they expect r as first input before b and c
        // so we have to pad all the partial evaluation assignments
        // TODO: clean this up, might not need to clone this much
        let rbc_partial_assignments = assignments
            .iter()
            .map(|(selector, coeff)| {
                let mut new_selector = vec![false; self.r.len()];
                new_selector.extend(selector);
                (new_selector, *coeff)
            })
            .collect::<Vec<(Vec<bool>, &F)>>();

        let new_add_mle = self
            .add_mle
            .partial_evaluate(rbc_partial_assignments.as_slice())?;
        let new_mul_mle = self
            .mul_mle
            .partial_evaluate(rbc_partial_assignments.as_slice())?;

        // next partial eval for w_b_mle and w_c_mle
        // need to split assignments for b and c
        let b_boundary = self.b_vars();
        let mut b_partial_assignments = vec![];
        let mut c_partial_assignments = vec![];

        for (selector, coeff) in assignments {
            let b_selector = &selector[..b_boundary];
            let c_selector = &selector[b_boundary..];

            if b_selector.iter().any(|v| *v) {
                b_partial_assignments.push((b_selector.to_vec(), *coeff))
            } else {
                c_partial_assignments.push((c_selector.to_vec(), *coeff))
            }
        }

        let new_w_b_mle = self
            .w_b_mle
            .partial_evaluate(b_partial_assignments.as_slice())?;
        let new_w_c_mle = self
            .w_c_mle
            .partial_evaluate(c_partial_assignments.as_slice())?;

        Ok(GateEvalExtension {
            r: self.r.clone(),
            add_mle: new_add_mle,
            mul_mle: new_mul_mle,
            w_b_mle: new_w_b_mle,
            w_c_mle: new_w_c_mle,
        })
    }

    fn relabel(self) -> Self {
        // TODO: test this
        // TODO: understand this
        GateEvalExtension {
            r: self.r,
            add_mle: self.add_mle.relabel(),
            mul_mle: self.mul_mle.relabel(),
            w_b_mle: self.w_b_mle.relabel(),
            w_c_mle: self.w_c_mle.relabel(),
        }
    }

    fn additive_identity() -> Self {
        Self {
            r: vec![],
            add_mle: MultiLinearPolynomial::<F>::additive_identity(),
            mul_mle: MultiLinearPolynomial::<F>::additive_identity(),
            w_b_mle: MultiLinearPolynomial::<F>::additive_identity(),
            w_c_mle: MultiLinearPolynomial::<F>::additive_identity(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.r.iter().fold(vec![], |mut acc, r_v| {
            acc.extend(r_v.into_bigint().to_bytes_be());
            acc
        });
        result.extend(self.add_mle.to_bytes());
        result.extend(self.mul_mle.to_bytes());
        result.extend(self.w_b_mle.to_bytes());
        result.extend(self.w_c_mle.to_bytes());
        result
    }
}

// TODO: test gate eval addition
// TODO: test gate eval addition with additive identity
impl<F: PrimeField> Add for &GateEvalExtension<F> {
    type Output = Result<GateEvalExtension<F>, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.r != rhs.r {
            // they are only allowed to be non equal if one of them is empty
            if !self.r.is_empty() && !rhs.r.is_empty() {
                return Err("cannot add gate extensions with different r values");
            }
        }

        // addition is just the sum of the individual polynomials
        Ok(GateEvalExtension {
            r: self.r.clone(),
            add_mle: (&self.add_mle + &rhs.add_mle)?,
            mul_mle: (&self.mul_mle + &rhs.mul_mle)?,
            w_b_mle: (&self.w_b_mle + &rhs.w_b_mle)?,
            w_c_mle: (&self.w_c_mle + &rhs.w_c_mle)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::gkr::circuit::tests::test_circuit;
    use crate::gkr::circuit::Circuit;
    use crate::gkr::gate_eval_extension::GateEvalExtension;
    use crate::polynomial::multilinear_extension::MultiLinearExtension;
    use crate::polynomial::multilinear_poly::MultiLinearPolynomial;
    use crate::sumcheck::util::sum_over_boolean_hyper_cube;
    use ark_bls12_381::Fr;

    #[test]
    fn test_gate_eval_extension() {
        // construct and evaluate circuit
        let circuit = test_circuit();
        let circuit_eval = circuit
            .evaluate(vec![
                Fr::from(1),
                Fr::from(2),
                Fr::from(3),
                Fr::from(4),
                Fr::from(5),
                Fr::from(6),
                Fr::from(7),
                Fr::from(8),
            ])
            .unwrap();

        // construct relevant mles
        // add and mul mle from layer 1, (a, b, c) -> (len(1), len(2), len(2)) -> 5 total variables
        let [add_1, mul_1] = circuit.add_mul_mle::<Fr>(1).unwrap();
        // w_mle from layer 2 with len(2)
        let w_2 = Circuit::w(circuit_eval.as_slice(), 2).unwrap();

        // setting r = 0
        let gate_eval_ext =
            GateEvalExtension::new(vec![Fr::from(0)], add_1.clone(), mul_1.clone(), w_2.clone())
                .unwrap();
        // eval at b = 0 and c = 1, expected result = 14
        assert_eq!(
            gate_eval_ext
                .evaluate(&[Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(1)])
                .unwrap(),
            Fr::from(14)
        );
        assert_eq!(sum_over_boolean_hyper_cube(&gate_eval_ext), Fr::from(14));

        // setting r = 1
        let gate_eval_ext = GateEvalExtension::new(vec![Fr::from(1)], add_1, mul_1, w_2).unwrap();
        // eval at b = 2, and c = 3, expected result = 165
        assert_eq!(
            gate_eval_ext
                .evaluate(&[Fr::from(1), Fr::from(0), Fr::from(1), Fr::from(1)])
                .unwrap(),
            Fr::from(165)
        );
        assert_eq!(sum_over_boolean_hyper_cube(&gate_eval_ext), Fr::from(165));
    }
}
