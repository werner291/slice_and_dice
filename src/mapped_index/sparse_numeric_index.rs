use super::VariableRange;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseNumericIndex<I> {
    pub indices: Vec<I>,
}

impl<I: PartialOrd + Copy> SparseNumericIndex<I> {
    /// Create a new SparseNumericIndex from a Vec. Panics if not sorted.
    pub fn new(indices: Vec<I>) -> Self {
        if !indices.windows(2).all(|w| w[0] < w[1]) {
            panic!("SparseNumericIndex: indices must be strictly increasing");
        }
        Self { indices }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SparseNumericValue<I> {
    pub value: I,
    pub index: usize,
}

impl<I: Copy> SparseNumericValue<I> {
    /// Create a new SparseNumericValue with the given value and index.
    pub const fn new(value: I, index: usize) -> Self {
        Self { value, index }
    }
}

impl<I: Copy + 'static> VariableRange for SparseNumericIndex<I> {
    type Value<'a> = SparseNumericValue<I>;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices
            .iter()
            .enumerate()
            .map(move |(index, v)| SparseNumericValue::new(*v, index))
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        SparseNumericValue::new(self.indices[index], index)
    }

    fn size(&self) -> usize {
        self.indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sparse_numeric_index_valid() {
        let idx = SparseNumericIndex::<i64>::new(vec![1, 3, 7, 10]);
        assert_eq!(idx.indices, vec![1, 3, 7, 10]);
    }

    #[test]
    #[should_panic]
    fn test_sparse_numeric_index_invalid() {
        let _ = SparseNumericIndex::<i64>::new(vec![1, 3, 2, 10]);
    }

    #[test]
    fn test_sparse_numeric_index_access() {
        let idx = SparseNumericIndex::<i64>::new(vec![5, 10, 15]);
        let val = idx.unflatten_index_value(1);
        assert_eq!(val.value, 10);
        assert_eq!(val.index, 1);
        // For SparseNumericValue, the flat index is stored in the value.index field
        assert_eq!(val.index, 1);
    }
}
