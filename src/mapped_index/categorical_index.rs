use super::VariableRange;

/// An index for categorical values, mapping indices to values of type `T`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoricalRange<T> {
    /// The values stored in the index.
    pub values: Vec<T>,
}

/// An index for categorical values, mapping indices to values of type `T` using a slice.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SliceCategoricalIndex<'a, T> {
    /// The values stored in the index.
    pub values: &'a [T],
}

impl<'a, T: 'a + Sync + Clone> VariableRange for SliceCategoricalIndex<'a, T> {
    type Value<'b>
        = &'a T
    where
        Self: 'b;

    /// Returns an iterator over all categorical values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.values.into_iter()
    }

    /// Returns the categorical value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        &self.values[index]
    }
    /// Returns the number of values in the categorical index.
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl<T: Sync + Clone> VariableRange for CategoricalRange<T> {
    type Value<'a>
        = &'a T
    where
        T: 'a;

    /// Returns an iterator over all categorical values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.values.iter()
    }
    /// Returns the categorical value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        &self.values[index]
    }
    /// Returns the number of values in the categorical index.
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl<T> CategoricalRange<T> {
    /// Create a new CategoricalIndex from a vector of values.
    pub const fn new(values: Vec<T>) -> Self {
        Self { values }
    }
}

impl<'a, T> SliceCategoricalIndex<'a, T> {
    /// Create a new SliceCategoricalIndex from a slice of values.
    pub const fn new(values: &'a [T]) -> Self {
        Self { values }
    }
}
