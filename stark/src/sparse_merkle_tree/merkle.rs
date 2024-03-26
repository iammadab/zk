use crate::hasher::Hasher;

/// Sparse Merkle Tree Struct
/// doesn't pad the leaves to a power of 2, only ensure each layer has an even number of elements
struct SparseMerkleTree<H: Hasher>{
    tree: Vec<H::Digest>,
    no_of_leaves: usize
}