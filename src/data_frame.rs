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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::categorical_index::{CategoricalIndex, CategoricalValue};
    use crate::mapped_index::MappedIndex;
    use std::marker::PhantomData;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct Tag;

    #[test]
    fn test_dataframe_get_and_index() {
        let index = CategoricalIndex { values: vec!["a", "b", "c"], _phantom: PhantomData::<Tag> };
        let data = vec![10, 20, 30];
        let df = DataFrame { index, data, _phantom: PhantomData };
        let val = df.index.from_flat_index(1);
        assert_eq!(df.get(val), &20);
        assert_eq!(df[val], 20);
        assert_eq!(df.get_flat(2), &30);
    }

    #[test]
    fn test_dataframe_round_trip() {
        let index = CategoricalIndex { values: vec!["x", "y", "z"], _phantom: PhantomData::<Tag> };
        let data = vec![100, 200, 300];
        let df = DataFrame { index, data, _phantom: PhantomData };
        for flat in 0..df.index.size() {
            let val = df.index.from_flat_index(flat);
            let round = df.index.to_flat_index(val);
            assert_eq!(flat, round);
            assert_eq!(df.get(val), &df.get_flat(flat));
        }
    }
} 