use crate::hasher;

struct MerkleTree<H: hasher::Hasher> {
    tree: Vec<H::Digest>,
}
