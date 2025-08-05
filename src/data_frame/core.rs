//! Core DataFrame struct and basic methods.
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use crate::mapped_index::sparse_numeric_index::SparseNumericIndex;
use crate::mapped_index::MappedIndex;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::ops::Index;

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
///
/// # Example
/// ```
/// use slice_and_dice::data_frame::core::DataFrame;
/// use slice_and_dice::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
/// // Tag type to mark the index dimension
/// #[derive(Debug)]
/// struct Row;
/// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
/// let data = vec![10, 20, 30];
/// let df = DataFrame::new(index.clone(), data.clone());
/// assert_eq!(df.index, index);
/// assert_eq!(df.data, data);
/// assert_eq!(*df.get(NumericValue::new(1)), 20);
/// assert_eq!(*df.get_flat(2), 30);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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
    I: MappedIndex + Clone,
    D: Index<usize> + IntoIterator,
{
    /// Construct a new DataFrame from index and data.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
    /// // Tag type to mark the index dimension
    /// #[derive(Debug)]
    /// struct Row;
    /// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
    /// let data = vec![10, 20, 30];
    /// let df = DataFrame::new(index.clone(), data.clone());
    /// assert_eq!(df.index, index);
    /// assert_eq!(df.data, data);
    /// ```
    pub fn new(index: I, data: D) -> Self {
        Self { index, data }
    }

    /// Get a reference to the data for a given index value.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
    /// // Tag type to mark the index dimension
    /// #[derive(Debug)]
    /// struct Row;
    /// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
    /// let data = vec![10, 20, 30];
    /// let df = DataFrame::new(index, data);
    /// assert_eq!(*df.get(NumericValue::new(1)), 20);
    /// ```
    pub fn get<'a>(&'a self, value: I::Value<'a>) -> &'a D::Output {
        &self.data[self.index.flatten_index_value(value)]
    }

    /// Get a reference to the data for a given flat index.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
    /// // Tag type to mark the index dimension
    /// #[derive(Debug)]
    /// struct Row;
    /// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
    /// let data = vec![10, 20, 30];
    /// let df = DataFrame::new(index, data);
    /// assert_eq!(*df.get_flat(2), 30);
    /// ```
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }
}

impl<I, T> DataFrame<I, Vec<T>>
where
    I: MappedIndex + Clone,
{
    /// Apply a function to all values in the DataFrame, returning a new DataFrame with mapped data.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
    /// #[derive(Debug)]
    /// struct Row;
    /// let df = DataFrame::new(NumericRangeIndex::<i32, Row>::new(0, 3), vec![1, 2, 3]);
    /// let df2 = df.map(|x| x * 10);
    /// assert_eq!(df2.data, vec![10, 20, 30]);
    /// assert_eq!(df2.index, NumericRangeIndex::<i32, Row>::new(0, 3));
    /// ```
    pub fn map<U, F>(&self, mut f: F) -> DataFrame<I, Vec<U>>
    where
        F: FnMut(&T) -> U,
    {
        let data = self.data.iter().map(|v| f(v)).collect();
        DataFrame {
            index: self.index.clone(),
            data,
        }
    }

    /// Create a DataFrame from an index by mapping each value in the index to some value through a user-provided function.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
    /// #[derive(Debug)]
    /// struct Row;
    /// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
    /// let df = DataFrame::<NumericRangeIndex<i32, Row>, Vec<i32>>::build_from_index(&index, |v| v.index * 10);
    /// assert_eq!(df.data, vec![0, 10, 20]);
    /// assert_eq!(df.index, index);
    /// ```
    pub fn build_from_index<F, U>(index: &I, mut f: F) -> DataFrame<I, Vec<U>>
    where
        F: FnMut(I::Value<'_>) -> U,
    {
        let data = index.iter().map(|v| f(v)).collect();
        DataFrame {
            index: index.clone(),
            data,
        }
    }

    /// Create a DataFrame from an index by mapping each value in the index to some value through a user-provided function,
    /// using parallel execution with rayon.
    ///
    /// This function is only available when the "rayon" feature is enabled.
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "rayon")]
    /// # {
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
    /// #[derive(Debug)]
    /// struct Row;
    /// let index = NumericRangeIndex::<i32, Row>::new(0, 3);
    /// let df = DataFrame::<NumericRangeIndex<i32, Row>, Vec<i32>>::build_from_index_par(&index, |v| v.index * 10);
    /// assert_eq!(df.data, vec![0, 10, 20]);
    /// assert_eq!(df.index, index);
    /// # }
    /// ```
    #[cfg(feature = "rayon")]
    pub fn build_from_index_par<F, U>(index: &I, f: F) -> DataFrame<I, Vec<U>>
    where
        F: Fn(I::Value<'_>) -> U + Send + Sync,
        U: Send,
    {
        let data = index
            .iter()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|&v| f(v))
            .collect();
        DataFrame {
            index: index.clone(),
            data,
        }
    }
}

impl<I, D> DataFrame<CompoundIndex<(I,)>, D>
where
    I: MappedIndex + 'static,
    D: Index<usize>,
{
    /// Collapse a DataFrame with a (I,) index tuple into one with just an I index.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::DataFrame;
    /// use slice_and_dice::mapped_index::compound_index::CompoundIndex;
    /// use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
    /// #[derive(Debug)]
    /// struct Tag;
    /// let index = CompoundIndex { indices: (NumericRangeIndex::<i32, Tag>::new(0, 3),) };
    /// let df = DataFrame::new(index, vec![1, 2, 3]);
    /// let df2 = df.collapse_single_index();
    /// assert_eq!(df2.index, NumericRangeIndex::<i32, Tag>::new(0, 3));
    /// assert_eq!(df2.data, vec![1, 2, 3]);
    /// ```
    pub fn collapse_single_index(self) -> DataFrame<I, D> {
        DataFrame {
            index: self.index.collapse_single(),
            data: self.data,
        }
    }
}

/// Extension trait for creating a DataFrame with a NumericRangeIndex from an iterator.
///
/// # Example
/// ```
/// use slice_and_dice::data_frame::core::DataFrameFromNumericExt;
/// use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
/// // Tag type to mark the index dimension
/// #[derive(Debug)]
/// struct Row;
/// let df = (0..3).to_numeric_dataframe::<Row>();
/// assert_eq!(df.index, NumericRangeIndex::<i32, Row>::new(0, 3));
/// assert_eq!(df.data, vec![0, 1, 2]);
/// ```
pub trait DataFrameFromNumericExt: Sized {
    fn to_numeric_dataframe<Tag: 'static + std::fmt::Debug>(
        self,
    ) -> DataFrame<NumericRangeIndex<i32, Tag>, Vec<Self::Item>>
    where
        Self: Iterator,
        Self::Item: 'static;
}

impl<I> DataFrameFromNumericExt for I
where
    I: Iterator,
    I::Item: 'static,
{
    fn to_numeric_dataframe<Tag: 'static + std::fmt::Debug>(
        self,
    ) -> DataFrame<NumericRangeIndex<i32, Tag>, Vec<I::Item>> {
        let data: Vec<I::Item> = self.collect();
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

/// Extension trait for creating a DataFrame with a SparseNumericIndex from an iterator of (I, T).
///
/// # Example
/// ```
/// use slice_and_dice::data_frame::core::DataFrameFromSparseNumericExt;
/// use slice_and_dice::mapped_index::sparse_numeric_index::SparseNumericIndex;
/// #[derive(Debug)]
/// struct Row;
/// let df = [(10i64, "a"), (20i64, "b")]
///     .into_iter()
///     .to_sparse_numeric_dataframe::<Row>();
/// assert_eq!(df.index.indices, vec![10, 20]);
/// assert_eq!(df.data, vec!["a", "b"]);
/// ```
pub trait DataFrameFromSparseNumericExt<I, T>: Iterator<Item = (I, T)> + Sized
where
    I: Copy + PartialOrd + 'static,
    T: 'static,
{
    fn to_sparse_numeric_dataframe<Tag: 'static>(
        self,
    ) -> DataFrame<SparseNumericIndex<I, Tag>, Vec<T>>;
}

impl<Itr, I, T> DataFrameFromSparseNumericExt<I, T> for Itr
where
    Itr: Iterator<Item = (I, T)>,
    I: Copy + PartialOrd + 'static,
    T: 'static,
{
    fn to_sparse_numeric_dataframe<Tag: 'static>(
        self,
    ) -> DataFrame<SparseNumericIndex<I, Tag>, Vec<T>> {
        let (indices, data): (Vec<I>, Vec<T>) = self.unzip();
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

    // Removed test_new, test_get, test_get_flat (now doctests)

    #[test]
    fn test_nonzero_start_index() {
        let index = NumericRangeIndex::<i32, Tag>::new(5, 8); // Range: 5, 6, 7
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
        use crate::data_frame::core::DataFrameFromNumericExt;
        let df = (0..5).to_numeric_dataframe::<Tag>();
        assert_eq!(df.index.start, 0);
        assert_eq!(df.index.end, 5);
        assert_eq!(df.data, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_from_iter_sparse_numeric() {
        use crate::data_frame::core::DataFrameFromSparseNumericExt;
        let df = [(10i64, "a"), (20i64, "b")]
            .into_iter()
            .to_sparse_numeric_dataframe::<Tag>();
        assert_eq!(df.index.indices, vec![10i64, 20i64]);
        assert_eq!(df.data, vec!["a", "b"]);
    }

    #[test]
    fn test_map() {
        use crate::data_frame::core::DataFrame;
        use crate::mapped_index::numeric_range_index::NumericRangeIndex;
        #[derive(Debug, PartialEq)]
        struct Row;
        let df = DataFrame::new(NumericRangeIndex::<i32, Row>::new(0, 3), vec![1, 2, 3]);
        let df2 = df.map(|x| x * 2);
        assert_eq!(df2.data, vec![2, 4, 6]);
        assert_eq!(df2.index, NumericRangeIndex::<i32, Row>::new(0, 3));
    }

    #[test]
    fn test_collapse_single_index() {
        use crate::mapped_index::compound_index::CompoundIndex;
        use crate::mapped_index::numeric_range_index::NumericRangeIndex;
        #[derive(Debug, PartialEq)]
        struct Tag;
        let index = CompoundIndex {
            indices: (NumericRangeIndex::<i32, Tag>::new(0, 3),),
        };
        let df = DataFrame::new(index, vec![1, 2, 3]);
        let df2 = df.collapse_single_index();
        assert_eq!(df2.index, NumericRangeIndex::<i32, Tag>::new(0, 3));
        assert_eq!(df2.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_build_from_index() {
        use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
        #[derive(Debug, PartialEq)]
        struct Row;
        let index = NumericRangeIndex::<i32, Row>::new(0, 3);
        let df =
            DataFrame::<NumericRangeIndex<i32, Row>, Vec<i32>>::build_from_index(&index, |v| {
                v.index * 10
            });
        assert_eq!(df.data, vec![0, 10, 20]);
        assert_eq!(df.index, index);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn test_build_from_index_par() {
        use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
        #[derive(Debug, PartialEq)]
        struct Row;
        let index = NumericRangeIndex::<i32, Row>::new(0, 3);
        let df =
            DataFrame::<NumericRangeIndex<i32, Row>, Vec<i32>>::build_from_index_par(&index, |v| {
                v.index * 10
            });
        assert_eq!(df.data, vec![0, 10, 20]);
        assert_eq!(df.index, index);
    }
}
