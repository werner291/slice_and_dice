//! Aggregation logic for DataFrame over a dimension.

use super::core::DataFrame;
use crate::data_frame::core::FrameData;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::{
    CompoundIndex, HLConcat, HListConcat, IndexHlist, PluckSplit, RefIndexHList,
};
use itertools::Itertools;
use num_traits::Zero;

impl<Indices: IndexHlist, D: FrameData> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
    /// Iterate over the dimension specified by typenum.
    ///
    /// Returns an iterator that yields StridedIndexViews for each combination of indices
    /// except the dimension being iterated over.
    pub fn iter_over_dim<'a, Idx, Middle: VariableRange + 'a>(
        &'a self,
    ) -> DataFrame<&'a Middle, Vec<Vec<&'a D::Output>>>
    where
        Indices::Refs<'a>: PluckSplit<Idx, Extract = &'a Middle>,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Left: RefIndexHList,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Right: RefIndexHList,
    {
        let (l, m, r) = self.index().indices.refs().pluck_split();

        let m_size = m.size();
        let r_size = r.size();

        let data = m
            .iter()
            .enumerate()
            .map(|(m_i, m_v)| {
                l.iter()
                    .enumerate()
                    .flat_map(|(l_i, l_v)| {
                        r.iter().enumerate().map(move |(r_i, r_v)| {
                            let flat_index = l_i * (m_size * r_size) + m_i * r_size + r_i;
                            &self.data()[flat_index]
                        })
                    })
                    .collect_vec() // TODO: can we do without copying? Some kinda fancy index translation store?
            })
            .collect_vec();

        DataFrame::new(m, data)
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
        let refs = self.index().indices.clone();
        let (l, m, r) = refs.pluck_split();
        let l_size = l.size();
        let m_size = m.size();
        let r_size = r.size();
        let agg_data = (0..l_size)
            .flat_map(|l_i| {
                let f = &f;
                let data = self.data();
                (0..r_size).map(move |r_i| {
                    f(StridedIndexView::new(
                        l_i * m_size * r_size + r_i,
                        r_size,
                        m_size,
                        data,
                    ))
                })
            })
            .collect_vec();
        DataFrame::new(CompoundIndex::new(l.concat(r)), agg_data)
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
    use crate::mapped_index::compound_index::{Dim0, Dim1};
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use frunk::hlist::h_cons;
    use frunk::indices::{Here, There};
    use frunk::{HNil, hlist, hlist_pat};
    use itertools::iproduct;

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

    // Test iter_over_dim functionality
    #[test]
    fn test_iter_over_dim() {
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

        // Iterate over first dimension (rows)
        let iter_rows = df.iter_over_dim::<Here, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [0, 1] and data containing references
        // to the original data elements for each row
        assert_eq!(iter_rows.index.size(), 2);
        assert_eq!(iter_rows.data.len(), 2);

        // First row should contain [10, 20, 30]
        assert_eq!(iter_rows.data[0].len(), 3);
        assert_eq!(*iter_rows.data[0][0], 10);
        assert_eq!(*iter_rows.data[0][1], 20);
        assert_eq!(*iter_rows.data[0][2], 30);

        // Second row should contain [40, 50, 60]
        assert_eq!(iter_rows.data[1].len(), 3);
        assert_eq!(*iter_rows.data[1][0], 40);
        assert_eq!(*iter_rows.data[1][1], 50);
        assert_eq!(*iter_rows.data[1][2], 60);

        // Iterate over second dimension (columns)
        let iter_cols = df.iter_over_dim::<There<Here>, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [10, 11, 12] and data containing references
        // to the original data elements for each column
        assert_eq!(iter_cols.index.size(), 3);
        assert_eq!(iter_cols.data.len(), 3);

        // First column should contain [10, 40]
        assert_eq!(iter_cols.data[0].len(), 2);
        assert_eq!(*iter_cols.data[0][0], 10);
        assert_eq!(*iter_cols.data[0][1], 40);

        // Second column should contain [20, 50]
        assert_eq!(iter_cols.data[1].len(), 2);
        assert_eq!(*iter_cols.data[1][0], 20);
        assert_eq!(*iter_cols.data[1][1], 50);

        // Third column should contain [30, 60]
        assert_eq!(iter_cols.data[2].len(), 2);
        assert_eq!(*iter_cols.data[2][0], 30);
        assert_eq!(*iter_cols.data[2][1], 60);
    }

    // Test iter_over_dim with a more complex 4D array
    #[test]
    fn test_iter_over_dim_complex() {
        // Create a 4D DataFrame with dimensions 3x4x2x5
        let index1 = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let index2 = NumericRangeIndex::<i32>::new(10, 14); // [10, 11, 12, 13]
        let index3 = NumericRangeIndex::<i32>::new(20, 22); // [20, 21]
        let index4 = NumericRangeIndex::<i32>::new(30, 35); // [30, 31, 32, 33, 34]

        // Create compound index with all four dimensions
        let indices = h_cons(
            index1.clone(),
            h_cons(
                index2.clone(),
                h_cons(index3.clone(), h_cons(index4.clone(), HNil)),
            ),
        );
        let compound_index = CompoundIndex::new(indices);

        // Create data for a 3x4x2x5 array (120 elements)
        let data: Vec<i32> = (1..=120).collect();

        let df = DataFrame::new(compound_index, data);

        // Iterate over first dimension (dim0)
        let iter_dim0 = df.iter_over_dim::<Here, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [0, 1, 2] and data containing references
        // to the original data elements for each slice of dim0
        assert_eq!(iter_dim0.index.size(), 3);
        assert_eq!(iter_dim0.data.len(), 3);

        // Each slice should have 4*2*5 = 40 elements
        assert_eq!(iter_dim0.data[0].len(), 40);
        assert_eq!(iter_dim0.data[1].len(), 40);
        assert_eq!(iter_dim0.data[2].len(), 40);

        // Check some specific values in the first slice
        assert_eq!(*iter_dim0.data[0][0], 1); // First element of first slice
        assert_eq!(*iter_dim0.data[0][39], 40); // Last element of first slice

        // Check some specific values in the second slice
        assert_eq!(*iter_dim0.data[1][0], 41); // First element of second slice
        assert_eq!(*iter_dim0.data[1][39], 80); // Last element of second slice

        // Check some specific values in the third slice
        assert_eq!(*iter_dim0.data[2][0], 81); // First element of third slice
        assert_eq!(*iter_dim0.data[2][39], 120); // Last element of third slice

        // Iterate over second dimension (dim1)
        let iter_dim1 = df.iter_over_dim::<There<Here>, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [10, 11, 12, 13] and data containing references
        // to the original data elements for each slice of dim1
        assert_eq!(iter_dim1.index.size(), 4);
        assert_eq!(iter_dim1.data.len(), 4);

        // Each slice should have 3*2*5 = 30 elements
        assert_eq!(iter_dim1.data[0].len(), 30);
        assert_eq!(iter_dim1.data[1].len(), 30);
        assert_eq!(iter_dim1.data[2].len(), 30);
        assert_eq!(iter_dim1.data[3].len(), 30);

        // Iterate over third dimension (dim2)
        let iter_dim2 = df.iter_over_dim::<There<There<Here>>, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [20, 21] and data containing references
        // to the original data elements for each slice of dim2
        assert_eq!(iter_dim2.index.size(), 2);
        assert_eq!(iter_dim2.data.len(), 2);

        // Each slice should have 3*4*5 = 60 elements
        assert_eq!(iter_dim2.data[0].len(), 60);
        assert_eq!(iter_dim2.data[1].len(), 60);

        // Iterate over fourth dimension (dim3)
        let iter_dim3 = df.iter_over_dim::<There<There<There<Here>>>, NumericRangeIndex<i32>>();

        // Result should be a DataFrame with index [30, 31, 32, 33, 34] and data containing references
        // to the original data elements for each slice of dim3
        assert_eq!(iter_dim3.index.size(), 5);
        assert_eq!(iter_dim3.data.len(), 5);

        // Each slice should have 3*4*2 = 24 elements
        assert_eq!(iter_dim3.data[0].len(), 24);
        assert_eq!(iter_dim3.data[1].len(), 24);
        assert_eq!(iter_dim3.data[2].len(), 24);
        assert_eq!(iter_dim3.data[3].len(), 24);
        assert_eq!(iter_dim3.data[4].len(), 24);
    }
}
