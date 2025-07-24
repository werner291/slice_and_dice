use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::{CompoundIndex, IndexRefTuple, IndexTuple};
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use crate::mapped_index::tuple_utils::{
    Extract, ExtractAt, ExtractLeft, ExtractRemainder, ExtractRight, TupleConcat, TupleExtract,
};
use itertools::Itertools;
use peano::NonNeg;
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

    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack<StackTag: 'static>(
        dfs: impl IntoIterator<Item = DataFrame<I, D>>,
    ) -> Option<DataFrame<CompoundIndex<(NumericRangeIndex<StackTag>, I)>, Vec<D::Output>>>
    where
        I: Clone + PartialEq + 'static, // TODO: Can we do with a looser bound than 'static ?
        D::Output: Clone,
    {
        let dfs: Vec<DataFrame<I, D>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return None;
        }
        // Check all inner indices are equal
        let first_index = &dfs[0].index;
        for df in &dfs[1..] {
            // Compare by iterating over all values
            if first_index != &df.index {
                panic!("Indices mismatched.");
            }
        }

        // Build the compound index: outer is a numeric range, inner is the shared index
        let outer_index = NumericRangeIndex::new(0, dfs.len() as i32);

        let compound_index = CompoundIndex {
            indices: (outer_index, first_index.clone()),
        };
        // Flatten the data
        let mut data = Vec::new();
        for df in &dfs {
            for i in 0..df.index.size() {
                data.push(df.data[i].clone());
            }
        }
        Some(DataFrame::new(compound_index, data))
    }
}

pub struct StridedIndexView<'a, D: Index<usize>> {
    base: usize,
    stride: usize,
    n_strides: usize,
    view_into: &'a D,
}

impl<'a, D> Iterator for StridedIndexView<'a, D>
where
    D: Index<usize>,
{
    type Item = &'a D::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n_strides == 0 {
            None
        } else {
            let item = &self.view_into[self.base];
            self.base += self.stride;
            self.n_strides -= 1;
            Some(item)
        }
    }
}

impl<Indices: IndexTuple, D: Index<usize>> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
    /// Aggregate over the dimension specified by typenum.
    pub fn aggregate_over_dim<N: NonNeg, F, R>(
        self,
        mut f: F,
    ) -> DataFrame<CompoundIndex<ExtractRemainder<N, Indices>>, Vec<R>>
    where
        Indices: TupleExtract<N>,
        <Indices as TupleExtract<N>>::Before: TupleConcat, // Direct requirement
        F: for<'a> Fn(StridedIndexView<'a, D>) -> R,
        ExtractLeft<N, Indices>: IndexTuple,
        Extract<N, Indices>: MappedIndex,
        ExtractRemainder<N, Indices>: IndexTuple,
        ExtractRight<N, Indices>: IndexTuple,
    {
        let (l, m, r) = self.index.indices.extract_at::<N>();

        let l_size = r.as_ref_tuple().size();
        let m_size = m.size();
        let r_size = r.as_ref_tuple().size();

        let agg_data = (0..l_size)
            .flat_map(|l_i| {
                let f = &f;
                let data = &self.data;
                (0..r_size).map(move |r_i| {
                    f(StridedIndexView {
                        base: l_i * r_size + r_i,
                        stride: r_size,
                        n_strides: m_size,
                        view_into: data,
                    })
                })
            })
            .collect_vec();

        DataFrame {
            index: CompoundIndex::new(l.concat(r)),
            data: agg_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
    use peano::P1;

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

    struct StackTag;

    #[test]
    fn test_stack() {
        let index = NumericRangeIndex::<Tag>::new(0, 2);
        let df1 = DataFrame::new(index.clone(), vec![10, 20]);
        let df2 = DataFrame::new(index.clone(), vec![30, 40]);
        let stacked = DataFrame::stack::<StackTag>(vec![df1, df2]).unwrap();

        assert_eq!(stacked.index.indices.0.size(), 2); // Outer index size
        assert_eq!(stacked.index.indices.1, index); // Inner index
        assert_eq!(stacked.data, vec![10, 20, 30, 40]); // Flattened data
    }

    #[test]
    fn test_aggregate_over_dim() {
        // Define a compound index (outer: NumericRangeIndex, inner: NumericRangeIndex)
        let outer_index = NumericRangeIndex::<Tag>::new(0, 2); // Outer index: 0, 1
        let inner_index = NumericRangeIndex::<Tag>::new(0, 3); // Inner index: 0, 1, 2
        let compound_index = CompoundIndex {
            indices: (outer_index.clone(), inner_index.clone()),
        };

        // Create a DataFrame with the compound index and some data
        let data = vec![10, 20, 30, 40, 50, 60]; // 2 outer * 3 inner = 6 elements
        let df = DataFrame::new(compound_index, data);

        // Define an aggregation function (e.g., sum over the inner dimension)
        let agg_df = df.aggregate_over_dim::<P1, _, i32>(|iter| iter.cloned().sum::<i32>());

        // Expected result: sum over the inner dimension
        let expected_index = outer_index; // Remaining index after aggregation
        let expected_data = vec![60, 150]; // [10+20+30, 40+50+60]

        // Verify the result
        assert_eq!(agg_df.index.collapse_single(), expected_index);
        assert_eq!(agg_df.data, expected_data);
    }
}
