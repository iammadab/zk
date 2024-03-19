pub(crate) trait Hasher {
    type Item;
    type Digest;

    fn hash_item(input: &Self::Item) -> Self::Digest;
    fn hash_slice(input: &[Self::Item]) -> Self::Digest;
    fn hash_items(input: &[Self::Item]) -> Vec<Self::Digest>;
}
