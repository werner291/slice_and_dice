use super::VariableRange;

/// A range of a single value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SingletonRange<T> {
    /// The value of type T.
    pub value: T,
}

impl<T> SingletonRange<T> {
    /// Create a new SingletonIndex with the given value.
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Get a reference to the value.
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> VariableRange for SingletonRange<T> {
    type Value<'a>
        = &'a T
    where
        T: 'a;

    /// Returns an iterator over the single value in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        std::iter::once(&self.value)
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
    use crate::mapped_index::VariableRange;

    #[test]
    fn test_singleton_index() {
        let index = SingletonRange::<()>::new(());
        assert_eq!(index.size(), 1);

        let value = index.value();
        // For a singleton index, the flat index is always 0
        let flat = 0;

        let round_trip = index.unflatten_index_value(flat);
        assert_eq!(value, round_trip);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds: 1 (expected 0)")]
    fn test_out_of_bounds() {
        let index = SingletonRange::<()>::new(());
        index.unflatten_index_value(1); // Should panic
    }
}
