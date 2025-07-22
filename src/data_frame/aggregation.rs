//! Aggregation logic for DataFrame, split from the main file for maintainability.
//! Projection traits are now in projection.rs.
use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::CompoundIndex;
use typenum::Unsigned;

// N-dimensional aggregation for CompoundIndex
impl<'idx, Indices, D, IdxTuple, N> crate::data_frame::DataFrame<'idx, CompoundIndex<Indices>, D, IdxTuple>
where
    CompoundIndex<Indices>: MappedIndex<'idx, IdxTuple> + crate::data_frame::projection::ProjectedIndex<N>,
    D: std::ops::Index<usize>,
    N: Unsigned,
    <CompoundIndex<Indices> as crate::data_frame::projection::ProjectedIndex<N>>::Remaining: Clone,
{
    pub fn aggregate_over_dim_nd<R, F>(&self, mut f: F) -> crate::data_frame::DataFrame<'idx, <CompoundIndex<Indices> as crate::data_frame::projection::ProjectedIndex<N>>::Remaining, Vec<R>, _>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
    {
        let projected_index = self.index.project_index();
        let mut result = Vec::with_capacity(projected_index.size());
        for i in 0..projected_index.size() {
            let proj_val = projected_index.from_flat_index(i);
            // For each value in the projected index, collect all values in the original index that match
            let mut values = (0..self.index.size()).filter_map(|flat| {
                let full_val = self.index.from_flat_index(flat);
                // Project out the N-th element from full_val and compare to proj_val
                let (_, remaining) = full_val.project_out();
                if remaining == proj_val {
                    Some(&self.data[flat])
                } else {
                    None
                }
            });
            result.push(f(&mut values));
        }
        crate::data_frame::DataFrame::new(projected_index, result)
    }
}

// 2D aggregation (aggregate_over_dim)
impl<'idx, A, B, D, IdxA, IdxB> crate::data_frame::DataFrame<'idx, CompoundIndex<(A, B)>, D, (IdxA, IdxB)>
where
    A: MappedIndex<'idx, IdxA> + Clone,
    B: MappedIndex<'idx, IdxB> + Clone,
    D: std::ops::Index<usize>,
{
    pub fn aggregate_over_dim<R, F, N>(&self, mut f: F) -> crate::data_frame::DataFrame<'idx, _, Vec<R>, _>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
        N: typenum::Unsigned,
    {
        if N::USIZE == 1 {
            // Aggregate over B (second dimension)
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
            crate::data_frame::DataFrame::new(a_index, result)
        } else if N::USIZE == 0 {
            // Aggregate over A (first dimension)
            let a_index = self.index.indices.0.clone();
            let b_index = self.index.indices.1.clone();
            let mut result = Vec::with_capacity(b_index.size());
            for b_val in b_index.iter() {
                let mut values = (0..a_index.size()).map(|a_i| {
                    let a_val = a_index.from_flat_index(a_i);
                    let idx = (a_val, b_val);
                    &self.data[self.index.to_flat_index(idx)]
                });
                result.push(f(&mut values));
            }
            crate::data_frame::DataFrame::new(b_index, result)
        } else {
            panic!("Only 2D supported for now");
        }
    }
} 