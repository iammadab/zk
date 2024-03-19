use crate::hasher::Hasher;

struct MerkleTree<H: Hasher> {
    tree: Vec<H::Digest>,
}

impl<H: Hasher> MerkleTree<H> {
    // fn build(input: &[H::Item]) -> Self {

    // }
}

// #[cfg(test)]
// mod tests {
// }
