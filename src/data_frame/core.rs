//! Core DataFrame struct and basic methods.
use crate::mapped_index::MappedIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use crate::mapped_index::sparse_numeric_index::SparseNumericIndex;
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

impl<T: 'static> DataFrame<NumericRangeIndex<T>, Vec<T>> {
    /// Constructs a DataFrame from an iterator of values, using a numeric index (0..N).
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// let df = DataFrame::from_iter_numeric([1, 2, 3]);
    /// assert_eq!(df.index.start, 0);
    /// assert_eq!(df.index.end, 3);
    /// assert_eq!(df.data, vec![1, 2, 3]);
    /// ```
    pub fn from_iter_numeric<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let data: Vec<T> = iter.into_iter().collect();
        let len = data.len() as i32;
        DataFrame {
            index: NumericRangeIndex {
                start: 0,
                end: len,
                _phantom: std::marker::PhantomData,
            },
            data,
        }
    }
}

impl<T: 'static> DataFrame<SparseNumericIndex<T>, Vec<T>> {
    /// Constructs a DataFrame from an iterator of (i32, value) pairs, using a sparse numeric index.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// let df = DataFrame::from_iter_sparse_numeric([(10, "a"), (20, "b")]);
    /// assert_eq!(df.index.indices, vec![10, 20]);
    /// assert_eq!(df.data, vec!["a", "b"]);
    /// ```
    pub fn from_iter_sparse_numeric<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (i32, T)>,
    {
        let (indices, data): (Vec<i32>, Vec<T>) = iter.into_iter().unzip();
        let indices: Vec<i64> = indices.into_iter().map(|x| x as i64).collect();
        DataFrame {
            index: SparseNumericIndex {
                indices,
                _phantom: std::marker::PhantomData,
            },
            data,
        }
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

    #[test]
    fn test_from_iter_numeric() {
        let df = DataFrame::from_iter_numeric(0..5);
        assert_eq!(df.index.start, 0);
        assert_eq!(df.index.end, 5);
        assert_eq!(df.data, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_from_iter_sparse_numeric() {
        let df = DataFrame::from_iter_sparse_numeric([(10, "a"), (20, "b")]);
        assert_eq!(df.index.indices, vec![10i64, 20i64]);
        assert_eq!(df.data, vec!["a", "b"]);
    }
}
