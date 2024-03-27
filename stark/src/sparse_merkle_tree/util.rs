use std::iter::IntoIterator;

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

/// Return the extra hash values needed to build the tree
pub fn extra_hash_count(leaf_count: usize) -> usize {
    let layer_counts = layer_counts(leaf_count);
    let total_count = layer_counts.into_iter().sum::<usize>();
    total_count - leaf_count
}

/// Return the index of the parent element given the child index
/// (taking into account the sparse merkle tree structure)
pub fn parent(index: usize, layer_counts: Vec<usize>) -> usize {
    // ensure we are not trying to find the parent of the root element
    assert!(index > 0);

    let layer_running_sum = running_sum(layer_counts.into_iter().rev());

    // find the layer the current index belongs, the layer above will be for the parent
    // determine the exact index in the current layer the child belongs
    // use that to extrapolate the index of parent node
    for i in 1..layer_running_sum.len() {
        if layer_running_sum[i] <= index {
            continue;
        }

        let mut diff = index - layer_running_sum[i - 1];
        if diff % 2 == 1 {
            diff -= 1;
        }
        diff /= 2;

        return layer_running_sum[i - 2] + diff;
    }

    // safe to panic here as all the merkle tree implementation will enforce correct arguments
    panic!("invalid index");
}

/// Computes and returns the running sum given an array of values
/// e.g. [1, 2, 3] -> [0, 1, 3, 6]
pub fn running_sum(mut values: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut sum_array = vec![0];
    for value in values {
        sum_array.push(sum_array.last().unwrap() + value)
    }
    sum_array
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

    #[test]
    fn test_extra_hash_count() {
        assert_eq!(extra_hash_count(6), 7);
        assert_eq!(extra_hash_count(10), 13);
    }

    #[test]
    fn test_running_sum() {
        assert_eq!(running_sum(vec![1, 2].into_iter()), vec![0, 1, 3]);
        assert_eq!(
            running_sum(vec![1, 2, 4, 6].into_iter()),
            vec![0, 1, 3, 7, 13]
        );
    }

    #[test]
    fn test_get_parent_index() {
        assert_eq!(parent(21, layer_counts(10)), 11);
        assert_eq!(parent(22, layer_counts(10)), 11);
        assert_eq!(parent(4, layer_counts(10)), 1);
        assert_eq!(parent(2, layer_counts(10)), 0);
        assert_eq!(parent(1, layer_counts(10)), 0);
        assert_eq!(parent(1, layer_counts(6)), 0);
        assert_eq!(parent(11, layer_counts(6)), 5);
    }
}
