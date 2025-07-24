//! Stacking logic for DataFrame.
use super::core::DataFrame;
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use crate::mapped_index::MappedIndex;
use std::ops::Index;

impl<I, D> DataFrame<I, D>
where
    I: MappedIndex + Clone + PartialEq + 'static,
    D: Index<usize>,
    D::Output: Clone,
{
    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack<StackTag: 'static + std::fmt::Debug>(
        dfs: impl IntoIterator<Item = DataFrame<I, D>>,
    ) -> Option<DataFrame<CompoundIndex<(NumericRangeIndex<i32, StackTag>, I)>, Vec<D::Output>>> {
        let dfs: Vec<DataFrame<I, D>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return None;
        }
        // Check all inner indices are equal
        let first_index = &dfs[0].index;
        for df in &dfs[1..] {
            if first_index != &df.index {
                panic!("Indices mismatched.");
            }
        }
        let outer_index = NumericRangeIndex::new(0, dfs.len() as i32);
        let compound_index = CompoundIndex {
            indices: (outer_index, first_index.clone()),
        };
        let mut data = Vec::new();
        for df in &dfs {
            for i in 0..df.index.size() {
                data.push(df.data[i].clone());
            }
        }
        Some(DataFrame::new(compound_index, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range_index::NumericRangeIndex;

    #[derive(Debug)]
    struct Tag;
    #[derive(Debug)]
    struct StackTag;

    #[test]
    fn test_stack() {
        let index = NumericRangeIndex::<i32, Tag>::new(0, 2);
        let df1 = DataFrame::new(index.clone(), vec![10, 20]);
        let df2 = DataFrame::new(index.clone(), vec![30, 40]);
        let stacked = DataFrame::stack::<StackTag>(vec![df1, df2]).unwrap();

        assert_eq!(stacked.index.indices.0.size(), 2); // Outer index size
        assert_eq!(stacked.index.indices.1, index); // Inner index
        assert_eq!(stacked.data, vec![10, 20, 30, 40]); // Flattened data
    }
}
