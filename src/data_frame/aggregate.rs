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
    for<'b> Indices::Refs<'b>: PluckSplit<At, Extract = &'a Middle>,
    for<'b> <Indices::Refs<'b> as PluckSplit<At>>::Left: RefIndexHList,
    for<'b> <Indices::Refs<'b> as PluckSplit<At>>::Right: RefIndexHList,
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

impl<'a, Indices: IndexHlist, D: Index<usize>, At> Iterator
    for DimIter<'a, Indices, D, At, <Indices::Refs<'a> as PluckSplit<At>>::Extract>
where
    for<'b> Indices::Refs<'b>: PluckSplit<At>,
    for<'b> <Indices::Refs<'b> as PluckSplit<At>>::Left: RefIndexHList,
    for<'b> <Indices::Refs<'b> as PluckSplit<At>>::Extract: VariableRange + 'b,
    for<'b> <Indices::Refs<'b> as PluckSplit<At>>::Right: RefIndexHList,
    for<'b> <<Indices::Refs<'b> as PluckSplit<At>>::Extract as VariableRange>::Value<'b>: 'a,
{
    type Item = (
        <<Indices::Refs<'a> as PluckSplit<At>>::Extract as VariableRange>::Value<'a>,
        StridedIndexView<'a, D>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.l_size * self.m_size * self.r_size {
            let current_index = self.index;
            self.index += 1;

            // Instead of creating a local middle variable, use the data's indices directly
            let middle_value = self.m.unflatten_index_value(current_index);

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
    // /// Iterate over the dimension specified by typenum.
    // ///
    // /// Returns an iterator that yields StridedIndexViews for each combination of indices
    // /// except the dimension being iterated over.
    // pub fn iter_over_dim<'a, Idx>(&'a self) -> impl Iterator<Item = (<<Indices::Refs<'a> as PluckSplit<Idx>>::Extract as VariableRange>::Value<'a>,StridedIndexView<'a, D>)> + 'a
    // where
    //     Indices::Refs<'a>: PluckSplit<Idx>,
    //     <Indices::Refs<'a> as PluckSplit<Idx>>::Left: RefIndexHList,
    //     <Indices::Refs<'a> as PluckSplit<Idx>>::Extract: VariableRange + 'a,
    //     <Indices::Refs<'a> as PluckSplit<Idx>>::Right: RefIndexHList,
    // {
    //     todo!()
    // }

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
