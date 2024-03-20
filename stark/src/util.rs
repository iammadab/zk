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

/// Return parent index of a node
pub fn parent(index: usize) -> usize {
    (index - 1) / 2
}

/// Return the number of extra hash data needed to build a merkle tree
pub fn extra_hash_count(leaf_count: usize) -> usize {
    leaf_count - 1
}

/// Determines if a given value is a power of 2
pub fn is_power_of_2(value: usize) -> bool {
    if value == 0 {
        return false;
    }

    value & (value - 1) == 0
}

/// Returns the next power of 2 from a number
pub fn next_power_of_2(mut value: usize) -> usize {
    // TODO: there has to be a better way to do this
    //  with bitwise operations most likely
    while !is_power_of_2(value) {
        value += 1;
    }
    value
}

/// Takes a slice of n elements, returns a slice of m elements
/// where m is a power of 2.
pub fn extend_to_power_of_two<T: Clone>(input: &mut Vec<T>, default_value: T) {
    let padding_count = next_power_of_2(input.len()) - input.len();
    let padding = vec![default_value.clone(); padding_count];
    input.extend(padding);
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

    #[test]
    fn test_extend_to_power_of_two() {
        // 5 elements, next values of 2 is 8
        let mut set1 = vec![5, 6, 7, 8, 9];
        extend_to_power_of_two(&mut set1, 0);
        assert_eq!(set1.len(), 8);
        assert_eq!(set1, vec![5, 6, 7, 8, 9, 0, 0, 0]);
    }
}
