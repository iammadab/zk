/// Performs a low degree extension of the polynomial specified by the given evaluation points
/// and evaluates the provided point at that extension
// TODO: use field arithmetic
fn lde(evaluation_points: Vec<f32>, point: f32) -> f32 {
    // this can be done in a streaming fashion so very space efficient
    // 1. evaluate the first lagrange basis polynomial l0
    // 2. compute l0 * b0, add it to some running total
    // 3. computer l1 from l0, mul with b1, add to total
    // repeat

    // TODO: do something about eval point less than or equal to 1
    if evaluation_points.is_empty() {
        return 0.0;
    }

    let mut result = 0.0;

    // how does one compute l0
    // we need the interpolating point and the given point
    let mut l_0 = 1.0;
    for i in 1..evaluation_points.len() {
        // we need to compute two things
        l_0 *= (point - i as f32) / -(i as f32);
    }
    result += l_0 * evaluation_points[0];

    // once we have l_1..l_n
    for i in 1..evaluation_points.len() {
        l_0 = (l_0 * (point - (i as f32 - 1.0)) * (-(evaluation_points.len() as f32 - i as f32)))
            / ((point - i as f32) * i as f32);
        result += l_0 * evaluation_points[i];
    }

    result
}

#[cfg(test)]
mod tests {
    use super::lde;

    #[test]
    fn lde_1() {
        dbg!(lde(vec![], 6.0));
        dbg!(lde(vec![2.0], 6.0));
        dbg!(lde(vec![2.0, 5.0], 6.0));
    }
}
