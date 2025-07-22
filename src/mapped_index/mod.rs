pub mod numeric_range_index;
pub mod categorical_index;

pub trait MappedIndex<'idx, Idx> {
    type Value: Copy;
    /// Returns the value mapped to the given index, or None if the index is not mapped.
    fn get(&'idx self, index: Idx) -> Option<Self::Value>;
    /// Returns an iterator over all values in the index.
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value>;
} 