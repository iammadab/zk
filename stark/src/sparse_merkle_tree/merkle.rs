use crate::hasher::Hasher;
use crate::sparse_merkle_tree::util::{extra_hash_count, layer_counts, parent, sibling};

/// Represents a sparse merkle proof, need to keep track of the proved node and leaf count
/// as hint to the verifier
struct SparseMerkleProof<T> {
    hashes: Vec<T>,
    leaf_count: usize,
    node_index: usize,
}

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

    /// Generate a merkle proof for the element at a given index
    fn prove(&self, index: usize) -> Result<SparseMerkleProof<H::Digest>, &'static str> {
        let mut proof = vec![];

        if index >= self.no_of_leaves {
            return Err("no element at given proving index");
        }

        // need to offset the provided index as the leaves are at the end of the tree array
        let leaf_index = index + extra_hash_count(self.no_of_leaves);
        let mut proof_node_index = sibling(leaf_index);

        while proof_node_index != 0 {
            proof.push(self.tree[proof_node_index].clone());
            proof_node_index = sibling(parent(proof_node_index, layer_counts(self.no_of_leaves)));
        }

        Ok(SparseMerkleProof {
            hashes: proof,
            node_index: leaf_index,
            leaf_count: self.no_of_leaves,
        })
    }

    /// Verify the merkle proof for a given leaf element
    fn verify(
        input: &H::Item,
        proof: SparseMerkleProof<H::Digest>,
        expected_root_hash: &H::Digest,
    ) -> bool {
        let input_hash = H::hash_item(input);

        // this represents the node index of the current running hash
        let mut known_hash_index = proof.node_index;

        let root_hash = proof.hashes.iter().fold(input_hash, |acc, proof_hash| {
            let next_known_hash = if known_hash_index % 2 == 0 {
                H::hash_digest_slice(&[proof_hash, &acc])
            } else {
                H::hash_digest_slice(&[&acc, proof_hash])
            };
            known_hash_index = parent(known_hash_index, layer_counts(proof.leaf_count));
            next_known_hash
        });

        root_hash == *expected_root_hash
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

    #[test]
    fn test_prove_verify_index() {
        let values = vec![
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
            6_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            4_u8.to_be_bytes().to_vec(),
            vec![],
        ];

        let tree = build_merkle_tree();

        // cannot prove outside of allow leaf indexes
        assert_eq!(tree.prove(6).is_err(), true);

        let proof = tree.prove(0).unwrap();
        assert_eq!(proof.hashes.len(), 3);
        assert_eq!(proof.node_index, 7);
        assert_eq!(proof.leaf_count, 6);
        assert_eq!(proof.hashes[0], tree.tree[8]);
        assert_eq!(proof.hashes[1], tree.tree[4]);
        assert_eq!(proof.hashes[2], tree.tree[2]);
        assert!(SparseMerkleTree::<Sha3Hasher>::verify(
            &values[0],
            proof,
            tree.root_hash()
        ));

        let proof = tree.prove(3).unwrap();
        assert_eq!(proof.hashes.len(), 3);
        assert_eq!(proof.node_index, 10);
        assert_eq!(proof.leaf_count, 6);
        assert_eq!(proof.hashes[0], tree.tree[9]);
        assert_eq!(proof.hashes[1], tree.tree[3]);
        assert_eq!(proof.hashes[2], tree.tree[2]);
        assert!(SparseMerkleTree::<Sha3Hasher>::verify(
            &values[3],
            proof,
            tree.root_hash()
        ));
    }
}
