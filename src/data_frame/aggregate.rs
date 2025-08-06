//! Aggregation logic for DataFrame over a dimension.
use super::core::DataFrame;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::mapped_index::compound_index::{CompoundIndex, IndexRefTuple, IndexTuple};
use crate::mapped_index::VariableRange;
use crate::tuple_utils::{
    Extract, ExtractAt, ExtractLeft, ExtractRemainder, ExtractRight, TupleConcat, TupleExtract,
};
use itertools::Itertools;
use num_traits::{FromPrimitive, Zero};
use peano::NonNeg;
use std::ops::Index;

impl<Indices: IndexTuple + Clone, D: Index<usize>> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
    /// Iterate over the dimension specified by typenum.
    ///
    /// Returns an iterator that yields StridedIndexViews for each combination of indices
    /// except the dimension being iterated over.
    pub fn iter_over_dim<'a, N: NonNeg>(
        &'a self,
    ) -> impl Iterator<Item = StridedIndexView<'a, D>> + 'a
    where
        Indices: TupleExtract<N>,
        <Indices as TupleExtract<N>>::Before: TupleConcat,
        ExtractLeft<N, Indices>: IndexTuple,
        Extract<N, Indices>: VariableRange,
        ExtractRemainder<N, Indices>: IndexTuple,
        ExtractRight<N, Indices>: IndexTuple,
    {
        let (l, m, r) = self.index.indices.clone().extract_at::<N>();
        let l_size = l.as_ref_tuple().size();
        let m_size = m.size();
        let r_size = r.as_ref_tuple().size();
        let data = &self.data;

        (0..l_size).flat_map(move |l_i| {
            (0..r_size).map(move |r_i| StridedIndexView {
                base: l_i * m_size * r_size + r_i,
                stride: r_size,
                n_strides: m_size,
                view_into: data,
            })
        })
    }

    /// Aggregate over the dimension specified by typenum.
    pub fn aggregate_over_dim<N: NonNeg, F, R>(
        self,
        f: F,
    ) -> DataFrame<CompoundIndex<ExtractRemainder<N, Indices>>, Vec<R>>
    where
        Indices: TupleExtract<N>,
        <Indices as TupleExtract<N>>::Before: TupleConcat,
        F: for<'a> Fn(StridedIndexView<'a, D>) -> R,
        ExtractLeft<N, Indices>: IndexTuple,
        Extract<N, Indices>: VariableRange,
        ExtractRemainder<N, Indices>: IndexTuple,
        ExtractRight<N, Indices>: IndexTuple,
    {
        let (l, m, r) = self.index.indices.extract_at::<N>();
        let l_size = l.as_ref_tuple().size();
        let m_size = m.size();
        let r_size = r.as_ref_tuple().size();
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
    pub fn mean_over_dim<N: NonNeg>(
        self,
    ) -> DataFrame<
        CompoundIndex<ExtractRemainder<N, Indices>>,
        Vec<<D::Output as std::ops::Div<f64>>::Output>,
    >
    where
        Indices: TupleExtract<N>,
        <Indices as TupleExtract<N>>::Before: TupleConcat,
        D::Output: Copy + Zero + std::ops::AddAssign + std::ops::Div<f64>,
        <D::Output as std::ops::Div<f64>>::Output: Copy,
        ExtractLeft<N, Indices>: IndexTuple,
        Extract<N, Indices>: VariableRange,
        ExtractRemainder<N, Indices>: IndexTuple,
        ExtractRight<N, Indices>: IndexTuple,
    {
        self.aggregate_over_dim::<N, _, <D::Output as std::ops::Div<f64>>::Output>(|iter| {
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
    use peano::P1;

    #[test]
    fn test_aggregate_over_dim() {
        let outer_index = NumericRangeIndex::<i32>::new(0, 2);
        let inner_index = NumericRangeIndex::<i32>::new(0, 3);
        let compound_index = CompoundIndex {
            indices: (outer_index.clone(), inner_index.clone()),
        };
        let data = vec![10, 20, 30, 40, 50, 60];
        let df = DataFrame::new(compound_index, data);
        let agg_df = df.aggregate_over_dim::<P1, _, i32>(|iter| iter.cloned().sum::<i32>());
        let expected_index = outer_index;
        let expected_data = vec![60, 150];
        assert_eq!(agg_df.index.collapse_single(), expected_index);
        assert_eq!(agg_df.data, expected_data);
    }

    #[test]
    fn test_mean_over_dim() {
        let outer_index = NumericRangeIndex::<i32>::new(0, 2);
        let inner_index = NumericRangeIndex::<i32>::new(0, 3);
        let compound_index = CompoundIndex {
            indices: (outer_index.clone(), inner_index.clone()),
        };
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        let df = DataFrame::new(compound_index, data);
        let mean_df = df.mean_over_dim::<P1>();
        let expected_index = outer_index;
        let expected_data = vec![(10.0 + 20.0 + 30.0) / 3.0, (40.0 + 50.0 + 60.0) / 3.0];
        assert_eq!(mean_df.index.collapse_single(), expected_index);
        assert!((mean_df.data[0] - expected_data[0]).abs() < 1e-8);
        assert!((mean_df.data[1] - expected_data[1]).abs() < 1e-8);
    }

    #[test]
    fn test_iter_over_dim() {
        let outer_index = NumericRangeIndex::<i32>::new(0, 2);
        let inner_index = NumericRangeIndex::<i32>::new(0, 3);
        let compound_index = CompoundIndex {
            indices: (outer_index.clone(), inner_index.clone()),
        };
        let data = vec![10, 20, 30, 40, 50, 60];
        let df = DataFrame::new(compound_index, data);

        // Collect all the StridedIndexViews into a vector of vectors
        let views: Vec<Vec<i32>> = df
            .iter_over_dim::<P1>()
            .map(|view| view.copied().collect())
            .collect();

        // We expect 2 views (one for each outer index)
        assert_eq!(views.len(), 2);

        // Each view should have 3 elements (the size of the inner index)
        assert_eq!(views[0].len(), 3);
        assert_eq!(views[1].len(), 3);

        // Check the values in each view
        assert_eq!(views[0], vec![10, 20, 30]);
        assert_eq!(views[1], vec![40, 50, 60]);
    }
}
