//! Aggregation logic for DataFrame over a dimension.
use super::core::DataFrame;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::mapped_index::compound_index::{CompoundIndex, IndexRefTuple, IndexTuple};
use crate::mapped_index::MappedIndex;
use crate::tuple_utils::{
    Extract, ExtractAt, ExtractLeft, ExtractRemainder, ExtractRight, TupleConcat, TupleExtract,
};
use itertools::Itertools;
use peano::NonNeg;
use std::ops::Index;

impl<Indices: IndexTuple, D: Index<usize>> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
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
        Extract<N, Indices>: MappedIndex,
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
    /// The mean is computed as f64, regardless of the input type.
    pub fn mean_over_dim<N: NonNeg>(
        self,
    ) -> DataFrame<CompoundIndex<ExtractRemainder<N, Indices>>, Vec<f64>>
    where
        Indices: TupleExtract<N>,
        <Indices as TupleExtract<N>>::Before: TupleConcat,
        D::Output: Copy + Into<f64>,
        ExtractLeft<N, Indices>: IndexTuple,
        Extract<N, Indices>: MappedIndex,
        ExtractRemainder<N, Indices>: IndexTuple,
        ExtractRight<N, Indices>: IndexTuple,
    {
        self.aggregate_over_dim::<N, _, f64>(|iter| {
            let n = iter.len();
            if n == 0 {
                f64::NAN
            } else {
                iter.copied().map(Into::into).sum::<f64>() / n as f64
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range_index::NumericRangeIndex;
    use peano::P1;

    #[derive(Debug)]
    struct Tag;

    #[test]
    fn test_aggregate_over_dim() {
        let outer_index = NumericRangeIndex::<i32, Tag>::new(0, 2);
        let inner_index = NumericRangeIndex::<i32, Tag>::new(0, 3);
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
        let outer_index = NumericRangeIndex::<i32, Tag>::new(0, 2);
        let inner_index = NumericRangeIndex::<i32, Tag>::new(0, 3);
        let compound_index = CompoundIndex {
            indices: (outer_index.clone(), inner_index.clone()),
        };
        let data = vec![10, 20, 30, 40, 50, 60];
        let df = DataFrame::new(compound_index, data);
        let mean_df = df.mean_over_dim::<P1>();
        let expected_index = outer_index;
        let expected_data = vec![(10.0 + 20.0 + 30.0) / 3.0, (40.0 + 50.0 + 60.0) / 3.0];
        assert_eq!(mean_df.index.collapse_single(), expected_index);
        assert!((mean_df.data[0] - expected_data[0]).abs() < 1e-8);
        assert!((mean_df.data[1] - expected_data[1]).abs() < 1e-8);
    }
}
