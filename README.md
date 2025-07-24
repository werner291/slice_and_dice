# Slice And Dice

A strongly-typed, modular data table library for Rust, inspired by data frames and enhanced with type-level safety.

- **Type-safe DataFrame**: Associate indices and data with compile-time guarantees using tag types.
- **Modular**: Organized into clear modules for DataFrame, index types, and tuple utilities.
- **Type-level programming**: Use traits and tag types to prevent mistakes and enable expressive, safe APIs.

## Example: Constructing a DataFrame from an Iterator

```rust
use slice_and_dice::data_frame::core::{DataFrameFromNumericExt, DataFrameFromSparseNumericExt};
use slice_and_dice::mapped_index::numeric_range_index::NumericRangeIndex;
use slice_and_dice::mapped_index::sparse_numeric_index::SparseNumericIndex;

// Tag type to mark the index dimension
derive(Debug)]
struct Row;

// Numeric index
let df = (0..3).to_numeric_dataframe::<Row>();
assert_eq!(df.index, NumericRangeIndex::<Row>::new(0, 3));
assert_eq!(df.data, vec![0, 1, 2]);

// Sparse numeric index
let df = [(10, "a"), (20, "b")]
    .into_iter()
    .to_sparse_numeric_dataframe::<Row>();
assert_eq!(df.index, SparseNumericIndex::<Row> { indices: vec![10, 20], _phantom: std::marker::PhantomData });
assert_eq!(df.data, vec!["a", "b"]);
```

## License

MIT No Attribution (MIT-0). See LICENSE for details.