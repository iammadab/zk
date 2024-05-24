// TODO: add documentation and add assumptions made
//  pairing var is 0 indexed
pub struct PairingIndex {
    evaluation_len: usize,
    shift_value: usize,
    current_index: usize,
    counter: usize,
}

impl PairingIndex {
    pub fn new(n_vars: usize, position: usize) -> Result<Self, &'static str> {
        if position >= n_vars {
            return Err("pairing variable must be less than number of variables (zero indexed)");
        }

        let evaluation_len = 1 << n_vars;

        Ok(Self {
            evaluation_len,
            // TODO: use vars for this or explain each term
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
        // TODO: add comments
        if self.current_index + self.shift_value >= self.evaluation_len {
            return None;
        }

        let index = self.current_index;
        self.current_index += 1;

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
