use ark_ff::PrimeField;
use polynomial::multilinear::coefficient_form::binary_string;
use std::marker::PhantomData;

/// Structure for point iteration over boolean hypercube
/// e.g. BooleanHyperCube 2 variables
/// Some(00), Some(01), Some(10), Some(11), None
pub struct BooleanHyperCube<F: PrimeField> {
    bit_size: usize,
    total_points: usize,
    current_point: usize,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> BooleanHyperCube<F> {
    pub fn new(bit_size: usize) -> Self {
        Self {
            bit_size,
            total_points: 2_usize.pow(bit_size as u32),
            current_point: 0,
            _marker: PhantomData,
        }
    }
}

impl<F: PrimeField> Iterator for BooleanHyperCube<F> {
    type Item = Vec<F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_point == self.total_points || self.bit_size == 0 {
            return None;
        }

        // convert the current index to binary value of the given length
        let index_as_binary = binary_string(self.current_point, self.bit_size);
        let point = index_as_binary
            .chars()
            .map(|a| if a == '1' { F::one() } else { F::zero() })
            .collect::<Vec<F>>();

        self.current_point += 1;

        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_hypercube::BooleanHyperCube;
    use ark_ff::{Fp64, MontBackend, MontConfig, One, Zero};
    use std::iter::Iterator;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    struct FqConfig;
    type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_boolean_hypercube_iteration() {
        let mut two_bit_iterator = BooleanHyperCube::<Fq>::new(2);
        assert_eq!(two_bit_iterator.total_points, 4);
        assert_eq!(two_bit_iterator.next(), Some(vec![Fq::zero(); 2]));
        assert_eq!(two_bit_iterator.next(), Some(vec![Fq::zero(), Fq::one()]));
        assert_eq!(two_bit_iterator.next(), Some(vec![Fq::one(), Fq::zero()]));
        assert_eq!(two_bit_iterator.next(), Some(vec![Fq::one(); 2]));
        assert_eq!(two_bit_iterator.next(), None);

        let mut three_bit_iterator = BooleanHyperCube::<Fq>::new(3);
        assert_eq!(three_bit_iterator.total_points, 8);
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::zero(), Fq::zero(), Fq::zero()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::zero(), Fq::zero(), Fq::one()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::zero(), Fq::one(), Fq::zero()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::zero(), Fq::one(), Fq::one()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::one(), Fq::zero(), Fq::zero()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::one(), Fq::zero(), Fq::one()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::one(), Fq::one(), Fq::zero()])
        );
        assert_eq!(
            three_bit_iterator.next(),
            Some(vec![Fq::one(), Fq::one(), Fq::one()])
        );
        assert_eq!(three_bit_iterator.next(), None);
    }
}
