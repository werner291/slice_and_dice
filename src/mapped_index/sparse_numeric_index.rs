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

impl<I: Copy + 'static> VariableRange for SparseNumericIndex<I> {
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
}
