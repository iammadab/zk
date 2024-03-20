use crate::hasher::Hasher;
use crate::util::{extend_to_power_of_two, extra_hash_count, is_power_of_2, parent, sibling};

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

    /// Builds a Merkle tree from a list of leave values
    fn build(input: &[H::Item]) -> Self {
        // input cannot be empty
        assert!(input.len() > 0);

        // hash the input items and extend to a power of 2 if needed
        let mut hashed_leaves = H::hash_items(input);
        extend_to_power_of_two(&mut hashed_leaves, H::Digest::default());

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
}

// #[cfg(test)]
// mod tests {
// }
