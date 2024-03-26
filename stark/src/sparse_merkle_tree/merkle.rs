use crate::hasher::Hasher;

/// Sparse Merkle Tree Struct
/// doesn't pad the leaves to a power of 2, only ensure each layer has an even number of elements
struct SparseMerkleTree<H: Hasher> {
    tree: Vec<H::Digest>,
    no_of_leaves: usize,
}

impl<H: Hasher> SparseMerkleTree<H> {
    /// Return the root hash of the tree
    fn root_hash(&self) -> &H::Digest {
        &self.tree[0]
    }

    /// Builds a merkle tree from a list of leaf values
    fn build(input: &[H::Item]) -> Self {
        // input cannot be empty
        assert!(input.len() > 0);

        // hash the input items and extend to even number
        let mut hashed_leaves = H::hash_items(input);
        if hashed_leaves.len() % 2 != 0 {
            hashed_leaves.push(H::hash_item(&H::Item::default()));
        }

        // build empty slots for parent hashes, store the leaf hashes at the end of the vector
        // TODO:
        //  to do this, I need to know what the total length will be given the number of leaves
        //

        todo!()
    }
}
