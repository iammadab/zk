use crate::hasher::Hasher;
use crate::sparse_merkle_tree::util::extra_hash_count;

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
        // the first extra hash count elements will be updated
        let mut tree = vec![H::Digest::default(); extra_hash_count(hashed_leaves.len())];
        tree.extend(hashed_leaves);

        // iteratively hash sibling leaves up to the root
        // TODO: need to figure out when to insert the placeholder term
        //  seems it might need the hint
        //  if we have 6 leaves, we'd generate 3 elements for the next layer, but we need to add
        //  the placeholder term, before adding the 3 elements
        //  hence we need a way to determine if the current layer will require some padding
        //  if it does then we should duplicate the placeholder (I believe)
        //  another thing to consider is the fact that the parent logic should avoid that slot
        //  so maybe we don't actually need to change anything???
        //  we could also do a secondary pass that does the duplication for us, multiple ways to handle this
        //  but first need to implement the parent function

        todo!()
    }
}
