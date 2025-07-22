use crate::mapped_index::MappedIndex;
use std::marker::PhantomData;
use std::ops::Index;

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
pub struct DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
    _phantom: PhantomData<&'idx Idx>,
}

impl<'idx, I, D, Idx> DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// Get a reference to the data for a given index value.
    pub fn get(&'idx self, value: I::Value) -> &D::Output {
        &self.data[self.index.to_flat_index(value)]
    }
    /// Get a reference to the data for a given flat index.
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }
}

impl<'idx, I, D, Idx> Index<I::Value> for DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    type Output = D::Output;
    fn index(&self, value: I::Value) -> &Self::Output {
        &self.data[self.index.to_flat_index(value)]
    }
} 