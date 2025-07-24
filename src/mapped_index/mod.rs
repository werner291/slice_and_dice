//! Index types and traits for mapping between flat indices and values.

pub mod categorical_index;
pub mod compound_index;
pub mod numeric_range_index;
pub mod sparse_numeric_index;

/// A trait for types that provide a mapping between a flat numeric index and a value.
///
/// This trait enables efficient, index-based access to values, and supports round-trip
/// conversion between values and their flat indices. It is intended for use with index types
/// such as categorical, numeric range, and compound indices.
///
/// All implementors must also implement Eq and PartialEq.
pub trait MappedIndex {
    /// The value type stored in the index.
    type Value<'a>: Copy
    where
        Self: 'a;

    /// Returns an iterator over all values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone;

    /// Returns the flat numeric index for the given value. Panics if the value is not found.
    ///
    /// # Panics
    ///
    /// Implementations must panic if the value is not present in the index.
    fn flatten_index_value<'a>(&'a self, value: Self::Value<'a>) -> usize;

    /// Returns the value for the given flat numeric index.
    ///
    /// # Panics
    ///
    /// Implementations must panic if the index is out of bounds.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_>;

    /// Returns the total number of values in the index.
    fn size(&self) -> usize;

    /// Returns the minimum value in the index, if any (requires Ord).
    fn min<'a>(&'a self) -> Option<Self::Value<'a>>
    where
        Self::Value<'a>: Ord;

    /// Returns the maximum value in the index, if any (requires Ord).
    fn max<'a>(&'a self) -> Option<Self::Value<'a>>
    where
        Self::Value<'a>: Ord;
}
