use super::VariableRange;
use sorted_vec::SortedSet;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SparseNumericIndex<I: Ord> {
    pub indices: SortedSet<I>,
}

impl<I: Ord + Copy> SparseNumericIndex<I> {
    /// Create a new SparseNumericIndex from a sorted set of indices.
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::SparseNumericIndex;
    /// use slice_and_dice::mapped_index::VariableRange;
    /// use sorted_vec::SortedSet;
    /// let idx = SparseNumericIndex::new(SortedSet::from(vec![1_i64, 3, 5]));
    /// assert_eq!(idx.size(), 3);
    /// ```
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
