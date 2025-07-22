//! Index types and traits for mapping between flat indices and values.

pub mod numeric_range_index;
pub mod categorical_index;
pub mod compound_index;
pub mod data_frame;

/// A trait for types that provide a mapping between a flat numeric index and a value.
///
/// This trait enables efficient, index-based access to values, and supports round-trip
/// conversion between values and their flat indices. It is intended for use with index types
/// such as categorical, numeric range, and compound indices.
pub trait MappedIndex<'idx, Idx> {
    /// The value type stored in the index.
    type Value: Copy;
    /// Returns an iterator over all values in the index.
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value>;
    /// Returns the flat numeric index for the given value. Panics if the value is not found.
    ///
    /// # Panics
    ///
    /// Implementations must panic if the value is not present in the index.
    fn to_flat_index(&self, value: Self::Value) -> usize;
    /// Returns the value for the given flat numeric index.
    ///
    /// # Panics
    ///
    /// Implementations must panic if the index is out of bounds.
    fn from_flat_index(&'idx self, index: usize) -> Self::Value;
    /// Returns the total number of values in the index.
    fn size(&self) -> usize;
} 