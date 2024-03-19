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
}
