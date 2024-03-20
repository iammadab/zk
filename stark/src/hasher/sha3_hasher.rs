use sha3::{Digest, Sha3_256};
use crate::hasher::Hasher;

struct Sha3Hasher {}

impl Hasher for Sha3Hasher {
    type Item = Vec<u8>;
    type Digest = [u8; 32];

    fn hash_item(input: &Self::Item) -> Self::Digest {
        let mut hasher = Sha3_256::new();
        hasher.update(input);
        let mut hash = [0; 32];
        hash.copy_from_slice(&hasher.finalize());
        hash
    }

    fn hash_digest_slice(input: &[&Self::Digest]) -> Self::Digest {
        let mut hasher = Sha3_256::new();

        for item in input {
            hasher.update(item);
        }

        let mut hash = [0; 32];
        hash.copy_from_slice(&hasher.finalize());
        hash
    }

    fn hash_items(input: &[Self::Item]) -> Vec<Self::Digest> {
        input.iter().map(Self::hash_item).collect()
    }
}
