use crate::hasher::Hasher;
use crate::util::{
    extend_to_power_of_two, extra_hash_count, is_power_of_2, number_of_leaves, parent, sibling,
};

// TODO: add documentation
struct MerkleProof<T> {
    hashes: Vec<T>,
    node_index: usize,
}

// TODO: add documentation
struct MerkleTree<H: Hasher> {
    tree: Vec<H::Digest>,
}

impl<H: Hasher> MerkleTree<H> {
    /// Instantiate a new merkle tree
    fn new(tree: Vec<H::Digest>) -> Self {
        // tree len + 1 should be a power of 2, asserts that a valid
        // binary tree can be built from it.
        assert!(is_power_of_2(tree.len() + 1));
        Self { tree }
    }

    /// Return the root hash of the tree
    fn root_hash(&self) -> &H::Digest {
        &self.tree[0]
    }

    /// Builds a Merkle tree from a list of leave values
    fn build(input: &[H::Item]) -> Self {
        // input cannot be empty
        assert!(input.len() > 0);

        // hash the input items and extend to a power of 2 if needed
        let mut hashed_leaves = H::hash_items(input);
        extend_to_power_of_two(&mut hashed_leaves, H::hash_item(&H::Item::default()));

        // build empty slots for parent hashes, store the leaf hashes at the end of the vector
        let mut tree = vec![H::Digest::default(); extra_hash_count(hashed_leaves.len())];
        tree.extend(hashed_leaves);

        // iteratively hash sibling leaves up to the root
        for right_index in (1..tree.len()).rev().step_by(2) {
            let left_index = sibling(right_index);
            let parent_index = parent(right_index);

            // hash left and right leaves, store in parent
            tree[parent_index] = H::hash_digest_slice(&[&tree[left_index], &tree[right_index]]);
        }

        MerkleTree::new(tree)
    }

    /// Generate a merkle proof for the element at a given index
    fn prove(&self, index: usize) -> Result<MerkleProof<H::Digest>, &'static str> {
        let mut proof = vec![];

        let number_of_leaves = number_of_leaves(self.tree.len());
        if index >= number_of_leaves {
            return Err("no element at given proving index");
        }

        // need to offset the provided index as the leaves are at the end of the tree array
        let leaf_index = index + number_of_leaves - 1;
        let mut proof_node_index = sibling(leaf_index);

        while proof_node_index != 0 {
            proof.push(self.tree[proof_node_index].clone());
            proof_node_index = sibling(parent(proof_node_index));
        }

        Ok(MerkleProof {
            hashes: proof,
            node_index: leaf_index,
        })
    }

    /// Verify the merkle proof for a given leaf element
    fn verify(
        input: &H::Item,
        proof: MerkleProof<H::Digest>,
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
            known_hash_index = parent(known_hash_index);
            next_known_hash
        });

        root_hash == *expected_root_hash
    }
}

#[cfg(test)]
mod tests {
    use crate::hasher::sha3_hasher::Sha3Hasher;
    use crate::merkle::MerkleTree;
    use crate::util::extra_hash_count;
    use sha3::{Digest, Sha3_256};

    fn build_merkle_tree() -> MerkleTree<Sha3Hasher> {
        let values = vec![
            1_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            3_u8.to_be_bytes().to_vec(),
        ];
        let tree = MerkleTree::<Sha3Hasher>::build(&values);
        tree
    }

    #[test]
    fn test_build_merkle_tree() {
        let mut values = vec![
            1_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            3_u8.to_be_bytes().to_vec(),
        ];

        let tree = build_merkle_tree();
        assert_eq!(tree.tree.len(), 7);

        // extend values by the empty vector (for default0
        values.push(vec![]);

        // hash the input leaves
        let mut hasher = Sha3_256::new();
        let values_hash = values
            .into_iter()
            .map(|val| {
                hasher.update(&val);
                let mut hash = [0; 32];
                hash.copy_from_slice(&hasher.finalize_reset());
                hash
            })
            .collect::<Vec<_>>();

        hasher.update(&values_hash[2]);
        hasher.update(&values_hash[3]);
        let mut expected_hash = [0; 32];
        expected_hash.copy_from_slice(&hasher.finalize_reset());
        assert_eq!(expected_hash, tree.tree[2]);

        hasher.update(&values_hash[0]);
        hasher.update(&values_hash[1]);
        let mut expected_hash = [0; 32];
        expected_hash.copy_from_slice(&hasher.finalize_reset());
        assert_eq!(expected_hash, tree.tree[1]);

        hasher.update(&tree.tree[1]);
        hasher.update(&tree.tree[2]);
        let mut expected_hash = [0; 32];
        expected_hash.copy_from_slice(&hasher.finalize_reset());
        assert_eq!(expected_hash, tree.tree[0]);
        assert_eq!(&expected_hash, tree.root_hash());
    }

    #[test]
    fn test_prove_verify_index() {
        let values = vec![
            1_u8.to_be_bytes().to_vec(),
            2_u8.to_be_bytes().to_vec(),
            3_u8.to_be_bytes().to_vec(),
            vec![],
        ];
        let tree = build_merkle_tree();

        let proof = tree.prove(0).unwrap();
        assert_eq!(proof.hashes.len(), 2);
        assert_eq!(proof.node_index, 3);
        assert_eq!(proof.hashes[0], tree.tree[4]);
        assert_eq!(proof.hashes[1], tree.tree[2]);
        assert!(MerkleTree::<Sha3Hasher>::verify(
            &values[0],
            proof,
            tree.root_hash()
        ));

        let proof = tree.prove(3).unwrap();
        assert_eq!(proof.hashes.len(), 2);
        assert_eq!(proof.node_index, 6);
        assert_eq!(proof.hashes[0], tree.tree[5]);
        assert_eq!(proof.hashes[1], tree.tree[1]);
        assert!(MerkleTree::<Sha3Hasher>::verify(
            &values[3],
            proof,
            tree.root_hash()
        ));
    }
}
