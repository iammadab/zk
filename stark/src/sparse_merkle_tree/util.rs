/// Returns the sibling index of a node
pub fn sibling(index: usize) -> usize {
    if index == 0 {
        0
    } else if index % 2 == 0 {
        index - 1
    } else {
        index + 1
    }
}

/// Returns the number of elements on each layer given the number of leaves
/// [leaf_layer_count, leaf_layer_minus_1_count, ..., root_layer_count]
pub fn layer_counts(mut leaf_count: usize) -> Vec<usize> {
    // ensure the leaf_count is even
    assert_eq!(leaf_count % 2, 0);

    let mut counts = vec![leaf_count];

    // layers with an even number of element can be reduced to the next layer
    // after reduction (n / 2) if the result is odd we need to add 1 so this
    // new layer can be further reduced.
    // we keep doing this until the layer length = 1
    while leaf_count != 1 {
        leaf_count = leaf_count / 2;
        if leaf_count % 2 == 1 && leaf_count != 1 {
            leaf_count += 1;
        }
        counts.push(leaf_count);
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sibling_index() {
        assert_eq!(sibling(4), 3);
        assert_eq!(sibling(1), 2);
        assert_eq!(sibling(0), 0);
    }

    #[test]
    fn test_get_layer_counts() {
        assert_eq!(layer_counts(6), vec![6, 4, 2, 1]);
        assert_eq!(layer_counts(10), vec![10, 6, 4, 2, 1]);
        assert_eq!(layer_counts(26), vec![26, 14, 8, 4, 2, 1]);
    }
}
