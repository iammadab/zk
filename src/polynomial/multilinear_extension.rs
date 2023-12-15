use ark_ff::PrimeField;

// TODO: consider calling this polynomial extension
pub trait MultiLinearExtension<F: PrimeField> {
    /// Returns the number of variables in the extension
    fn n_vars(&self) -> usize;

    /// Assign a value to every variable, return the evaluation
    fn evaluate(&self, assignments: &[F]) -> Result<F, &'static str>;

    // TODO: consider making partial assignments a trait type
    /// Fix certain variables in the extension, return the reduced extension
    fn partial_evaluate(&self, assignments: &[(Vec<bool>, &F)]) -> Result<Self, &'static str>
    where
        Self: Sized;

    /// Remove variables that are no longer used (shrinking the extension)
    /// e.g. 2a + 9c uses three variables [a, b, c] but b is not represented in any term
    /// we can relabel to 2a + 9b uses 2 variables
    fn relabel(self) -> Self;

    /// Convert the extension to a sequence of bytes
    /// mostly used for transcript addition
    fn to_bytes(&self) -> Vec<u8>;
}
