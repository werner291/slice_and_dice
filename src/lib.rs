//! Top-level exports for core data structures and index types.

pub mod data_frame;
pub mod mapped_index;
pub mod tuple_utils;

/// The main DataFrame type for associating an index with data.
pub use crate::data_frame::core::DataFrame;
/// Numeric range index (contiguous, 0..N or arbitrary start..end).
pub use crate::mapped_index::numeric_range::{NumericRange, NumericRangeIndex};
/// Sparse numeric index (arbitrary, sorted i64 indices).
pub use crate::mapped_index::sparse_numeric_index::{SparseNumericIndex, SparseNumericValue};
