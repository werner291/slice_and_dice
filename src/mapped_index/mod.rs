pub mod numeric_range_index;
pub mod categorical_index;
pub mod compound_index;

pub trait MappedIndex<'idx, Idx> {
    type Value: Copy;
    /// Returns an iterator over all values in the index.
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value>;
    /// Returns the flat numeric index for the given value. Panics if the value is not found.
    fn to_flat_index(&self, value: Self::Value) -> usize;
    /// Returns the value for the given flat numeric index.
    fn from_flat_index(&'idx self, index: usize) -> Self::Value;
    /// Returns the total number of values in the index.
    fn size(&self) -> usize;
} 