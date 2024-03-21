pub mod sha3_hasher;

/// Hasher Trait
pub(crate) trait Hasher {
    type Item;
    type Digest: Clone + Default + PartialEq;

    fn hash_item(input: &Self::Item) -> Self::Digest;
    fn hash_digest_slice(input: &[&Self::Digest]) -> Self::Digest;
    fn hash_items(input: &[Self::Item]) -> Vec<Self::Digest>;
}
