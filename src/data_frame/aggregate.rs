//! Aggregation logic for DataFrame over a dimension.

use super::core::DataFrame;
use crate::data_frame::core::FrameData;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::data_frame::util::tri_product_index_view::TriProductIndexView;
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::{CompoundIndex, IndexHlist};
use crate::mapped_index::util::as_refs::{AsRefs, HRefs};
use crate::mapped_index::util::concat::{HLConcat, HListConcat};
use crate::mapped_index::util::pluck_split::{
    PluckAt, PluckLeft, PluckRemainder, PluckRight, PluckSplit, PluckSplitImpl,
};
use itertools::Itertools;
use num_traits::Zero;

pub struct IterOverDim<'a, Data, Plucked, Left, Right, Remainder>
where
    Data: FrameData,
    Plucked: VariableRange + 'a,
    Left: IndexHlist,
    Right: IndexHlist,
    Remainder: IndexHlist,
{
    at_index: usize,
    data: &'a Data,
    plucked_index: &'a Plucked,
    left_index: Left,
    right_index: Right,
    remainder_index: Remainder,
}

impl<'a, Data, Plucked, Left, Right, Remainder> Iterator
    for IterOverDim<'a, Data, Plucked, Left, Right, Remainder>
where
    Data: FrameData,
    Plucked: VariableRange + 'a,
    Left: IndexHlist,
    Right: IndexHlist,
    Remainder: IndexHlist,
{
    type Item = (
        <Plucked as VariableRange>::Value<'a>,
        DataFrame<CompoundIndex<Remainder>, TriProductIndexView<'a, Data>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        let m_i = self.at_index;
        self.at_index += 1;

        let l_size = self.left_index.size();
        let m_size = self.plucked_index.size();
        let r_size = self.right_index.size();
        if m_i >= m_size {
            return None;
        }

        let view = TriProductIndexView::new(l_size, m_size, r_size, m_i, self.data);

        let mv = self.plucked_index.unflatten_index_value(m_i);
        let index = CompoundIndex::new(self.remainder_index.clone());

        Some((mv, DataFrame::new(index, view)))
    }
}

impl<Indices, D> DataFrame<CompoundIndex<Indices>, D>
where
    Indices: IndexHlist + AsRefs,
    D: FrameData,
{
    /// Iterate over the dimension specified by typenum.
    ///
    /// Returns an iterator that yields StridedIndexViews for each combination of indices
    /// except the dimension being iterated over.
    ///
    /// # Example
    /// ```
    /// use slice_and_dice::data_frame::core::{DataFrame, FrameData};
    /// use slice_and_dice::mapped_index::numeric_range::NumericRangeIndex;
    /// use frunk::hlist;
    /// use itertools::Itertools;
    /// use slice_and_dice::mapped_index::compound_index::{CompoundIndex, Dim0};
    ///
    /// // Create 2D DataFrame with dimensions 2x3
    /// let index1 = NumericRangeIndex::<i32>::new(0, 2);  // [0, 1]
    /// let index2 = NumericRangeIndex::<i32>::new(10, 13); // [10, 11, 12]
    /// let indices = CompoundIndex::new(hlist![index1, index2]);
    /// let df = DataFrame::new(indices, vec![1, 2, 3, 4, 5, 6]);
    ///
    /// // Iterate over first dimension (rows)
    /// for (ix, row_df) in df.iter_over_dim::<Dim0>() {
    ///     // ix will be 0, then 1
    ///     // row_df will contain [1,2,3] for first row, [4,5,6] for second row
    ///     println!("Row {}: {:?}", ix, row_df.data().iter().collect_vec());
    /// }
    /// ```
    pub fn iter_over_dim<'a, DimIx: 'a>(
        &'a self,
    ) -> IterOverDim<
        'a,
        D,
        PluckAt<DimIx, Indices>,
        PluckLeft<DimIx, HRefs<'a, Indices>>,
        PluckRight<DimIx, HRefs<'a, Indices>>,
        PluckRemainder<DimIx, HRefs<'a, Indices>>,
    >
    where
        Indices: IndexHlist + AsRefs + PluckSplitImpl<DimIx>,
        D: FrameData,
        HRefs<'a, Indices>: PluckSplitImpl<DimIx, Extract = &'a PluckAt<DimIx, Indices>>,
        PluckLeft<DimIx, HRefs<'a, Indices>>: HListConcat<PluckRight<DimIx, HRefs<'a, Indices>>>,
        PluckAt<DimIx, Indices>: VariableRange + 'a,
        PluckLeft<DimIx, HRefs<'a, Indices>>: IndexHlist + Copy,
        PluckRight<DimIx, HRefs<'a, Indices>>: IndexHlist + Copy,
        PluckRemainder<DimIx, HRefs<'a, Indices>>: IndexHlist,
    {
        let (left, middle, right) = self.index().indices.as_refs().pluck_split();

        IterOverDim {
            at_index: 0,
            plucked_index: middle,
            left_index: left,
            right_index: right,
            data: &self.data,
            remainder_index: left.concat(right),
        }
    }

    /// Aggregate over the dimension specified by typenum.
    pub fn aggregate_over_dim<'a, Idx, F, R>(
        self,
        f: F,
    ) -> DataFrame<
        CompoundIndex<
            HLConcat<
                <Indices as PluckSplitImpl<Idx>>::Left,
                <Indices as PluckSplitImpl<Idx>>::Right,
            >,
        >,
        Vec<R>,
    >
    where
        Indices: 'a,
        Indices: PluckSplitImpl<Idx>,
        <Indices as PluckSplitImpl<Idx>>::Left:
            IndexHlist + HListConcat<<Indices as PluckSplitImpl<Idx>>::Right>,
        <Indices as PluckSplitImpl<Idx>>::Extract: VariableRange,
        <Indices as PluckSplitImpl<Idx>>::Right: IndexHlist,
        HLConcat<<Indices as PluckSplitImpl<Idx>>::Left, <Indices as PluckSplitImpl<Idx>>::Right>:
            IndexHlist,
        F: for<'any> Fn(StridedIndexView<'any, D>) -> R,
    {
        let refs = self.index().indices.clone();
        let (l, m, r) = refs.pluck_split_impl();
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
            HLConcat<
                <Indices as PluckSplitImpl<Idx>>::Left,
                <Indices as PluckSplitImpl<Idx>>::Right,
            >,
        >,
        Vec<<D::Output as std::ops::Div<f64>>::Output>,
    >
    where
        Indices: PluckSplitImpl<Idx>,
        <Indices as PluckSplitImpl<Idx>>::Left:
            IndexHlist + HListConcat<<Indices as PluckSplitImpl<Idx>>::Right>,
        <Indices as PluckSplitImpl<Idx>>::Extract: VariableRange,
        <Indices as PluckSplitImpl<Idx>>::Right: IndexHlist,
        HLConcat<<Indices as PluckSplitImpl<Idx>>::Left, <Indices as PluckSplitImpl<Idx>>::Right>:
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
    use frunk::HNil;
    use frunk::hlist::h_cons;
    use frunk::indices::{Here, There};
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
        let iter_rows = df.iter_over_dim::<Here>();

        // Result should be a DataFrame with index [0, 1] and data containing references
        // to the original data elements for each row
        for (i, (ix, row_df)) in iter_rows.enumerate() {
            assert_eq!(ix as i32, i as i32);
            assert_eq!(row_df.data().len(), 3);

            if i == 0 {
                // First row should contain [10, 20, 30]
                assert_eq!(row_df.data()[0], 10);
                assert_eq!(row_df.data()[1], 20);
                assert_eq!(row_df.data()[2], 30);
            } else {
                // Second row should contain [40, 50, 60]
                assert_eq!(row_df.data()[0], 40);
                assert_eq!(row_df.data()[1], 50);
                assert_eq!(row_df.data()[2], 60);
            }
        }

        // Iterate over second dimension (columns)
        let iter_cols = df.iter_over_dim::<There<Here>>();

        // Result should be a DataFrame with index [10, 11, 12] and data containing references
        // to the original data elements for each column
        for (i, (ix, col_df)) in iter_cols.enumerate() {
            assert_eq!(ix as i32, i as i32 + 10);
            assert_eq!(col_df.data().len(), 2);

            match i {
                0 => {
                    // First column should contain [10, 40]
                    assert_eq!(col_df.data()[0], 10);
                    assert_eq!(col_df.data()[1], 40);
                }
                1 => {
                    // Second column should contain [20, 50]
                    assert_eq!(col_df.data()[0], 20);
                    assert_eq!(col_df.data()[1], 50);
                }
                2 => {
                    // Third column should contain [30, 60]
                    assert_eq!(col_df.data()[0], 30);
                    assert_eq!(col_df.data()[1], 60);
                }
                _ => panic!("Unexpected column index"),
            }
        }
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
        let iter_dim0 = df.iter_over_dim::<Here>();

        // Result should be a DataFrame with index [0, 1, 2] and data containing references
        // to the original data elements for each slice of dim0
        let mut count = 0;
        for (index_value, slice_df) in iter_dim0 {
            assert_eq!(index_value, count);
            assert_eq!(slice_df.data().len(), 40); // 4*2*5 = 40 elements per slice

            // Check first and last element of each slice
            assert_eq!(slice_df.data()[0], count * 40 + 1); // First element
            assert_eq!(slice_df.data()[39], (count + 1) * 40); // Last element
            count += 1;
        }
        assert_eq!(count, 3);

        // Iterate over second dimension (dim1)
        let iter_dim1 = df.iter_over_dim::<There<Here>>();

        // Result should be a DataFrame with index [10, 11, 12, 13] and data containing references
        // to the original data elements for each slice of dim1
        let mut count = 0;
        for (index_value, slice_df) in iter_dim1 {
            assert_eq!(index_value, count + 10);
            assert_eq!(slice_df.data().len(), 30); // 3*2*5 = 30 elements per slice
            count += 1;
        }
        assert_eq!(count, 4);

        // Iterate over third dimension (dim2)
        let iter_dim2 = df.iter_over_dim::<There<There<Here>>>();

        // Result should be a DataFrame with index [20, 21] and data containing references
        // to the original data elements for each slice of dim2
        let mut count = 0;
        for (index_value, slice_df) in iter_dim2 {
            assert_eq!(index_value, count + 20);
            assert_eq!(slice_df.data().len(), 60); // 3*4*5 = 60 elements per slice
            count += 1;
        }
        assert_eq!(count, 2);

        // Iterate over fourth dimension (dim3)
        let iter_dim3 = df.iter_over_dim::<There<There<There<Here>>>>();

        // Result should be a DataFrame with index [30, 31, 32, 33, 34] and data containing references
        // to the original data elements for each slice of dim3
        let mut count = 0;
        for (index_value, slice_df) in iter_dim3 {
            assert_eq!(index_value, count + 30);
            assert_eq!(slice_df.data().len(), 24); // 3*4*2 = 24 elements per slice
            count += 1;
        }
        assert_eq!(count, 5);
    }
}
