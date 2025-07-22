use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use std::ops::Index;

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
pub struct DataFrame<I, D>
where
    I: MappedIndex,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
}

impl<I, D> DataFrame<I, D>
where
    I: MappedIndex,
    D: Index<usize>,
{
    /// Construct a new DataFrame from index and data.
    pub fn new(index: I, data: D) -> Self {
        Self { index, data }
    }

    /// Get a reference to the data for a given index value.
    pub fn get(&self, value: I::Value<'_>) -> &D::Output {
        &self.data[self.index.flatten_index_value(value)]
    }
    /// Get a reference to the data for a given flat index.
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }

    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack<'a, J, E, It, StackTag: 'static>(
        dfs: impl IntoIterator<Item = DataFrame<I, D>>,
    ) -> Option<DataFrame<CompoundIndex<(NumericRangeIndex<StackTag>, I)>, Vec<D::Output>>>
    where
        I: Clone + PartialEq,
        D::Output: Clone,
    {
        let dfs: Vec<DataFrame<I, D>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return None;
        }
        // Check all inner indices are equal
        let first_index = &dfs[0].index;
        for df in &dfs[1..] {
            // Compare by iterating over all values
            if first_index != &df.index {
                panic!("Indices mismatched.");
            }
        }

        // Build the compound index: outer is a numeric range, inner is the shared index
        let outer_index = NumericRangeIndex::new(0, dfs.len() as i32);

        let compound_index = CompoundIndex {
            indices: (outer_index, first_index.clone()),
        };
        // Flatten the data
        let mut data = Vec::new();
        for df in &dfs {
            for i in 0..df.index.size() {
                data.push(df.data[i].clone());
            }
        }
        Some(DataFrame::new(compound_index, data))
    }
}

impl<'idx, A, B, D> DataFrame<CompoundIndex<(A, B)>, D>
where
    A: MappedIndex + Clone,
    B: MappedIndex + Clone,
    D: Index<usize>,
{
    /// Aggregate over the dimension specified by typenum (U0 for first, U1 for second).
    pub fn aggregate_over_a<R, F, N>(&self, mut f: F) -> DataFrame<B, Vec<R>>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
        N: typenum::Unsigned,
    {
        // Aggregate over A (first dimension)
        let a_index = self.index.indices.0.clone();
        let b_index = self.index.indices.1.clone();
        let mut result = Vec::with_capacity(b_index.size());
        for b_val in b_index.iter() {
            let mut values = (0..a_index.size()).map(|a_i| {
                let a_val = a_index.unflatten_index_value(a_i);
                let idx = (a_val, b_val);
                &self.data[self.index.flatten_index_value(idx)]
            });
            result.push(f(&mut values));
        }
        DataFrame::new(b_index, result)
    }
}
