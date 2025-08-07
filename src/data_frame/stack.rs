//! Stacking logic for DataFrame.
use super::core::DataFrame;
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range::NumericRangeIndex;
use crate::mapped_index::sparse_numeric_index::SparseNumericIndex;
use frunk::{HList, hlist};
use sorted_vec::SortedSet;
use std::cmp::Ordering;
use std::ops::Index;

/// Interpolation method for missing data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMethod {
    /// Use the nearest value.
    Nearest,
    /// Use the previous value.
    Previous,
    /// Use the next value.
    Next,
    /// Use a default value.
    Default,
}

/// Extrapolation method for missing data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtrapolationMethod {
    /// Use the nearest value.
    Nearest,
    /// Use a default value.
    Default,
}

impl<I, D> DataFrame<I, D>
where
    I: VariableRange + Clone + PartialEq + 'static,
    D: Index<usize>,
    D::Output: Clone,
{
    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack(
        dfs: impl IntoIterator<Item = DataFrame<I, D>>,
    ) -> Option<DataFrame<CompoundIndex<HList![NumericRangeIndex<usize>, I]>, Vec<D::Output>>> {
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
        let outer_index = NumericRangeIndex::new(0, dfs.len());
        let compound_index = CompoundIndex {
            indices: hlist![outer_index, first_index.clone()],
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

impl<I, D> DataFrame<SparseNumericIndex<I>, D>
where
    I: Copy + PartialOrd + Ord + 'static,
    D: Index<usize>,
    D::Output: Clone + Default,
{
    /// Stack an iterator of DataFrames with potentially mismatching SparseNumericIndex into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is a union of all indices from the original DataFrames.
    /// Missing values are handled according to the specified interpolation and extrapolation methods.
    ///
    /// # Arguments
    ///
    /// * `dfs` - An iterator of DataFrames to stack
    /// * `interpolation` - The method to use for interpolating missing values
    /// * `extrapolation` - The method to use for extrapolating missing values
    /// * `default_value` - The default value to use when interpolation or extrapolation method is Default
    pub fn stack_sparse(
        dfs: impl IntoIterator<Item = DataFrame<SparseNumericIndex<I>, D>>,
        interpolation: InterpolationMethod,
        extrapolation: ExtrapolationMethod,
        default_value: D::Output,
    ) -> Option<
        DataFrame<
            CompoundIndex<HList![NumericRangeIndex<usize>, SparseNumericIndex<I>]>,
            Vec<D::Output>,
        >,
    > {
        let dfs: Vec<DataFrame<SparseNumericIndex<I>, D>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return None;
        }

        // Create a union of all indices
        let mut all_indices = SortedSet::new();
        for df in &dfs {
            all_indices.extend(df.index.indices.iter().copied());
        }

        // Create a new index from the union
        let union_index = SparseNumericIndex::new(all_indices);

        // Create the outer index
        let outer_index = NumericRangeIndex::new(0, dfs.len());
        let compound_index = CompoundIndex {
            indices: hlist![outer_index, union_index.clone()],
        };

        // Fill in the data, handling missing values
        let mut data = Vec::new();
        for df in &dfs {
            let df_indices = &df.index.indices;

            for &union_idx in &union_index.indices {
                // Check if this index exists in the current DataFrame
                match df_indices.binary_search(&union_idx) {
                    Ok(pos) => {
                        // Index exists, use the actual value
                        data.push(df.data[pos].clone());
                    }
                    Err(insert_pos) => {
                        // Index doesn't exist, interpolate or extrapolate
                        let value = if insert_pos == 0 || insert_pos == df_indices.len() {
                            // Extrapolation case
                            match extrapolation {
                                ExtrapolationMethod::Nearest => {
                                    if df_indices.is_empty() {
                                        // If the DataFrame has no indices, use the default value
                                        default_value.clone()
                                    } else if insert_pos == 0 {
                                        // Use the first value
                                        df.data[0].clone()
                                    } else {
                                        // Use the last value
                                        df.data[df_indices.len() - 1].clone()
                                    }
                                }
                                ExtrapolationMethod::Default => default_value.clone(),
                            }
                        } else {
                            // Interpolation case
                            match interpolation {
                                InterpolationMethod::Nearest => {
                                    // Find the nearest index
                                    let prev_idx = df_indices[insert_pos - 1];
                                    let next_idx = df_indices[insert_pos];

                                    // Compare distances without casting
                                    let prev_dist = match union_idx.cmp(&prev_idx) {
                                        Ordering::Less => prev_idx.cmp(&union_idx),
                                        _ => union_idx.cmp(&prev_idx),
                                    };

                                    let next_dist = match union_idx.cmp(&next_idx) {
                                        Ordering::Less => next_idx.cmp(&union_idx),
                                        _ => union_idx.cmp(&next_idx),
                                    };

                                    match prev_dist.cmp(&next_dist) {
                                        Ordering::Less | Ordering::Equal => {
                                            df.data[insert_pos - 1].clone()
                                        }
                                        Ordering::Greater => df.data[insert_pos].clone(),
                                    }
                                }
                                InterpolationMethod::Previous => {
                                    // Use the previous value
                                    df.data[insert_pos - 1].clone()
                                }
                                InterpolationMethod::Next => {
                                    // Use the next value
                                    df.data[insert_pos].clone()
                                }
                                InterpolationMethod::Default => default_value.clone(),
                            }
                        };
                        data.push(value);
                    }
                }
            }
        }

        Some(DataFrame::new(compound_index, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use crate::mapped_index::sparse_numeric_index::SparseNumericIndex;

    #[test]
    fn test_stack() {
        let index = NumericRangeIndex::<i32>::new(0, 2);
        let df1 = DataFrame::new(index.clone(), vec![10, 20]);
        let df2 = DataFrame::new(index.clone(), vec![30, 40]);
        let stacked = DataFrame::stack(vec![df1, df2]).unwrap();

        assert_eq!(stacked.index.indices.head.size(), 2); // Outer index size
        assert_eq!(stacked.index.indices.tail.head, index); // Inner index
        assert_eq!(stacked.data, vec![10, 20, 30, 40]); // Flattened data
    }

    #[test]
    fn test_stack_sparse_matching() {
        // Test with matching indices
        let index1 = SparseNumericIndex::<i32>::new(vec![1, 3, 5].into());
        let df1 = DataFrame::new(index1.clone(), vec![10, 30, 50]);
        let df2 = DataFrame::new(index1.clone(), vec![100, 300, 500]);

        let stacked = DataFrame::stack_sparse(
            vec![df1, df2],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        assert_eq!(stacked.index.indices.head.size(), 2); // Outer index size
        assert_eq!(
            stacked.index.indices.tail.head.indices,
            vec![1, 3, 5].into()
        ); // Inner index
        assert_eq!(stacked.data, vec![10, 30, 50, 100, 300, 500]); // Flattened data
    }

    #[test]
    fn test_stack_sparse_mismatching() {
        // Test with mismatching indices
        let index1 = SparseNumericIndex::<i32>::new(vec![1, 3, 5].into());
        let index2 = SparseNumericIndex::<i32>::new(vec![2, 3, 6].into());
        let df1 = DataFrame::new(index1, vec![10, 30, 50]);
        let df2 = DataFrame::new(index2, vec![20, 30, 60]);

        let stacked = DataFrame::stack_sparse(
            vec![df1, df2],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        // Union of indices should be [1, 2, 3, 5, 6]
        assert_eq!(stacked.index.indices.head.size(), 2); // Outer index size
        assert_eq!(
            stacked.index.indices.tail.head.indices,
            vec![1, 2, 3, 5, 6].into()
        ); // Inner index

        // First DataFrame: [10, ?, 30, 50, ?] where ? are interpolated/extrapolated
        // Second DataFrame: [?, 20, 30, ?, 60] where ? are interpolated/extrapolated
        // With Nearest interpolation/extrapolation:
        // First DataFrame: [10, 10, 30, 50, 50]
        // Second DataFrame: [20, 20, 30, 30, 60]
        assert_eq!(stacked.data, vec![10, 10, 30, 50, 50, 20, 20, 30, 30, 60]);
    }

    #[test]
    fn test_stack_sparse_interpolation_methods() {
        // Test different interpolation methods
        let index1 = SparseNumericIndex::<i32>::new(vec![1, 5].into());
        let index2 = SparseNumericIndex::<i32>::new(vec![1, 5].into());
        let df1 = DataFrame::new(index1, vec![10, 50]);
        let df2 = DataFrame::new(index2, vec![100, 500]);

        // Create a third DataFrame with the index 3 to ensure it's in the union
        let index3 = SparseNumericIndex::<i32>::new(vec![1, 3, 5].into());
        let df3 = DataFrame::new(index3, vec![1000, 3000, 5000]);

        // Test Previous interpolation
        let stacked_prev = DataFrame::stack_sparse(
            vec![df1.clone(), df2.clone(), df3.clone()],
            InterpolationMethod::Previous,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        assert_eq!(
            stacked_prev.index.indices.tail.head.indices,
            vec![1, 3, 5].into()
        ); // Inner index
        // First DataFrame: [10, 10, 50] (value at 3 is from previous at 1)
        // Second DataFrame: [100, 100, 500] (value at 3 is from previous at 1)
        // Third DataFrame: [1000, 3000, 5000] (actual values)
        assert_eq!(
            stacked_prev.data,
            vec![10, 10, 50, 100, 100, 500, 1000, 3000, 5000]
        );

        // Test Next interpolation
        let stacked_next = DataFrame::stack_sparse(
            vec![df1.clone(), df2.clone(), df3.clone()],
            InterpolationMethod::Next,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        assert_eq!(
            stacked_next.index.indices.tail.head.indices,
            vec![1, 3, 5].into()
        ); // Inner index
        // First DataFrame: [10, 50, 50] (value at 3 is from next at 5)
        // Second DataFrame: [100, 500, 500] (value at 3 is from next at 5)
        // Third DataFrame: [1000, 3000, 5000] (actual values)
        assert_eq!(
            stacked_next.data,
            vec![10, 50, 50, 100, 500, 500, 1000, 3000, 5000]
        );

        // Test Default interpolation
        let stacked_default = DataFrame::stack_sparse(
            vec![df1, df2, df3],
            InterpolationMethod::Default,
            ExtrapolationMethod::Default,
            999,
        )
        .unwrap();

        assert_eq!(
            stacked_default.index.indices.tail.head.indices,
            vec![1, 3, 5].into()
        ); // Inner index
        // First DataFrame: [10, 999, 50] (value at 3 is default)
        // Second DataFrame: [100, 999, 500] (value at 3 is default)
        // Third DataFrame: [1000, 3000, 5000] (actual values)
        assert_eq!(
            stacked_default.data,
            vec![10, 999, 50, 100, 999, 500, 1000, 3000, 5000]
        );
    }

    #[test]
    fn test_stack_sparse_large_matching() {
        // Test with large matching indices (at least 10 elements each)
        let indices1 = vec![1, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55];
        let indices2 = indices1.clone();

        let index1 = SparseNumericIndex::<i32>::new(indices1.into());
        let index2 = SparseNumericIndex::<i32>::new(indices2.into());

        let data1: Vec<i32> = (0..12).map(|i| i * 10).collect();
        let data2: Vec<i32> = (0..12).map(|i| i * 100).collect();

        let df1 = DataFrame::new(index1.clone(), data1);
        let df2 = DataFrame::new(index2.clone(), data2);

        let stacked = DataFrame::stack_sparse(
            vec![df1, df2],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        assert_eq!(stacked.index.indices.head.size(), 2); // Outer index size
        assert_eq!(stacked.index.indices.tail.head.indices.len(), 12); // Inner index size
        assert_eq!(
            stacked.index.indices.tail.head.indices,
            vec![1, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55].into()
        ); // Inner index

        // Expected data: all values from df1 followed by all values from df2
        let expected_data: Vec<i32> = vec![
            0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, // df1 values
            0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, // df2 values
        ];
        assert_eq!(stacked.data, expected_data);
    }

    #[test]
    fn test_stack_sparse_large_mismatching() {
        // Test with large mismatching indices (at least 10 elements each)
        // First DataFrame has even indices
        let indices1: Vec<i32> = (0..12).map(|i| i * 2).collect();
        // Second DataFrame has odd indices
        let indices2: Vec<i32> = (0..12).map(|i| i * 2 + 1).collect();

        let index1 = SparseNumericIndex::<i32>::new(indices1.into());
        let index2 = SparseNumericIndex::<i32>::new(indices2.into());

        let data1: Vec<i32> = (0..12).map(|i| i * 10).collect();
        let data2: Vec<i32> = (0..12).map(|i| i * 100).collect();

        let df1 = DataFrame::new(index1, data1);
        let df2 = DataFrame::new(index2, data2);

        let stacked = DataFrame::stack_sparse(
            vec![df1, df2],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Nearest,
            0,
        )
        .unwrap();

        assert_eq!(stacked.index.indices.head.size(), 2); // Outer index size
        assert_eq!(stacked.index.indices.tail.head.indices.len(), 24); // Inner index size (union of both indices)

        // Union of indices should be [0, 1, 2, 3, ..., 22, 23]
        let expected_indices: Vec<i32> = (0..24).collect();
        assert_eq!(
            stacked.index.indices.tail.head.indices,
            expected_indices.into()
        );

        // Verify data length
        assert_eq!(stacked.data.len(), 48); // 24 indices * 2 DataFrames
    }

    #[test]
    fn test_stack_sparse_empty_dataframe() {
        // Test with an empty DataFrame
        let empty_indices: Vec<i32> = vec![];
        let non_empty_indices: Vec<i32> = vec![1, 2, 3];

        let empty_index = SparseNumericIndex::<i32>::new(empty_indices.into());
        let non_empty_index = SparseNumericIndex::<i32>::new(non_empty_indices.into());

        let empty_data: Vec<i32> = vec![];
        let non_empty_data: Vec<i32> = vec![10, 20, 30];

        let empty_df = DataFrame::new(empty_index, empty_data);
        let non_empty_df = DataFrame::new(non_empty_index, non_empty_data);

        // This should not panic
        let stacked = DataFrame::stack_sparse(
            vec![empty_df.clone(), non_empty_df.clone()],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Default, // Use Default to avoid the panic
            -999,
        )
        .unwrap();

        // The union of indices should be [1, 2, 3]
        assert_eq!(
            stacked.index.indices.tail.head.indices,
            vec![1, 2, 3].into()
        );

        // First DataFrame should use default values for all indices
        // Second DataFrame should use actual values
        assert_eq!(stacked.data, vec![-999, -999, -999, 10, 20, 30]);

        // This would panic without a fix because it tries to use Nearest extrapolation
        // on an empty DataFrame, but with our fix it should work correctly
        let stacked_nearest = DataFrame::stack_sparse(
            vec![empty_df, non_empty_df],
            InterpolationMethod::Nearest,
            ExtrapolationMethod::Nearest,
            -999,
        )
        .unwrap();

        // The union of indices should be [1, 2, 3]
        assert_eq!(
            stacked_nearest.index.indices.tail.head.indices,
            vec![1, 2, 3].into()
        );

        // First DataFrame should use default values for all indices (since it's empty)
        // Second DataFrame should use actual values
        assert_eq!(stacked_nearest.data, vec![-999, -999, -999, 10, 20, 30]);
    }

    #[test]
    fn test_stack_sparse_large_interpolation() {
        // Test different interpolation methods with large indices
        // Create sparse indices with gaps
        let indices1: Vec<i32> = vec![0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50];
        let indices2: Vec<i32> = vec![0, 10, 20, 30, 40, 50];
        let indices3: Vec<i32> = vec![0, 2, 4, 6, 8, 12, 16, 24, 32, 48];

        let index1 = SparseNumericIndex::<i32>::new(indices1.into());
        let index2 = SparseNumericIndex::<i32>::new(indices2.into());
        let index3 = SparseNumericIndex::<i32>::new(indices3.into());

        let data1: Vec<i32> = (0..11).map(|i| i * 10).collect();
        let data2: Vec<i32> = (0..6).map(|i| i * 100).collect();
        let data3: Vec<i32> = (0..10).map(|i| i * 1000).collect();

        let df1 = DataFrame::new(index1, data1);
        let df2 = DataFrame::new(index2, data2);
        let df3 = DataFrame::new(index3, data3);

        // Test Previous interpolation
        let stacked_prev = DataFrame::stack_sparse(
            vec![df1.clone(), df2.clone(), df3.clone()],
            InterpolationMethod::Previous,
            ExtrapolationMethod::Nearest,
            -999,
        )
        .unwrap();

        // The union of all indices should have at least 20 elements
        assert!(stacked_prev.index.indices.tail.head.indices.len() >= 20);

        // Test Next interpolation
        let stacked_next = DataFrame::stack_sparse(
            vec![df1.clone(), df2.clone(), df3.clone()],
            InterpolationMethod::Next,
            ExtrapolationMethod::Nearest,
            -999,
        )
        .unwrap();

        assert_eq!(
            stacked_next.index.indices.tail.head.indices,
            stacked_prev.index.indices.tail.head.indices
        );

        // Test Default interpolation and extrapolation
        let stacked_default = DataFrame::stack_sparse(
            vec![df1, df2, df3],
            InterpolationMethod::Default,
            ExtrapolationMethod::Default,
            -999,
        )
        .unwrap();

        assert_eq!(
            stacked_default.index.indices.tail.head.indices,
            stacked_prev.index.indices.tail.head.indices
        );

        // Verify that the default value appears in the data
        assert!(stacked_default.data.contains(&-999));
    }
}
