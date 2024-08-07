use ark_ff::{BigInteger, PrimeField};
use polynomial::multilinear::coefficient_form::{
    selector_from_position, CoeffMultilinearPolynomial,
};
use polynomial::univariate_poly::UnivariatePolynomial;
use polynomial::Polynomial;
use std::ops::Add;

#[derive(Clone, Debug, PartialEq)]
/// Multivariate Extension structure for gate evaluations in a circuit layer
/// Given three values a, b, c the structure will:
/// - first check if a, b, c is a valid gate in the current layer
/// - then return the evaluation of that gate
/// it is also an extension structure because if a, b, c don't belong in the
/// boolean hypercube, a field element is returned.
pub struct GateEvalExtension<F: PrimeField> {
    r: Vec<F>,
    add_mle: Vec<CoeffMultilinearPolynomial<F>>,
    mul_mle: Vec<CoeffMultilinearPolynomial<F>>,
    w_b_mle: Vec<CoeffMultilinearPolynomial<F>>,
    w_c_mle: Vec<CoeffMultilinearPolynomial<F>>,
}

impl<F: PrimeField> GateEvalExtension<F> {
    pub fn new(
        r: Vec<F>,
        add_mle: CoeffMultilinearPolynomial<F>,
        mul_mle: CoeffMultilinearPolynomial<F>,
        w_mle: CoeffMultilinearPolynomial<F>,
    ) -> Result<Self, &'static str> {
        // add_mle and mul_mle must have the same variable length
        // proxy signal that they come from the same layer
        if add_mle.n_vars() != mul_mle.n_vars() {
            // only reason they should be different is if one of them has 0 variables
            // this can happen if that gate doesn't exist on the layer at all
            // otherwise then an invariant has been broken
            if add_mle.n_vars() != 0 && mul_mle.n_vars() != 0 {
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
            add_mle: vec![add_mle],
            mul_mle: vec![mul_mle],
            w_b_mle: vec![w_mle.clone()],
            w_c_mle: vec![w_mle],
        })
    }

    fn is_additive_identity(&self) -> bool {
        if self.r.is_empty()
            && self.add_mle.is_empty()
            && self.mul_mle.is_empty()
            && self.w_b_mle.is_empty()
            && self.w_c_mle.is_empty()
        {
            return true;
        }
        false
    }

    /// Returns the number of f(b, c) function in the struct
    fn count(&self) -> usize {
        self.add_mle.len()
    }

    /// Returns the number of variables for b in f(b, c)
    fn b_vars(&self) -> usize {
        self.w_b_mle.first().map(|p| p.n_vars()).unwrap_or(0)
    }

    /// Returns the number of variables for c in f(b, c)
    fn c_vars(&self) -> usize {
        self.w_c_mle.first().map(|p| p.n_vars()).unwrap_or(0)
    }
}

impl<F: PrimeField> Polynomial<F> for GateEvalExtension<F> {
    fn n_vars(&self) -> usize {
        // n vars = |b| + |c|
        self.b_vars() + self.c_vars()
    }

    fn evaluate_slice(&self, assignments: &[F]) -> Result<F, &'static str> {
        if assignments.len() != self.n_vars() {
            return Err("invalid assignment length, should be twice the size of w_mle.n_vars()");
        }

        let mut evaluation_result = F::zero();

        let mut rbc = self.r.clone();
        rbc.extend(assignments.to_vec());

        for i in 0..self.count() {
            let mid = self.w_b_mle[i].n_vars();
            let b_val = self.w_b_mle[i].evaluate_slice(&assignments[..mid]).unwrap();
            let c_val = self.w_c_mle[i].evaluate_slice(&assignments[mid..]).unwrap();

            let add_result =
                self.add_mle[i].evaluate_slice(rbc.as_slice()).unwrap() * (b_val + c_val);
            let mul_result =
                self.mul_mle[i].evaluate_slice(rbc.as_slice()).unwrap() * (b_val * c_val);

            evaluation_result += add_result + mul_result;
        }

        Ok(evaluation_result)
    }

    fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        // partial evaluate add_mle and mul_mle
        // they expect r as first input before b and c
        // so we have to pad all the partial evaluation assignments

        let mut result = self.clone();

        // TODO: clean this up, might not need to clone this much
        let rbc_partial_assignments = assignments
            .iter()
            .map(|(selector, coeff)| {
                let mut new_selector = vec![false; self.r.len()];
                new_selector.extend(selector);
                (new_selector, *coeff)
            })
            .collect::<Vec<(Vec<bool>, &F)>>();

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

        for i in 0..self.count() {
            result.add_mle[i] =
                result.add_mle[i].partial_evaluate(rbc_partial_assignments.as_slice())?;
            result.mul_mle[i] =
                result.mul_mle[i].partial_evaluate(rbc_partial_assignments.as_slice())?;

            result.w_b_mle[i] =
                result.w_b_mle[i].partial_evaluate(b_partial_assignments.as_slice())?;
            result.w_c_mle[i] =
                result.w_c_mle[i].partial_evaluate(c_partial_assignments.as_slice())?;
        }

        Ok(result)
    }

    fn to_univariate(&self) -> Result<UnivariatePolynomial<F>, &'static str> {
        if self.n_vars() > 1 {
            return Err(
                "cannot create univariate poly from gate eval extension with more than 1 variable",
            );
        }

        // TODO: add test for n_vars = 0

        let mut result = UnivariatePolynomial::<F>::additive_identity();

        // create r_assignments for partial evaluation
        let mut r_assignments = vec![];
        for (i, r) in self.r.iter().enumerate() {
            r_assignments.push((selector_from_position(self.r.len() + 1, i)?, r))
        }

        for i in 0..self.count() {
            // TODO: might need to relabel before hand
            // partially evaluate add_mle and mul_mle at r
            let add_mle_uni = self.add_mle[i]
                .partial_evaluate(r_assignments.as_slice())?
                .relabel()
                .to_univariate()?;
            let mul_mle_uni = self.mul_mle[i]
                .partial_evaluate(r_assignments.as_slice())?
                .relabel()
                .to_univariate()?;
            // TODO: do we need to relabel here
            //  figure out if you need to
            let w_b_uni = self.w_b_mle[i].to_univariate()?;
            let w_c_uni = self.w_c_mle[i].to_univariate()?;

            let add_result_uni = &add_mle_uni * &(&w_b_uni + &w_c_uni);
            let mul_result_uni = &mul_mle_uni * &(&w_b_uni * &w_c_uni);
            let f_x = &add_result_uni + &mul_result_uni;

            result = &result + &f_x;
        }

        Ok(result)
    }

    fn relabel(self) -> Self {
        GateEvalExtension {
            r: self.r,
            add_mle: self.add_mle.into_iter().map(|p| p.relabel()).collect(),
            mul_mle: self.mul_mle.into_iter().map(|p| p.relabel()).collect(),
            w_b_mle: self.w_b_mle.into_iter().map(|p| p.relabel()).collect(),
            w_c_mle: self.w_c_mle.into_iter().map(|p| p.relabel()).collect(),
        }
    }

    fn additive_identity() -> Self {
        Self {
            r: vec![],
            add_mle: vec![],
            mul_mle: vec![],
            w_b_mle: vec![],
            w_c_mle: vec![],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.r.iter().fold(vec![], |mut acc, r_v| {
            acc.extend(r_v.into_bigint().to_bytes_be());
            acc
        });
        self.add_mle
            .iter()
            .for_each(|p| result.extend(p.to_bytes()));
        self.mul_mle
            .iter()
            .for_each(|p| result.extend(p.to_bytes()));
        self.w_b_mle
            .iter()
            .for_each(|p| result.extend(p.to_bytes()));
        self.w_c_mle
            .iter()
            .for_each(|p| result.extend(p.to_bytes()));
        result
    }
}

impl<F: PrimeField> Add for &GateEvalExtension<F> {
    type Output = Result<GateEvalExtension<F>, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_additive_identity() {
            return Ok(rhs.clone());
        }

        if rhs.is_additive_identity() {
            return Ok(self.clone());
        }

        // check that all polynomial have the same signature
        if self.r != rhs.r {
            return Err("cannot add gate extensions with different r values");
        }
        if (self.b_vars() != rhs.b_vars()) || (self.c_vars() != rhs.c_vars()) {
            return Err("cannot add gate extensions with different polynomial signatures");
        }

        let mut new_add_mle = self.add_mle.clone();
        new_add_mle.extend(rhs.add_mle.clone());

        let mut new_mul_mle = self.mul_mle.clone();
        new_mul_mle.extend(rhs.mul_mle.clone());

        let mut new_w_b_mle = self.w_b_mle.clone();
        new_w_b_mle.extend(rhs.w_b_mle.clone());

        let mut new_w_c_mle = self.w_c_mle.clone();
        new_w_c_mle.extend(rhs.w_c_mle.clone());

        Ok(GateEvalExtension {
            r: self.r.clone(),
            add_mle: new_add_mle,
            mul_mle: new_mul_mle,
            w_b_mle: new_w_b_mle,
            w_c_mle: new_w_c_mle,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::circuit::tests::test_circuit;
    use crate::circuit::Circuit;
    use crate::gate_eval_extension::GateEvalExtension;
    use ark_bls12_381::Fr;
    use polynomial::Polynomial;
    use sumcheck::util::sum_over_boolean_hyper_cube;
    use sumcheck::Sumcheck;

    fn evaluated_circuit() -> (Circuit, Vec<Vec<Fr>>) {
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
        (circuit, circuit_eval)
    }

    #[test]
    fn test_gate_eval_extension() {
        let (circuit, circuit_eval) = evaluated_circuit();

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
                .evaluate_slice(&[Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(1)])
                .unwrap(),
            Fr::from(14)
        );
        assert_eq!(sum_over_boolean_hyper_cube(&gate_eval_ext), Fr::from(14));

        // setting r = 1
        let gate_eval_ext = GateEvalExtension::new(vec![Fr::from(1)], add_1, mul_1, w_2).unwrap();
        // eval at b = 2, and c = 3, expected result = 165
        assert_eq!(
            gate_eval_ext
                .evaluate_slice(&[Fr::from(1), Fr::from(0), Fr::from(1), Fr::from(1)])
                .unwrap(),
            Fr::from(165)
        );
        assert_eq!(sum_over_boolean_hyper_cube(&gate_eval_ext), Fr::from(165));
    }

    #[test]
    fn test_partial_evaluation() {
        let (circuit, circuit_eval) = evaluated_circuit();

        let [add_1, mul_1] = circuit.add_mul_mle::<Fr>(1).unwrap();
        let w_2 = Circuit::w(circuit_eval.as_slice(), 2).unwrap();

        let gate_eval_ext = GateEvalExtension::new(vec![Fr::from(10)], add_1, mul_1, w_2).unwrap();
        assert_eq!(gate_eval_ext.n_vars(), 4);

        // first we perform a full evaluation to get the expected result
        assert_eq!(
            gate_eval_ext
                .evaluate_slice(&[Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4)])
                .unwrap(),
            Fr::from(6840)
        );

        // now we partially evaluate with the same values
        let p1 = gate_eval_ext
            .partial_evaluate(&[(vec![true, false, false, false], &Fr::from(1))])
            .unwrap();
        let p2 = p1
            .partial_evaluate(&[(vec![false, true, false, false], &Fr::from(2))])
            .unwrap();
        assert_eq!(p2.n_vars(), 4);
        let p2 = p2.relabel();
        assert_eq!(p2.n_vars(), 2);

        let p3 = p2
            .partial_evaluate(&[(vec![true, false], &Fr::from(3))])
            .unwrap();
        assert_eq!(p3.n_vars(), 2);
        let p3 = p3.relabel();
        assert_eq!(p3.n_vars(), 1);

        let p4 = p3.partial_evaluate(&[(vec![true], &Fr::from(4))]).unwrap();
        assert_eq!(p4.n_vars(), 1);
        let p4 = p4.relabel();
        assert_eq!(p4.n_vars(), 0);

        assert_eq!(p4.evaluate_slice(&[]).unwrap(), Fr::from(6840));
    }

    #[test]
    fn test_to_univariate() {
        let (circuit, circuit_eval) = evaluated_circuit();

        let [add_1, mul_1] = circuit.add_mul_mle::<Fr>(1).unwrap();
        let w_2 = Circuit::w(circuit_eval.as_slice(), 2).unwrap();

        let gate_eval_ext = GateEvalExtension::new(vec![Fr::from(10)], add_1, mul_1, w_2).unwrap();
        assert_eq!(gate_eval_ext.n_vars(), 4);

        // eval b1, c1, and c2
        let p1 = gate_eval_ext
            .partial_evaluate(&[
                (vec![true, false, false, false], &Fr::from(1)),
                (vec![false, false, true, false], &Fr::from(2)),
                (vec![false, false, false, true], &Fr::from(3)),
            ])
            .unwrap()
            .relabel();
        assert_eq!(p1.n_vars(), 1);
        assert_eq!(p1.mul_mle[0].n_vars(), p1.r.len() + 1);
        assert_eq!(p1.add_mle[0].n_vars(), p1.r.len() + 1);
        assert_eq!(p1.w_b_mle[0].n_vars(), 1);
        assert_eq!(p1.w_c_mle[0].n_vars(), 0);

        let evaluation = p1.evaluate_slice(&[Fr::from(12)]).unwrap();

        let p1_univariate = p1.to_univariate().unwrap();
        let uni_evaluation = p1_univariate.evaluate(&Fr::from(12));

        assert_eq!(evaluation, uni_evaluation);
    }

    #[test]
    fn test_sum_check_eval() {
        let (circuit, circuit_eval) = evaluated_circuit();
        let [add_1, mul_1] = circuit.add_mul_mle::<Fr>(1).unwrap();
        let w_2 = Circuit::w(circuit_eval.as_slice(), 2).unwrap();
        let gate_eval_ext = GateEvalExtension::new(vec![Fr::from(0)], add_1, mul_1, w_2).unwrap();

        // sum over boolean hypercube = 14
        assert_eq!(sum_over_boolean_hyper_cube(&gate_eval_ext), Fr::from(14));

        // generate false sumcheck_old proof
        let false_proof = Sumcheck::prove(gate_eval_ext.clone(), Fr::from(20));
        assert!(!Sumcheck::verify(false_proof));

        // generate correct sumcheck_old proof
        let correct_proof = Sumcheck::prove(gate_eval_ext, Fr::from(14));
        assert!(Sumcheck::verify(correct_proof));
    }
}
