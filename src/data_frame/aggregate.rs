//! Aggregation logic for DataFrame over a dimension.
use super::core::DataFrame;
use crate::data_frame::strided_index_view::StridedIndexView;
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::{
    CompoundIndex, HLConcat, HListConcat, IndexHlist, PluckSplit, RefIndexHList,
};
use frunk::hlist::Plucker;
use itertools::Itertools;
use num_traits::{FromPrimitive, Zero};
use peano::NonNeg;
use std::ops::Index;

impl<Indices: IndexHlist, D: Index<usize>> DataFrame<CompoundIndex<Indices>, D>
where
    D::Output: Clone,
{
    /// Iterate over the dimension specified by typenum.
    ///
    /// Returns an iterator that yields StridedIndexViews for each combination of indices
    /// except the dimension being iterated over.
    pub fn iter_over_dim<'a, Idx>(&'a self) -> impl Iterator<Item = StridedIndexView<'a, D>> + 'a
    where
        Indices::Refs<'a>: PluckSplit<Idx>,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Left: RefIndexHList,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Extract: VariableRange,
        <Indices::Refs<'a> as PluckSplit<Idx>>::Right: RefIndexHList,
    {
        let (l, m, r) = self.index.indices.refs().pluck_split();
        let l_size = l.size();
        let m_size = m.size();
        let r_size = r.size();
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
            RefIndexHList + HListConcat<<Indices as PluckSplit<Idx>>::Right>,
        <Indices as PluckSplit<Idx>>::Extract: VariableRange,
        <Indices as PluckSplit<Idx>>::Right: RefIndexHList,
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
            RefIndexHList + HListConcat<<Indices as PluckSplit<Idx>>::Right>,
        <Indices as PluckSplit<Idx>>::Extract: VariableRange,
        <Indices as PluckSplit<Idx>>::Right: RefIndexHList,
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
