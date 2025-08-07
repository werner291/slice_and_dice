use super::VariableRange;
use sorted_vec::SortedSet;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseNumericIndex<I: Ord> {
    pub indices: SortedSet<I>,
}

impl<I: Ord + Copy> SparseNumericIndex<I> {
    /// Create a new SparseNumericIndex from a Vec. Panics if not sorted.
    pub fn new(indices: SortedSet<I>) -> Self {
        Self { indices }
    }
}

impl<I: Copy + 'static + Ord + Sync> VariableRange for SparseNumericIndex<I> {
    type Value<'a> = I;

    fn iter(&self) -> impl Iterator<Item = I> + Clone {
        self.indices.iter().copied()
    }

    fn unflatten_index_value(&self, index: usize) -> I {
        self.indices[index]
    }

    fn size(&self) -> usize {
        self.indices.len()
    }
}
