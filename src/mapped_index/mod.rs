//! Index types and traits for mapping between flat indices and values.

pub mod categorical_index;
pub mod compound_index;
pub mod fn_map_index;
pub mod numeric_range;
pub mod singleton_index;
pub mod sparse_numeric_index;

/// A trait for types that provide a range of values of a certain variable.
pub trait VariableRange {
    /// The value type stored in the index.
    type Value<'a>: Copy
    where
        Self: 'a;

    /// Returns an iterator over all values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone;

    /// Returns the value for the given flat numeric index.
    ///
    /// # Panics
    ///
    /// Implementations must panic if the index is out of bounds.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_>;

    /// Returns the total number of values in the index.
    fn size(&self) -> usize;
}
