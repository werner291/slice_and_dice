use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use std::marker::PhantomData;
use std::ops::Index;

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
pub struct DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
    _phantom: PhantomData<&'idx Idx>,
}

impl<'idx, I, D, Idx> DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// Construct a new DataFrame from index and data.
    pub fn new(index: I, data: D) -> Self {
        Self { index, data, _phantom: PhantomData }
    }
    /// Get a reference to the data for a given index value.
    pub fn get(&'idx self, value: I::Value) -> &D::Output {
        &self.data[self.index.to_flat_index(value)]
    }
    /// Get a reference to the data for a given flat index.
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }
    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack<'a, J, E, It>(dfs: It) -> Result<DataFrame<'idx, CompoundIndex<(J, I)>, Vec<D::Output>, (usize, Idx)>, &'static str>
    where
        I: Clone + 'idx,
        D: Clone,
        D::Output: Clone,
        J: MappedIndex<'idx, usize> + Clone,
        It: IntoIterator<Item = DataFrame<'idx, I, D, Idx>>,
    {
        let dfs: Vec<DataFrame<'idx, I, D, Idx>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return Err("No dataframes to stack");
        }
        // Check all inner indices are equal
        let first_index = &dfs[0].index;
        for df in &dfs[1..] {
            // Compare by iterating over all values
            if df.index.size() != first_index.size() || !df.index.iter().eq(first_index.iter()) {
                return Err("All inner indices must be equal to stack");
            }
        }
        // Build the compound index: outer is a numeric range, inner is the shared index
        let outer_index = crate::mapped_index::numeric_range_index::NumericRangeIndex::new(0, dfs.len() as i32);
        let compound_index = CompoundIndex { indices: (outer_index, first_index.clone()) };
        // Flatten the data
        let mut data = Vec::new();
        for df in &dfs {
            for i in 0..df.index.size() {
                data.push(df.data[i].clone());
            }
        }
        Ok(DataFrame::new(compound_index, data))
    }
}

impl<'idx, A, B, D, IdxA, IdxB> DataFrame<'idx, CompoundIndex<(A, B)>, D, (IdxA, IdxB)>
where
    A: MappedIndex<'idx, IdxA> + Clone,
    B: MappedIndex<'idx, IdxB> + Clone,
    D: Index<usize>,
{
    /// Aggregate over the second dimension (B) of a 2D compound index.
    /// The function f receives an iterator over &D::Output for each fixed A.
    pub fn aggregate_over_b<R, F>(&self, mut f: F) -> DataFrame<'idx, A, Vec<R>, IdxA>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
    {
        let a_index = self.index.indices.0.clone();
        let b_index = self.index.indices.1.clone();
        let mut result = Vec::with_capacity(a_index.size());
        for a_val in a_index.iter() {
            let mut values = (0..b_index.size()).map(|b_i| {
                let b_val = b_index.from_flat_index(b_i);
                let idx = (a_val, b_val);
                &self.data[self.index.to_flat_index(idx)]
            });
            result.push(f(&mut values));
        }
        DataFrame::new(a_index, result)
    }
}

impl<'idx, I, D, Idx> Index<I::Value> for DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    type Output = D::Output;
    fn index(&self, value: I::Value) -> &Self::Output {
        &self.data[self.index.to_flat_index(value)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::categorical_index::{CategoricalIndex, CategoricalValue};
    use crate::mapped_index::MappedIndex;
    use crate::mapped_index::compound_index::CompoundIndex;
    use crate::mapped_index::numeric_range_index::NumericRangeIndex;
    use std::marker::PhantomData;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct Tag;

    #[test]
    fn test_dataframe_get_and_index() {
        let index = CategoricalIndex::new(vec!["a", "b", "c"]);
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        let val = df.index.from_flat_index(1);
        assert_eq!(df.get(val), &20);
        assert_eq!(df[val], 20);
        assert_eq!(df.get_flat(2), &30);
    }

    #[test]
    fn test_dataframe_round_trip() {
        let index = CategoricalIndex::new(vec!["x", "y", "z"]);
        let data = vec![100, 200, 300];
        let df = DataFrame::new(index, data);
        for flat in 0..df.index.size() {
            let val = df.index.from_flat_index(flat);
            let round = df.index.to_flat_index(val);
            assert_eq!(flat, round);
            assert_eq!(df.get(val), &df.get_flat(flat));
        }
    }

    #[test]
    fn test_stack_success() {
        let index = CategoricalIndex::new(vec!["a", "b"]);
        let df1 = DataFrame::new(index.clone(), vec![1, 2]);
        let df2 = DataFrame::new(index.clone(), vec![3, 4]);
        let stacked = DataFrame::stack([df1, df2]).expect("should stack");
        // Compound index: outer is NumericRangeIndex, inner is CategoricalIndex
        assert_eq!(format!("{:?}", stacked.index.indices.0), format!("{:?}", NumericRangeIndex::new(0, 2)));
        assert_eq!(format!("{:?}", stacked.index.indices.1), format!("{:?}", index));
        assert_eq!(stacked.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_stack_incompatible() {
        let index1 = CategoricalIndex::new(vec!["a", "b"]);
        let index2 = CategoricalIndex::new(vec!["a", "c"]);
        let df1 = DataFrame::new(index1, vec![1, 2]);
        let df2 = DataFrame::new(index2, vec![3, 4]);
        let result = DataFrame::stack([df1, df2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_aggregate_over_b_sum() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b):
        // ("x", 0) = 1, ("x", 1) = 2, ("x", 2) = 3
        // ("y", 0) = 4, ("y", 1) = 5, ("y", 2) = 6
        let data = vec![1, 2, 3, 4, 5, 6];
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| it.copied().sum::<i32>());
        assert_eq!(agg.data, vec![6, 15]); // 1+2+3, 4+5+6
        assert_eq!(agg.index, a);
    }

    #[test]
    fn test_aggregate_over_b_count() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 2);
        let data = vec![10, 20, 30, 40]; // ("x",0)=10, ("x",1)=20, ("y",0)=30, ("y",1)=40
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| it.count());
        assert_eq!(agg.data, vec![2, 2]);
        assert_eq!(agg.index, a);
    }

    #[test]
    fn test_aggregate_over_b_min_max() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b):
        // ("x", 0) = 7, ("x", 1) = 2, ("x", 2) = 5
        // ("y", 0) = 4, ("y", 1) = 9, ("y", 2) = 6
        let data = vec![7, 2, 5, 4, 9, 6];
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| {
            let mut it = it.copied();
            let min = it.clone().min().unwrap();
            let max = it.max().unwrap();
            (min, max)
        });
        assert_eq!(agg.data, vec![(2, 7), (4, 9)]); // (min, max) for each a
        assert_eq!(agg.index, a);
    }
} 