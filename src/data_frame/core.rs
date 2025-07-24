//! Core DataFrame struct and basic methods.
use crate::mapped_index::MappedIndex;
use std::ops::Index;

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
pub struct DataFrame<I, D>
where
    I: MappedIndex,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
}

impl<I, D> DataFrame<I, D>
where
    I: MappedIndex,
    D: Index<usize>,
{
    /// Construct a new DataFrame from index and data.
    pub fn new(index: I, data: D) -> Self {
        Self { index, data }
    }

    /// Get a reference to the data for a given index value.
    pub fn get<'a>(&'a self, value: I::Value<'a>) -> &'a D::Output {
        &self.data[self.index.flatten_index_value(value)]
    }

    /// Get a reference to the data for a given flat index.
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};

    #[derive(Debug)]
    struct Tag;

    #[test]
    fn test_new() {
        let index = NumericRangeIndex::<Tag>::new(0, 3);
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index.clone(), data.clone());
        assert_eq!(df.index, index);
        assert_eq!(df.data, data);
    }

    #[test]
    fn test_get() {
        let index = NumericRangeIndex::<Tag>::new(0, 3);
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        assert_eq!(*df.get(NumericValue::new(1)), 20);
    }

    #[test]
    fn test_get_flat() {
        let index = NumericRangeIndex::<Tag>::new(0, 3);
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        assert_eq!(*df.get_flat(2), 30);
    }

    #[test]
    fn test_nonzero_start_index() {
        let index = NumericRangeIndex::<Tag>::new(5, 8); // Range: 5, 6, 7
        let data = vec![100, 200, 300];
        let df = DataFrame::new(index.clone(), data.clone());
        assert_eq!(df.index, index);
        assert_eq!(df.data, data);
        assert_eq!(*df.get(NumericValue::new(5)), 100);
        assert_eq!(*df.get(NumericValue::new(6)), 200);
        assert_eq!(*df.get(NumericValue::new(7)), 300);
        assert_eq!(*df.get_flat(0), 100);
        assert_eq!(*df.get_flat(2), 300);
    }
}
