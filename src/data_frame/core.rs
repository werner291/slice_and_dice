//! Core DataFrame struct and basic methods.
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::CompoundIndex;
use frunk::HList;
use rand::Rng;
use rand::seq::IteratorRandom;
use std::ops::Index;

pub trait FrameData: Index<usize> {
    fn len(&self) -> usize;

    fn iter(&self) -> impl Iterator<Item = &Self::Output> + '_ {
        (0..self.len()).map(|i| &self[i])
    }
}

/// Macro to allow direct field access for tests and internal code.
/// This is used to avoid having to update all the direct field accesses in the codebase.
#[macro_export]
macro_rules! allow_direct_field_access {
    () => {
        #[cfg(any(test, feature = "internal"))]
        pub struct DataFrame<I, D>
        where
            I: VariableRange,
            D: FrameData,
        {
            /// The index structure (categorical, numeric, compound, etc.).
            pub index: I,
            /// The data collection, indexable by flat index.
            pub data: D,
        }

        #[cfg(not(any(test, feature = "internal")))]
        pub struct DataFrame<I, D>
        where
            I: VariableRange,
            D: FrameData,
        {
            /// The index structure (categorical, numeric, compound, etc.).
            index: I,
            /// The data collection, indexable by flat index.
            data: D,
        }
    };
}

impl<D> FrameData for Vec<D> {
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash)]
pub struct DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
}

impl<I, D> DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    /// Construct a DataFrame from an index and a data collection.
    ///
    /// # Examples
    ///
    /// Basic usage with a numeric range index and Vec data:
    ///
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// let df = DataFrame::new(idx, vec![10, 20, 30]);
    /// assert_eq!(df.n_rows(), 3);
    /// ```
    pub fn new(index: I, data: D) -> Self {
        assert_eq!(
            index.size(),
            data.len(),
            "Index and data must have the same length"
        );
        Self { index, data }
    }

    /// Returns a reference to the index.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// use slice_and_dice::mapped_index::VariableRange;
    /// let idx = NumericRangeIndex::<i32>::new(0, 2);
    /// let df = DataFrame::new(idx.clone(), vec![1, 2]);
    /// assert_eq!(df.index().size(), idx.size());
    /// ```
    pub fn index(&self) -> &I {
        &self.index
    }

    /// Returns a reference to the data.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// let df = DataFrame::new(idx, vec![10, 20, 30]);
    /// assert_eq!(df.data()[1], 20);
    /// ```
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Returns a mutable reference to the data.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 2);
    /// let mut df = DataFrame::new(idx, vec![1, 2]);
    /// df.data_mut()[0] = 9;
    /// assert_eq!(df.data()[0], 9);
    /// ```
    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }

    /// Returns a reference to the data at the given index.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// let df = DataFrame::new(idx, vec![10, 20, 30]);
    /// assert_eq!(*df.data_at(2), 30);
    /// ```
    pub fn data_at(&self, index: usize) -> &D::Output {
        &self.data[index]
    }

    /// Iterate over (index_value, &data) pairs.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// let df = DataFrame::new(idx, vec![10, 20, 30]);
    /// let collected: Vec<(i32, i32)> = df.iter().map(|(i, v)| (i, *v)).collect();
    /// assert_eq!(collected, vec![(0,10), (1,20), (2,30)]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (I::Value<'_>, &D::Output)> + '_ {
        self.index.iter().zip(self.data.iter())
    }

    //noinspection RsNeedlessLifetimes
    /// Choose n rows without replacement using the provided RNG.
    /// If n >= length, all rows are returned (without guaranteed order).
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// use rand::SeedableRng;
    /// use rand::rngs::StdRng;
    /// let idx = NumericRangeIndex::<i32>::new(0, 5);
    /// let df = DataFrame::new(idx, vec![10, 20, 30, 40, 50]);
    /// let mut rng = StdRng::seed_from_u64(123);
    /// let picked: Vec<(i32, i32)> = df.choose_rows(&mut rng, 3).map(|(i, v)| (i, *v)).collect();
    /// assert_eq!(picked.len(), 3);
    /// ```
    pub fn choose_rows<'a, R>(
        &'a self,
        rng: &mut R,
        n: usize,
    ) -> impl Iterator<Item = (I::Value<'a>, &'a D::Output)>
    where
        R: Rng + ?Sized,
    {
        let len = self.data.len();
        let amount = n.min(len);
        // Sample unique indices without replacement
        let indices: Vec<usize> = (0..len).choose_multiple(rng, amount);
        indices
            .into_iter()
            .map(move |i| (self.index.unflatten_index_value(i), &self.data[i]))
    }

    /// Return number of rows in the DataFrame.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 2);
    /// let df = DataFrame::new(idx, vec![1, 2]);
    /// assert_eq!(df.n_rows(), 2);
    /// ```
    pub fn n_rows(&self) -> usize {
        self.data.len()
    }
}

impl<I, T> DataFrame<I, Vec<T>>
where
    I: VariableRange + Clone,
{
    /// Map each element of the DataFrame's data to a new value, keeping the same index.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// let df = DataFrame::new(idx, vec![1, 2, 3]);
    /// let df2 = df.map(|v| v * 10);
    /// assert_eq!(df2.data(), &vec![10, 20, 30]);
    /// ```
    pub fn map<U, F>(&self, mut f: F) -> DataFrame<I, Vec<U>>
    where
        F: FnMut(&T) -> U,
    {
        let data = self.data().iter().map(|v| f(v)).collect();
        DataFrame::new(self.index().clone(), data)
    }

    /// Build a DataFrame by mapping each index value to a data value.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::{DataFrame, NumericRangeIndex};
    /// let idx = NumericRangeIndex::<i32>::new(0, 4);
    /// let df = DataFrame::build_from_index(idx, |i| i * i);
    /// assert_eq!(df.data(), &vec![0, 1, 4, 9]);
    /// ```
    pub fn build_from_index<F>(index: I, mut f: F) -> DataFrame<I, Vec<T>>
    where
        F: FnMut(I::Value<'_>) -> T,
    {
        let data = index.iter().map(|v| f(v)).collect();
        DataFrame::new(index, data)
    }

    #[cfg(feature = "rayon")]
    pub fn build_from_index_par<F>(index: I, f: F) -> DataFrame<I, Vec<T>>
    where
        I: VariableRange + Clone + Sync,
        T: Send,
        F: Fn(I::Value<'_>) -> T + Sync,
    {
        use rayon::prelude::*;
        let size = index.size();
        let data: Vec<T> = (0..size)
            .into_par_iter()
            .map(|i| {
                let v = index.unflatten_index_value(i);
                f(v)
            })
            .collect();
        DataFrame::new(index, data)
    }
}

impl<I, D> DataFrame<CompoundIndex<HList![I]>, D>
where
    I: VariableRange,
    D: FrameData,
{
    pub fn collapse_single_index(self) -> DataFrame<I, D> {
        DataFrame::new(self.index.indices.head, self.data)
    }
}

impl<I, D> Index<usize> for DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    type Output = D::Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::compound_index::CompoundIndex;
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use frunk::hlist::{HNil, h_cons};

    #[test]
    fn test_dataframe_constructor_validation() {
        // Test with matching lengths
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        assert_eq!(df.index().size(), 3);
        assert_eq!(df.data().len(), 3);

        // Test with mismatched lengths - should panic
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20]; // Only 2 elements
        let result = std::panic::catch_unwind(|| {
            DataFrame::new(index, data);
        });
        assert!(
            result.is_err(),
            "DataFrame::new should panic when index and data lengths don't match"
        );
    }

    #[test]
    fn test_accessor_methods() {
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let mut df = DataFrame::new(index.clone(), data.clone());

        // Test index()
        assert_eq!(df.index().size(), 3);

        // Test data()
        assert_eq!(df.data().len(), 3);
        assert_eq!(df.data()[0], 10);
        assert_eq!(df.data()[1], 20);
        assert_eq!(df.data()[2], 30);

        // Test data_mut()
        df.data_mut()[0] = 100;
        assert_eq!(df.data()[0], 100);

        // Test data_at()
        assert_eq!(*df.data_at(0), 100);
        assert_eq!(*df.data_at(1), 20);
        assert_eq!(*df.data_at(2), 30);
    }

    #[test]
    fn test_index_operator() {
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);

        assert_eq!(df[0], 10);
        assert_eq!(df[1], 20);
        assert_eq!(df[2], 30);
    }

    #[test]
    fn test_iter() {
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);

        let mut iter = df.iter();
        assert_eq!(iter.next(), Some((0, &10)));
        assert_eq!(iter.next(), Some((1, &20)));
        assert_eq!(iter.next(), Some((2, &30)));
        assert_eq!(iter.next(), None);

        // Test collecting into a vector
        let collected: Vec<_> = df.iter().collect();
        assert_eq!(collected, vec![(0, &10), (1, &20), (2, &30)]);
    }

    #[test]
    fn test_map() {
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);

        // Map to double the values
        let mapped_df = df.map(|x| x * 2);

        assert_eq!(mapped_df.data().len(), 3);
        assert_eq!(mapped_df[0], 20);
        assert_eq!(mapped_df[1], 40);
        assert_eq!(mapped_df[2], 60);

        // Map to strings
        let string_df = df.map(|x| x.to_string());

        assert_eq!(string_df.data().len(), 3);
        assert_eq!(string_df[0], "10");
        assert_eq!(string_df[1], "20");
        assert_eq!(string_df[2], "30");
    }

    #[test]
    fn test_build_from_index() {
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]

        // Build a DataFrame where each value is the index squared
        let df = DataFrame::build_from_index(index, |i| i * i);

        assert_eq!(df.data().len(), 3);
        assert_eq!(df[0], 0);
        assert_eq!(df[1], 1);
        assert_eq!(df[2], 4);
    }

    #[test]
    fn test_collapse_single_index() {
        // Create a compound index with a single NumericRangeIndex
        let numeric_index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let indices = h_cons(numeric_index.clone(), HNil);
        let compound_index = CompoundIndex::new(indices);
        let data = vec![10, 20, 30];

        let compound_df = DataFrame::new(compound_index, data);

        // Collapse the compound index
        let collapsed_df = compound_df.collapse_single_index();

        // Verify the collapsed DataFrame has the correct index and data
        assert_eq!(collapsed_df.index().size(), 3);
        assert_eq!(collapsed_df.data().len(), 3);
        assert_eq!(collapsed_df[0], 10);
        assert_eq!(collapsed_df[1], 20);
        assert_eq!(collapsed_df[2], 30);
    }

    #[test]
    fn test_framedata_for_vec() {
        let vec = vec![1, 2, 3];

        // Test len()
        assert_eq!(vec.len(), 3);

        // Test iter()
        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_choose_rows_without_replacement() {
        use crate::mapped_index::numeric_range::NumericRangeIndex;
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let index = NumericRangeIndex::<i32>::new(0, 5); // indices: 0..=4
        let data = vec![10, 20, 30, 40, 50];
        let df = DataFrame::new(index, data);

        // Deterministic selection of 3 unique rows
        let mut rng = StdRng::seed_from_u64(42);
        let picked: Vec<_> = df.choose_rows(&mut rng, 3).collect();
        assert_eq!(picked.len(), 3);

        // Ensure uniqueness of index values
        let mut idx_vals: Vec<i32> = picked.iter().map(|(i, _)| *i).collect();
        idx_vals.sort();
        idx_vals.dedup();
        assert_eq!(idx_vals.len(), 3);

        // When n exceeds length, we get all rows (order not guaranteed)
        let mut rng2 = StdRng::seed_from_u64(7);
        let picked_all: Vec<_> = df.choose_rows(&mut rng2, 10).collect();
        assert_eq!(picked_all.len(), 5);
        let mut all_idx: Vec<i32> = picked_all.iter().map(|(i, _)| *i).collect();
        all_idx.sort();
        assert_eq!(all_idx, vec![0, 1, 2, 3, 4]);
    }
}
