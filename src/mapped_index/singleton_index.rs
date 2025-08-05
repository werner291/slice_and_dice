use super::MappedIndex;

/// An index representing a single value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SingletonIndex<T> {
    /// The value of type T.
    pub value: T,
}

impl<T: Clone> Clone for SingletonIndex<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for SingletonIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for SingletonIndex<T> {}

impl<T: Clone> SingletonIndex<T> {
    /// Create a new SingletonIndex with the given value.
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Returns the singleton value.
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: Copy + 'static> MappedIndex for SingletonIndex<T> {
    type Value<'a>
        = &'a T
    where
        T: 'a;

    /// Returns an iterator over the single value in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        std::iter::once(&self.value)
    }

    /// Returns the flat index for the singleton value (always 0).
    fn flatten_index_value(&self, _value: Self::Value<'_>) -> usize {
        0
    }

    /// Returns the singleton value for a given flat index.
    /// Panics if the index is not 0.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        if index != 0 {
            panic!("Index out of bounds: {} (expected 0)", index);
        }
        &self.value
    }

    /// Returns the number of values in the singleton index (always 1).
    fn size(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::MappedIndex;

    #[test]
    fn test_singleton_index() {
        let index = SingletonIndex::<()>::new(());
        assert_eq!(index.size(), 1);

        let value = index.value();
        let flat = index.flatten_index_value(value);
        assert_eq!(flat, 0);

        let round_trip = index.unflatten_index_value(flat);
        assert_eq!(value, round_trip);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds: 1 (expected 0)")]
    fn test_out_of_bounds() {
        let index = SingletonIndex::<()>::new(());
        index.unflatten_index_value(1); // Should panic
    }
}
