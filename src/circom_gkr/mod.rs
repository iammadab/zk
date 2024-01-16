use ark_ff::PrimeField;

/// Represents the constraint operation
enum Operation {
    Add,
    Mul
}

/// Contains a pointer to a variable and field element to mul the
/// variable's value with.
/// e.g. let [s1, s2, s3] be the set of variables
/// ProductArg(1, -1)
///     will get the value stored in s2 (e.g. 5) and mul that by -1
///     = -5
/// This is the building block for representing R1Cs constraints
struct ProductArg<F: PrimeField>(usize, F);

/// Represents a single R1CS constraint
/// As . Bs = Cs
/// where s contains the witness and constants
struct Constraint<F: PrimeField> {
    a: Vec<ProductArg<F>>,
    b: Vec<ProductArg<F>>,
    c: Vec<ProductArg<F>>,
    op: Operation
}

/// Simplified constraint that contains at most 3 operations
/// and at most 1 operation type
struct ReducedConstraint<F: PrimeField> {
    a: ProductArg<F>,
    b: ProductArg<F>,
    c: ProductArg<F>,
    op: Operation
}