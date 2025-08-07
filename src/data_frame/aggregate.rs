//! Aggregation logic for DataFrame over a dimension.

use super::core::DataFrame;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::{
    CompoundIndex, HLConcat, HListConcat, IndexHlist, PluckSplit, RefIndexHList,
};
use itertools::Itertools;
use num_traits::Zero;
use std::marker::PhantomData;
use std::ops::Index;

// Modify the DimIter struct to use higher-ranked trait bounds
pub struct DimIter<'a, Indices: IndexHlist, D: Index<usize>, At, Middle> {
    data: &'a DataFrame<CompoundIndex<Indices>, D>,
    index: usize,
    m: &'a Middle,
    l_size: usize,
    m_size: usize,
    r_size: usize,
    at: PhantomData<At>,
}

impl<'a, Middle: VariableRange, Indices: IndexHlist, D: Index<usize>, At>
    DimIter<'a, Indices, D, At, Middle>
where
    Indices::Refs<'a>: PluckSplit<At, Extract = &'a Middle>,
    <Indices::Refs<'a> as PluckSplit<At>>::Left: RefIndexHList,
    <Indices::Refs<'a> as PluckSplit<At>>::Right: RefIndexHList,
{
    pub fn new(data: &'a DataFrame<CompoundIndex<Indices>, D>) -> Self {
        let (l, m, r) = data.index.indices.refs().pluck_split();
        let l_size = l.size();
        let m_size = m.size();
        let r_size = r.size();

        Self {
            data,
            index: 0,
            m,
            l_size,
            m_size,
            r_size,
            at: PhantomData,
        }
    }
}

impl<'a, Middle: VariableRange, Indices: IndexHlist, D: Index<usize>, At> Iterator
    for DimIter<'a, Indices, D, At, Middle>
where
    Indices::Refs<'a>: PluckSplit<At, Extract = &'a Middle>,
    <Indices::Refs<'a> as PluckSplit<At>>::Left: RefIndexHList,
    <Indices::Refs<'a> as PluckSplit<At>>::Right: RefIndexHList,
{
    type Item = (Middle::Value<'a>, StridedIndexView<'a, D>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.l_size * self.m_size * self.r_size {
            let current_index = self.index;
            self.index += 1;

            // Calculate the middle dimension index from the flat index
            // For a compound index with components (L, M, R), the middle index is:
            // M_index = (flat_index / R.size()) % M.size()
            let middle_index = (current_index / self.r_size) % self.m_size;
            let middle_value = self.m.unflatten_index_value(middle_index);

            Some((
                middle_value,
                StridedIndexView {
                    base: current_index,
                    stride: self.r_size,
                    n_strides: self.m_size,
                    view_into: &self.data.data,
                },
            ))
        } else {
            None
        }
    }
}

impl<Indices: IndexHlist, D: Index<usize>> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
    /// Iterate over the dimension specified by typenum.
    ///
    /// Returns an iterator that yields StridedIndexViews for each combination of indices
    /// except the dimension being iterated over.
    pub fn iter_over_dim<'a, Idx, Middle: VariableRange>(
        &'a self,
    ) -> DimIter<'a, Indices, D, Idx, Middle>
    where
        Indices::Refs<'a>: PluckSplit<Idx, Extract = &'a Middle>,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Left: RefIndexHList,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Right: RefIndexHList,
    {
        DimIter::<Indices, D, Idx, Middle>::new(self)
    }

    /// Aggregate over the dimension specified by typenum.
    pub fn aggregate_over_dim<'a, Idx, F, R>(
        self,
        f: F,
    ) -> DataFrame<
        CompoundIndex<
            HLConcat<<Indices as PluckSplit<Idx>>::Left, <Indices as PluckSplit<Idx>>::Right>,
        >,
        Vec<R>,
    >
    where
        Indices: 'a,
        Indices: PluckSplit<Idx>,
        <Indices as PluckSplit<Idx>>::Left:
            IndexHlist + HListConcat<<Indices as PluckSplit<Idx>>::Right>,
        <Indices as PluckSplit<Idx>>::Extract: VariableRange,
        <Indices as PluckSplit<Idx>>::Right: IndexHlist,
        HLConcat<<Indices as PluckSplit<Idx>>::Left, <Indices as PluckSplit<Idx>>::Right>:
            IndexHlist,
        F: for<'any> Fn(StridedIndexView<'any, D>) -> R,
    {
        let refs = self.index.indices;
        let (l, m, r) = refs.pluck_split();
        let l_size = l.size();
        let m_size = m.size();
        let r_size = r.size();
        let agg_data = (0..l_size)
            .flat_map(|l_i| {
                let f = &f;
                let data = &self.data;
                (0..r_size).map(move |r_i| {
                    f(StridedIndexView {
                        base: l_i * m_size * r_size + r_i,
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

    /// Compute the mean over the dimension specified by typenum.
    ///
    /// The mean is computed as the sum divided by the count (as f64), using the output of Div<f64>.
    /// Only works for types that implement Div<f64> (e.g., f64, f32).
    pub fn mean_over_dim<'a, Idx>(
        self,
    ) -> DataFrame<
        CompoundIndex<
            HLConcat<<Indices as PluckSplit<Idx>>::Left, <Indices as PluckSplit<Idx>>::Right>,
        >,
        Vec<<D::Output as std::ops::Div<f64>>::Output>,
    >
    where
        Indices: PluckSplit<Idx>,
        <Indices as PluckSplit<Idx>>::Left:
            IndexHlist + HListConcat<<Indices as PluckSplit<Idx>>::Right>,
        <Indices as PluckSplit<Idx>>::Extract: VariableRange,
        <Indices as PluckSplit<Idx>>::Right: IndexHlist,
        HLConcat<<Indices as PluckSplit<Idx>>::Left, <Indices as PluckSplit<Idx>>::Right>:
            IndexHlist,
        D::Output: Copy + Zero + std::ops::AddAssign + std::ops::Div<f64>,
        <D::Output as std::ops::Div<f64>>::Output: Copy,
    {
        self.aggregate_over_dim::<Idx, _, <D::Output as std::ops::Div<f64>>::Output>(|iter| {
            let n = iter.len();
            if n == 0 {
                panic!("mean_over_dim: cannot compute mean of zero elements");
            } else {
                let mut sum = D::Output::zero();
                for v in iter.copied() {
                    sum += v;
                }
                sum / n as f64
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use frunk::hlist::h_cons;
    use frunk::indices::{Here, There};
    use frunk::{HNil, hlist};

    // Test that reproduces the crash in DimIter::next()
    #[test]
    fn test_dim_iter_crash() {
        // Create a 3D DataFrame with dimensions 2x2x2
        let index1 = NumericRangeIndex::<i32>::new(0, 2); // [0, 1]
        let index2 = NumericRangeIndex::<i32>::new(10, 12); // [10, 11]
        let index3 = NumericRangeIndex::<i32>::new(100, 102); // [100, 101]

        // Create compound index with all three dimensions
        let indices = h_cons(
            index1.clone(),
            h_cons(index2.clone(), h_cons(index3.clone(), HNil)),
        );
        let compound_index = CompoundIndex::new(indices);

        // Create data for a 2x2x2 cube
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let df = DataFrame::new(compound_index, data);

        // Iterate over the middle dimension
        let iter = df.iter_over_dim::<There<Here>, NumericRangeIndex<i32>>();

        // Collect the results to force iteration
        let results: Vec<_> = iter.collect();

        // Verify we got the expected number of results
        // For a 3D DataFrame with dimensions 2x2x2, we should get 8 results
        // when iterating over the middle dimension
        assert_eq!(results.len(), 8);

        // Verify that the middle dimension values are correct
        // The middle dimension has values [10, 11], and each should appear 4 times
        let middle_values: Vec<_> = results.iter().map(|(value, _)| *value).collect();
        assert_eq!(middle_values.iter().filter(|&&v| v == 10).count(), 4);
        assert_eq!(middle_values.iter().filter(|&&v| v == 11).count(), 4);
    }

    // Test that mean_over_dim works correctly (which uses iter_over_dim internally)
    #[test]
    fn test_mean_over_dim() {
        // Create a 2D DataFrame with dimensions 2x3
        let index1 = NumericRangeIndex::<i32>::new(0, 2); // [0, 1]
        let index2 = NumericRangeIndex::<i32>::new(10, 13); // [10, 11, 12]

        // Create compound index with both dimensions
        let indices = h_cons(index1.clone(), h_cons(index2.clone(), HNil));
        let compound_index = CompoundIndex::new(indices);

        // Create data: [10, 20, 30, 40, 50, 60]
        // This represents a 2x3 matrix:
        // [10, 20, 30]
        // [40, 50, 60]
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0];

        let df = DataFrame::new(compound_index, data);

        // Calculate mean over first dimension (rows)
        let mean_rows = df.clone().mean_over_dim::<Here>();

        // Result should be a 1D DataFrame with 3 values: [25, 35, 45]
        // (average of each column)
        assert_eq!(mean_rows.data.len(), 3);
        assert!((mean_rows.data[0] - 25.0).abs() < 1e-10);
        assert!((mean_rows.data[1] - 35.0).abs() < 1e-10);
        assert!((mean_rows.data[2] - 45.0).abs() < 1e-10);

        // Calculate mean over second dimension (columns)
        let mean_cols = df.mean_over_dim::<There<Here>>();

        // Result should be a 1D DataFrame with 2 values: [20, 50]
        // (average of each row)
        assert_eq!(mean_cols.data.len(), 2);
        assert!((mean_cols.data[0] - 20.0).abs() < 1e-10);
        assert!((mean_cols.data[1] - 50.0).abs() < 1e-10);
    }

    // Test aggregate_over_dim with a custom aggregation function
    #[test]
    fn test_aggregate_over_dim() {
        // Create a 2D DataFrame with dimensions 2x3
        let index1 = NumericRangeIndex::<i32>::new(0, 2); // [0, 1]
        let index2 = NumericRangeIndex::<i32>::new(10, 13); // [10, 11, 12]

        // Create compound index with both dimensions
        let indices = h_cons(index1.clone(), h_cons(index2.clone(), HNil));
        let compound_index = CompoundIndex::new(indices);

        // Create data: [10, 20, 30, 40, 50, 60]
        // This represents a 2x3 matrix:
        // [10, 20, 30]
        // [40, 50, 60]
        let data = vec![10, 20, 30, 40, 50, 60];

        let df = DataFrame::new(compound_index, data);

        // Calculate sum over first dimension (rows)
        let sum_rows = df
            .clone()
            .aggregate_over_dim::<Here, _, i32>(|view| view.copied().sum());

        // Result should be a 1D DataFrame with 3 values: [50, 70, 90]
        // (sum of each column)
        assert_eq!(sum_rows.data.len(), 3);
        assert_eq!(sum_rows.data[0], 50);
        assert_eq!(sum_rows.data[1], 70);
        assert_eq!(sum_rows.data[2], 90);

        // Calculate sum over second dimension (columns)
        let sum_cols = df.aggregate_over_dim::<There<Here>, _, i32>(|view| view.copied().sum());

        // Result should be a 1D DataFrame with 2 values: [60, 150]
        // (sum of each row)
        assert_eq!(sum_cols.data.len(), 2);
        assert_eq!(sum_cols.data[0], 60);
        assert_eq!(sum_cols.data[1], 150);
    }

    // Test with a 3D DataFrame
    #[test]
    fn test_aggregate_over_dim_3d() {
        // Create a 3D DataFrame with dimensions 2x2x2
        let index1 = NumericRangeIndex::<i32>::new(0, 2); // [0, 1]
        let index2 = NumericRangeIndex::<i32>::new(10, 12); // [10, 11]
        let index3 = NumericRangeIndex::<i32>::new(100, 102); // [100, 101]

        // Create compound index with all three dimensions
        let indices = h_cons(
            index1.clone(),
            h_cons(index2.clone(), h_cons(index3.clone(), HNil)),
        );
        let compound_index = CompoundIndex::new(indices);

        // Create data for a 2x2x2 cube:
        // [
        //   [[1, 2], [3, 4]],
        //   [[5, 6], [7, 8]]
        // ]
        // Flattened as: [1, 2, 3, 4, 5, 6, 7, 8]
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let df = DataFrame::new(compound_index, data);

        // Calculate sum over middle dimension
        let sum_middle = df.aggregate_over_dim::<There<Here>, _, i32>(|view| view.copied().sum());

        // Result should be a 2D DataFrame with 2x2 values: [[4, 6], [12, 14]]
        // (sum of each slice along the middle dimension)
        assert_eq!(sum_middle.data.len(), 4);
        assert_eq!(sum_middle.data[0], 4); // 1 + 3
        assert_eq!(sum_middle.data[1], 6); // 2 + 4
        assert_eq!(sum_middle.data[2], 12); // 5 + 7
        assert_eq!(sum_middle.data[3], 14); // 6 + 8
    }

    // Test with edge cases
    #[test]
    fn test_aggregate_over_dim_edge_cases() {
        // Test with a dimension of size 1
        let index1 = NumericRangeIndex::<i32>::new(0, 1); // [0]
        let index2 = NumericRangeIndex::<i32>::new(10, 13); // [10, 11, 12]

        // Create compound index with both dimensions
        let indices = h_cons(index1.clone(), h_cons(index2.clone(), HNil));
        let compound_index = CompoundIndex::new(indices);

        // Create data: [100, 101, 102]
        let data = vec![100, 101, 102];

        let df = DataFrame::new(compound_index, data);

        // Calculate sum over first dimension (which has only one value)
        let sum_rows = df
            .clone()
            .aggregate_over_dim::<Here, _, i32>(|view| view.copied().sum());

        // Result should be a 1D DataFrame with 3 values: [100, 101, 102]
        // (since there's only one row, the sum is just the values themselves)
        assert_eq!(sum_rows.data.len(), 3);
        assert_eq!(sum_rows.data[0], 100);
        assert_eq!(sum_rows.data[1], 101);
        assert_eq!(sum_rows.data[2], 102);

        // Calculate product over second dimension
        let product_cols =
            df.aggregate_over_dim::<There<Here>, _, i32>(|view| view.copied().product());

        // Result should be a 1D DataFrame with 1 value: [100*101*102]
        assert_eq!(product_cols.data.len(), 1);
        assert_eq!(product_cols.data[0], 100 * 101 * 102);
    }
}
