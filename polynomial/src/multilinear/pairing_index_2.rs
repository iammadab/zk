/// Returns the nth pairing in some boolean hypercube direction
fn index_pair(n_vars: u8, index: u8) -> impl Iterator<Item = (usize, usize)> {
    let base_no_of_vars = n_vars - 1;
    let no_of_pairs = 1 << base_no_of_vars;
    (0..no_of_pairs).map(move |val| {
        (
            insert_bit(val, base_no_of_vars - index, 0),
            insert_bit(val, base_no_of_vars - index, 1),
        )
    })
}

/// Inserts a bit at an arbitrary position in a bit sequence
/// e.g. insert 1 at position 2 in this sequence 101 = 1101
/// NOTE: position is counted from the back
/// sequence: 1 1 0 1
/// index   : 3 2 1 0
fn insert_bit(val: usize, index: u8, bit: usize) -> usize {
    let high = val >> index;
    let low = val & mask(index);
    high << (index + 1) | bit << index | low
}

/// Generates a bit sequence with n 1's
/// e.g. mask(1) -> 1, mask(3) -> 111
pub const fn mask(n: u8) -> usize {
    (1 << n) - 1
}

#[cfg(test)]
mod tests {
    use crate::multilinear::pairing_index_2::{index_pair, insert_bit};
    use std::ptr::write;

    #[test]
    fn test_bit_insertion() {
        let val: usize = 0b10101;
        // insert 0 at the last position
        assert_eq!(insert_bit(val, 0, 0), 0b101010);
        // insert 1 at the last position
        assert_eq!(insert_bit(val, 0, 1), 0b101011);
        // insert 0 at the first position
        assert_eq!(insert_bit(val, 5, 0), 0b010101);
        // insert 1 at the first position
        assert_eq!(insert_bit(val, 5, 1), 0b110101);

        assert_eq!(insert_bit(0b10, 1, 0), 0b100);
        assert_eq!(insert_bit(0b10, 1, 1), 0b110);
    }

    #[test]
    fn test_index_pairing() {
        // assuming f(a, b, c)
        // 000 - 0
        // 001 - 1
        // 010 - 2
        // 011 - 3
        // 100 - 4
        // 101 - 5
        // 110 - 6
        // 111 - 7

        // a pairing
        let a_pairs = index_pair(3, 0);
        assert_eq!(
            a_pairs.collect::<Vec<_>>(),
            vec![(0, 4), (1, 5), (2, 6), (3, 7)]
        );

        // b pairing
        let b_pairs = index_pair(3, 1);
        assert_eq!(
            b_pairs.collect::<Vec<_>>(),
            vec![(0, 2), (1, 3), (4, 6), (5, 7)]
        );

        // c pairing
        let c_pairs = index_pair(3, 2);
        assert_eq!(
            c_pairs.collect::<Vec<_>>(),
            vec![(0, 1), (2, 3), (4, 5), (6, 7)]
        );

        // assuming f(a, b)
        // 00 - 0
        // 01 - 1
        // 10 - 2
        // 11 - 3

        // a pairing
        assert_eq!(index_pair(2, 0).collect::<Vec<_>>(), vec![(0, 2), (1, 3)]);

        // b pairing
        assert_eq!(index_pair(2, 1).collect::<Vec<_>>(), vec![(0, 1), (2, 3)]);

        // assuming f(a)
        // 0 - 0
        // 1 - 1
        assert_eq!(index_pair(1, 0).collect::<Vec<_>>(), vec![(0, 1)]);
    }
}
