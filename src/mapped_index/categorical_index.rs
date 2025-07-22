use super::MappedIndex;
use std::marker::PhantomData;

/// A value in a categorical index, referencing a value in the index and its position.
#[derive(Debug, PartialEq, Eq)]
pub struct CategoricalValue<'a, T, Tag> {
    /// Reference to the value in the index.
    pub value: &'a T,
    /// The position of the value in the index.
    index: usize,
    _phantom: PhantomData<Tag>,
}

impl<'idx, T: Copy, Tag> Copy for CategoricalValue<'idx, T, Tag> {}
impl<'idx, T: Copy, Tag> Clone for CategoricalValue<'idx, T, Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index for categorical values, mapping indices to values of type `T`.
pub struct CategoricalIndex<T, Tag> {
    /// The values stored in the index.
    pub values: Vec<T>,
    pub _phantom: PhantomData<Tag>,
}

impl<'idx, T: Copy + 'idx + Eq, Tag: 'static> Eq for CategoricalIndex<T, Tag> {}

impl<'idx, T: Copy + 'idx + PartialEq, Tag: 'static> PartialEq<Self> for CategoricalIndex<T, Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<T: Copy, Tag: 'static> MappedIndex for CategoricalIndex<T, Tag> {
    type Value<'a>
        = CategoricalValue<'a, T, Tag>
    where
        T: 'a;

    /// Returns an iterator over all categorical values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.values
            .iter()
            .enumerate()
            .map(move |(index, v)| CategoricalValue {
                value: v,
                index,
                _phantom: PhantomData,
            })
    }
    /// Returns the flat index for a categorical value (its position).
    fn flatten_index_value(&self, value: Self::Value<'_>) -> usize {
        value.index
    }
    /// Returns the categorical value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        CategoricalValue {
            value: &self.values[index],
            index,
            _phantom: PhantomData,
        }
    }
    /// Returns the number of values in the categorical index.
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl<T: Copy, Tag> CategoricalIndex<T, Tag> {
    /// Create a new CategoricalIndex from a vector of values.
    pub fn new(values: Vec<T>) -> Self {
        Self {
            values,
            _phantom: PhantomData,
        }
    }
    /// Returns a reference to the value at the given categorical value.
    pub fn at<'idx>(&'idx self, cat_value: CategoricalValue<'idx, T, Tag>) -> &'idx T {
        &self.values[cat_value.index]
    }
}

impl<'idx, T, Tag> CategoricalValue<'idx, T, Tag> {
    /// Create a new CategoricalValue from a reference and index.
    pub fn new(value: &'idx T, index: usize) -> Self {
        Self {
            value,
            index,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::MappedIndex;

    struct Tag;

    #[test]
    fn test_flat_index_round_trip() {
        let index = CategoricalIndex {
            values: vec![1, 2, 3],
            _phantom: PhantomData::<Tag>,
        };
        let cat_val = index.unflatten_index_value(2);
        let flat = index.flatten_index_value(cat_val);
        let round = index.unflatten_index_value(flat);
        assert_eq!(cat_val.index, round.index);
        assert_eq!(*cat_val.value, *round.value);
    }
}
