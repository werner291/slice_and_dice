//! Aggregation logic for DataFrame, split from the main file for maintainability.
//! Projection traits are now in projection.rs.
use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::CompoundIndex;
use typenum::Unsigned;
use crate::data_frame::projection::ProjectOut;

// N-dimensional aggregation for CompoundIndex
impl<'idx, Indices, D, IdxTuple, N> crate::data_frame::DataFrame<'idx, CompoundIndex<Indices>, D, IdxTuple>
where
    CompoundIndex<Indices>: MappedIndex<'idx, IdxTuple> + ProjectOut<N>,
    D: std::ops::Index<usize>,
    N: Unsigned,
    <CompoundIndex<Indices> as ProjectOut<N>>::Remaining: Clone,
{
    pub fn aggregate_over_dim_nd<R, F>(&self, mut f: F) -> crate::data_frame::DataFrame<'idx, <CompoundIndex<Indices> as ProjectOut<N>>::Remaining, Vec<R>, _>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
    {
        // Use ProjectOut to project out the N-th dimension from the index
        let mut projected_index = self.index.clone();
        let (_, projected_index) = projected_index.project_out();
        let projected_index = projected_index;
        let mut result = Vec::with_capacity(projected_index.size());
        for i in 0..projected_index.size() {
            let proj_val = projected_index.from_flat_index(i);
            // For each value in the projected index, collect all values in the original index that match
            let mut values = (0..self.index.size()).filter_map(|flat| {
                let full_val = self.index.from_flat_index(flat);
                // Project out the N-th element from full_val and compare to proj_val
                let (_, remaining) = ProjectOut::<N>::project_out(full_val);
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