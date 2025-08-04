use super::MappedIndex;
use std::marker::PhantomData;

/// A value in a singleton index, representing the single value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SingletonValue<T: std::fmt::Debug> {
    /// The phantom data for type T.
    _phantom: PhantomData<T>,
}

impl<T: std::fmt::Debug> PartialEq for SingletonValue<T> {
    fn eq(&self, _other: &Self) -> bool {
        // There's only one possible value, so all SingletonValues are equal
        true
    }
}

impl<T: std::fmt::Debug> Eq for SingletonValue<T> {}

impl<T: std::fmt::Debug> Copy for SingletonValue<T> {}
impl<T: std::fmt::Debug> Clone for SingletonValue<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index representing a single value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SingletonIndex<T: std::fmt::Debug> {
    /// The phantom data for type T.
    _phantom: PhantomData<T>,
}

impl<T: std::fmt::Debug> Clone for SingletonIndex<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: std::fmt::Debug> PartialEq for SingletonIndex<T> {
    fn eq(&self, _other: &Self) -> bool {
        // All SingletonIndex instances are equal
        true
    }
}

impl<T: std::fmt::Debug> Eq for SingletonIndex<T> {}

impl<T: std::fmt::Debug> SingletonIndex<T> {
    /// Create a new SingletonIndex.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Returns the singleton value.
    pub fn value(&self) -> SingletonValue<T> {
        SingletonValue {
            _phantom: PhantomData,
        }
    }
}

impl<T: std::fmt::Debug + 'static> MappedIndex for SingletonIndex<T> {
    type Value<'a> = SingletonValue<T>;

    /// Returns an iterator over the single value in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        std::iter::once(SingletonValue {
            _phantom: PhantomData,
        })
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
        SingletonValue {
            _phantom: PhantomData,
        }
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
        let index = SingletonIndex::<()>::new();
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
        let index = SingletonIndex::<()>::new();
        index.unflatten_index_value(1); // Should panic
    }
}
