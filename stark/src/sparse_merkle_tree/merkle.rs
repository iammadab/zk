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
    use crate::hasher::Hasher;
    use crate::sparse_merkle_tree::merkle::SparseMerkleTree;
    use sha3::{Digest, Sha3_256};

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
        let mut values = vec![
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
            6_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
        ];
        // extend to an even number of elements
        values.push(vec![]);

        let tree = build_merkle_tree();
        assert_eq!(tree.tree.len(), 13);

        // hash the input leaves
        let mut hasher = Sha3_256::new();
        let value_hashes = values
            .into_iter()
            .map(|val| {
                hasher.update(&val);
                let mut hash = [0; 32];
                hash.copy_from_slice(&hasher.finalize_reset());
                hash
            })
            .collect::<Vec<_>>();

        // assert the leaf nodes (4th layer)
        assert_eq!(tree.tree[12], value_hashes[5]);
        assert_eq!(tree.tree[11], value_hashes[4]);
        assert_eq!(tree.tree[10], value_hashes[3]);
        assert_eq!(tree.tree[9], value_hashes[2]);
        assert_eq!(tree.tree[8], value_hashes[1]);
        assert_eq!(tree.tree[7], value_hashes[0]);

        // assert 3rd layer
        assert_eq!(tree.tree[6], [0; 32]);
        let hash_4_empty = Sha3Hasher::hash_digest_slice(&[&tree.tree[11], &tree.tree[12]]);
        assert_eq!(tree.tree[5], hash_4_empty);
        let hash_6_2 = Sha3Hasher::hash_digest_slice(&[&tree.tree[9], &tree.tree[10]]);
        assert_eq!(tree.tree[4], hash_6_2);
        let hash_2_4 = Sha3Hasher::hash_digest_slice(&[&tree.tree[7], &tree.tree[8]]);
        assert_eq!(tree.tree[3], hash_2_4);

        // assert 2nd layer
        let hash_4_empty_default = Sha3Hasher::hash_digest_slice(&[&hash_4_empty, &[0; 32]]);
        assert_eq!(tree.tree[2], hash_4_empty_default);
        let hash_2_4_6_2 = Sha3Hasher::hash_digest_slice(&[&hash_2_4, &hash_6_2]);
        assert_eq!(tree.tree[1], hash_2_4_6_2);

        // assert root layer (1st layer)
        let hash_root = Sha3Hasher::hash_digest_slice(&[&hash_2_4_6_2, &hash_4_empty_default]);
        assert_eq!(tree.tree[0], hash_root);
        assert_eq!(tree.root_hash(), &hash_root);
    }
}
