use crate::hasher::Hasher;
use crate::sparse_merkle_tree::util::{extra_hash_count, layer_counts, parent, sibling};

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
        let no_of_leaves = hashed_leaves.len();

        // build empty slots for parent hashes, store the leaf hashes at the end of the vector
        // the first extra hash count elements will be updated
        let mut tree = vec![H::Digest::default(); extra_hash_count(no_of_leaves)];
        tree.extend(hashed_leaves);

        // iteratively hash sibling leaves up to the root
        for right_index in (1..tree.len()).rev().step_by(2) {
            let left_index = sibling(right_index);
            let parent_index = parent(right_index, layer_counts(no_of_leaves));

            // hash left and right leaves, store in parent
            tree[parent_index] = H::hash_digest_slice(&[&tree[left_index], &tree[right_index]]);
        }

        SparseMerkleTree { tree, no_of_leaves }
    }
}

#[cfg(test)]
mod tests {
    use crate::hasher::sha3_hasher::Sha3Hasher;
    use crate::sparse_merkle_tree::merkle::SparseMerkleTree;

    fn build_merkle_tree() -> SparseMerkleTree<Sha3Hasher> {
        let values = vec![
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
            6_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
        ];
        let tree = SparseMerkleTree::<Sha3Hasher>::build(&values);
        tree
    }

    #[test]
    fn test_build_sparse_merkle_tree() {
        // what is the best way to test this?
        // need to ensure it's building what's expected at all levels
        // hence verification of the entire tree might be needed
        // first will need to verify the length of the tree

        let tree = build_merkle_tree();
        assert_eq!(tree.tree.len(), 12);
    }
}
