/// Returns the sibling index of a node
fn sibling(index: usize) -> usize {
    if index == 0 {
        0
    } else if index % 2 == 0 {
        index - 1
    } else {
        index + 1
    }
}

/// Return parent index of a node
fn parent(index: usize) -> usize {
    (index - 1) / 2
}

/// Return the number of extra hash data needed to build a merkle tree
fn extra_hash_count(leaf_count: usize) -> usize {
    leaf_count - 1
}

/// Determines if a given value is a power of 2
fn is_power_of_2(value: usize) -> bool {
    if value == 0 {
        return false;
    }

    value & (value - 1) == 0
}

/// Returns the next power of 2 from a number
fn next_power_of_2(mut value: usize) -> usize {
    // TODO: there has to be a better way to do this
    // with bitwise operations most likely
    while !is_power_of_2(value) {
       value += 1;
    }
    value
}

// TODO
// fn make_power_of_two<T: Clone>(input: &mut [T]) -> &[T] {}

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
    fn test_get_parent_index() {
        assert_eq!(parent(1), 0);
        assert_eq!(parent(2), 0);
        assert_eq!(parent(11), 5);
        assert_eq!(parent(13), 6);
    }

    #[test]
    fn test_is_power_of_2() {
        assert_eq!(is_power_of_2(1), true);
        assert_eq!(is_power_of_2(2), true);
        assert_eq!(is_power_of_2(4), true);
        assert_eq!(is_power_of_2(8), true);
        assert_eq!(is_power_of_2(3), false);
        assert_eq!(is_power_of_2(9), false);
    }

    #[test]
    fn test_next_power_of_2() {
        assert_eq!(next_power_of_2(1), 1);
        assert_eq!(next_power_of_2(2), 2);
        assert_eq!(next_power_of_2(3), 4);
        assert_eq!(next_power_of_2(12), 16);
    }
}
