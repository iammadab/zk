/// State for pairing index iterator
/// Figures out the appropriate pairing of boolean hypercube entries
/// based on some evaluation index
/// Goal: Return the index such that the pair (index, index + shift_value)
/// represents two boolean hypercube entries that are the same on every bit
/// expect for the bit at a certain position.
///
/// Example:
/// BooleanHypercube len = 3
/// 000 - 0
/// 001 - 1
/// 010 - 2
/// 011 - 3
/// 100 - 4
/// 101 - 5
/// 110 - 6
/// 111 - 7
///
/// if position = 0 then pairing is as follows
/// 0 - 4
/// 1 - 5
/// 2 - 6
/// 3 - 7
/// this iterator should return [0, 1, 2, 3]
///
/// if position = 1 then pairing is as follows
/// 0 - 2
/// 1 - 3
/// 4 - 6
/// 5 - 7
/// this iterator should return [0, 1, 4, ,5]
pub struct PairingIndex {
    evaluation_len: usize,
    shift_value: usize,
    current_index: usize,
    counter: usize,
}

impl PairingIndex {
    /// Instantiates a new pairing index iterator based on the number of variables and a given variable position
    /// variable position is assumed to be 0 index
    /// so a 3 variable system will be indexed at the following positions [0, 1, 2]
    /// returns an error if indexing goes outside this bounds
    pub fn new(n_vars: usize, position: usize) -> Result<Self, &'static str> {
        if position >= n_vars {
            return Err("pairing variable must be less than number of variables (zero indexed)");
        }

        let evaluation_len = 1 << n_vars;

        Ok(Self {
            evaluation_len,
            // shift_value = 2^n_vars / 2^(var_index + 1)
            shift_value: evaluation_len / (1 << (position + 1)),
            counter: 0,
            current_index: 0,
        })
    }

    pub fn shift_value(&self) -> usize {
        self.shift_value
    }
}

impl Iterator for PairingIndex {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // termination condition
        if self.current_index + self.shift_value >= self.evaluation_len {
            return None;
        }

        let index = self.current_index;
        self.current_index += 1;

        // if we have return shift_value number of elements
        // we make a jump based on the shift value
        if (self.counter + 1) % self.shift_value == 0 {
            self.current_index += self.shift_value;
        }

        self.counter += 1;

        Some(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::pairing_index::PairingIndex;

    #[test]
    fn test_pairing_index_creation() {
        let pairing_index = PairingIndex::new(3, 0).unwrap();
        assert_eq!(pairing_index.shift_value, 4);

        let pairing_index = PairingIndex::new(3, 1).unwrap();
        assert_eq!(pairing_index.shift_value, 2);
    }

    #[test]
    fn test_pairing_index_computation() {
        let pairing_index = PairingIndex::new(3, 0).unwrap();
        assert_eq!(pairing_index.collect::<Vec<usize>>(), vec![0, 1, 2, 3]);

        let pairing_index = PairingIndex::new(3, 1).unwrap();
        assert_eq!(pairing_index.collect::<Vec<usize>>(), vec![0, 1, 4, 5]);

        let pairing_index = PairingIndex::new(3, 2).unwrap();
        assert_eq!(pairing_index.collect::<Vec<usize>>(), vec![0, 2, 4, 6]);

        let pairing_index = PairingIndex::new(4, 2).unwrap();
        assert_eq!(
            pairing_index.collect::<Vec<usize>>(),
            vec![0, 1, 4, 5, 8, 9, 12, 13]
        );

        let pairing_index = PairingIndex::new(4, 4);
        assert!(pairing_index.is_err());
    }
}
