use super::VariableRange;
use std::marker::PhantomData;

/// A value in a function-mapped index, representing a mapped value from the underlying index.
///
/// This struct holds the flat index in the underlying index and the mapped value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FnMapValue<O> {
    /// The flat index in the underlying index.
    pub flat_index: usize,
    /// The mapped value.
    pub mapped: O,
}

impl<O> FnMapValue<O>
where
    O: Copy + std::fmt::Debug,
{
    /// Create a new FnMapValue with the given flat index and mapped value.
    pub const fn new(flat_index: usize, mapped: O) -> Self {
        Self { flat_index, mapped }
    }
}

/// An index that wraps another index and maps its values using a function.
///
/// This allows an index to be "interpreted" as some higher-level value based on its lower-level components.
/// The mapping function should be fairly straightforward and reasonably cheap, as it will be called
/// frequently during index operations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct FnMapIndex<I, F, O> {
    /// The underlying index.
    pub index: I,
    /// The mapping function.
    pub map_fn: F,
    _phantom: PhantomData<O>,
}

impl<I, F, O> Clone for FnMapIndex<I, F, O>
where
    I: VariableRange + Clone,
    F: Fn(I::Value<'_>) -> O + Clone,
    O: Copy + std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            map_fn: self.map_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I, F, O> PartialEq for FnMapIndex<I, F, O>
where
    I: VariableRange + PartialEq,
    F: Fn(I::Value<'_>) -> O,
    O: Copy + std::fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool {
        // We can only compare the underlying indices, not the functions
        self.index == other.index
    }
}

impl<I, F, O> Eq for FnMapIndex<I, F, O>
where
    I: VariableRange + Eq,
    F: Fn(I::Value<'_>) -> O,
    O: Copy + std::fmt::Debug,
{
}

impl<I, F, O> FnMapIndex<I, F, O>
where
    I: VariableRange,
    F: Fn(I::Value<'_>) -> O,
    O: Copy + std::fmt::Debug,
{
    /// Create a new FnMapIndex with the given underlying index and mapping function.
    pub const fn new(index: I, map_fn: F) -> Self {
        Self {
            index,
            map_fn,
            _phantom: PhantomData,
        }
    }
}

impl<I, F, O> VariableRange for FnMapIndex<I, F, O>
where
    I: VariableRange + 'static,
    F: Fn(I::Value<'_>) -> O + 'static,
    O: Copy + std::fmt::Debug + 'static,
{
    type Value<'a>
        = FnMapValue<O>
    where
        Self: 'a;

    /// Returns an iterator over all mapped values in the index.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let map_fn = &self.map_fn;
        (0..self.index.size()).map(move |i| {
            let original = self.index.unflatten_index_value(i);
            let mapped = map_fn(original);
            FnMapValue::new(i, mapped)
        })
    }

    /// Returns the mapped value for a given flat index.
    fn unflatten_index_value<'a>(&'a self, index: usize) -> Self::Value<'a> {
        if index >= self.index.size() {
            panic!(
                "Index out of bounds: {} (max {})",
                index,
                self.index.size() - 1
            );
        }
        let original = self.index.unflatten_index_value(index);
        let mapped = (self.map_fn)(original);
        FnMapValue::new(index, mapped)
    }

    /// Returns the number of values in the index.
    fn size(&self) -> usize {
        self.index.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};

    #[test]
    fn test_fn_map_index() {
        // Create a numeric range index from 0 to 10
        let range_index = NumericRangeIndex::<i32, ()>::new(0, 10);

        // Create a function that maps i32 to a tuple of (i32, i32)
        let map_fn = |v: NumericValue<i32, ()>| (v.index, v.index * 2);

        // Create a FnMapIndex that maps the numeric values to tuples
        let fn_map_index = FnMapIndex::new(range_index, map_fn);

        // Check the size
        assert_eq!(fn_map_index.size(), 10);

        // Test iteration
        let values: Vec<_> = fn_map_index.iter().collect();
        assert_eq!(values.len(), 10);
        assert_eq!(values[0].mapped, (0, 0));
        assert_eq!(values[9].mapped, (9, 18));

        // Test unflatten_index_value
        let value = fn_map_index.unflatten_index_value(5);
        assert_eq!(value.flat_index, 5);
        assert_eq!(value.mapped, (5, 10));
    }
}
