/// Inserts a bit at an arbitrary position in a bit sequence
/// e.g. insert 1 at position 2 in this sequence 101 = 1101
/// NOTE: position is counted from the back
/// sequence: 1 1 0 1
/// index   : 3 2 1 0
fn insert_bit(val: usize, index: u8, bit: usize) -> usize {
    let high = val >> index;
    let low = val & mask(index);
    high << 1 | bit << index | low
}

/// Generates a bit sequence with n 1's
/// e.g. mask(1) -> 1, mask(3) -> 111
pub const fn mask(n: u8) -> usize {
    (1 << n) - 1
}

#[cfg(test)]
mod tests {
    use crate::multilinear::pairing_index_2::insert_bit;

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
    }
}
