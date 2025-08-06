use super::VariableRange;

/// A value in a categorical index, referencing a value in the index and its position.
#[derive(Debug)]
pub struct CategoricalValue<'a, T> {
    /// Reference to the value in the index.
    pub value: &'a T,
    /// The position of the value in the index.
    index: usize,
}

impl<'idx, T> Copy for CategoricalValue<'idx, T> {}
impl<'idx, T> Clone for CategoricalValue<'idx, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: PartialEq> PartialEq for CategoricalValue<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.index == other.index
    }
}

impl<'a, T: Eq> Eq for CategoricalValue<'a, T> {}

/// An index for categorical values, mapping indices to values of type `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoricalRange<T> {
    /// The values stored in the index.
    pub values: Vec<T>,
}

/// An index for categorical values, mapping indices to values of type `T` using a slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceCategoricalIndex<'a, T> {
    /// The values stored in the index.
    pub values: &'a [T],
}

impl<'a, T: 'a> SliceCategoricalIndex<'a, T> {
    /// Returns the flat index for a categorical value (its position).
    pub(crate) fn flatten_index_value<'b>(&'b self, value: CategoricalValue<'b, T>) -> usize {
        value.index
    }
}

impl<'a, T: 'a> VariableRange for SliceCategoricalIndex<'a, T> {
    type Value<'b>
        = CategoricalValue<'b, T>
    where
        Self: 'b;

    /// Returns an iterator over all categorical values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.values
            .iter()
            .enumerate()
            .map(move |(index, v)| CategoricalValue { value: v, index })
    }
    /// Returns the categorical value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        CategoricalValue {
            value: &self.values[index],
            index,
        }
    }
    /// Returns the number of values in the categorical index.
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl<T> VariableRange for CategoricalRange<T> {
    type Value<'a>
        = CategoricalValue<'a, T>
    where
        T: 'a;

    /// Returns an iterator over all categorical values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.values
            .iter()
            .enumerate()
            .map(move |(index, v)| CategoricalValue { value: v, index })
    }
    /// Returns the categorical value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        CategoricalValue {
            value: &self.values[index],
            index,
        }
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
    /// Returns a reference to the value at the given categorical value.
    pub fn at<'idx>(&'idx self, cat_value: CategoricalValue<'idx, T>) -> &'idx T {
        &self.values[cat_value.index]
    }
}

impl<'a, T> SliceCategoricalIndex<'a, T> {
    /// Create a new SliceCategoricalIndex from a slice of values.
    pub const fn new(values: &'a [T]) -> Self {
        Self { values }
    }
    /// Returns a reference to the value at the given categorical value.
    pub fn at<'idx>(&'idx self, cat_value: CategoricalValue<'idx, T>) -> &'idx T {
        &self.values[cat_value.index]
    }
}

impl<'idx, T> CategoricalValue<'idx, T> {
    /// Create a new CategoricalValue from a reference and index.
    pub const fn new(value: &'idx T, index: usize) -> Self {
        Self { value, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::VariableRange;

    #[test]
    fn test_unflatten_index_value() {
        let index = CategoricalRange {
            values: vec![1, 2, 3],
        };
        let cat_val = index.unflatten_index_value(2);
        assert_eq!(*cat_val.value, 3);
        assert_eq!(cat_val.index, 2);
    }

    #[test]
    fn test_slice_unflatten_index_value() {
        let values = [1, 2, 3];
        let index = SliceCategoricalIndex { values: &values };
        let cat_val = index.unflatten_index_value(2);
        assert_eq!(*cat_val.value, 3);
        assert_eq!(cat_val.index, 2);
    }

    #[test]
    fn test_slice_constructor() {
        let values = [1, 2, 3];
        let index = SliceCategoricalIndex::new(&values);
        assert_eq!(index.size(), 3);
        let cat_val = index.unflatten_index_value(1);
        assert_eq!(*cat_val.value, 2);
    }
}
