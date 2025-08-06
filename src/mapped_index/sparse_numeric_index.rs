use super::VariableRange;
use std::marker::PhantomData;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SparseNumericIndex<I, T> {
    pub indices: Vec<I>,
    pub _phantom: PhantomData<T>,
}

impl<I: Clone, T> Clone for SparseNumericIndex<I, T> {
    fn clone(&self) -> Self {
        Self {
            indices: self.indices.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I: PartialEq, T> PartialEq for SparseNumericIndex<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}
impl<I: Eq, T> Eq for SparseNumericIndex<I, T> {}

impl<I: PartialOrd + Copy, T> SparseNumericIndex<I, T> {
    /// Create a new SparseNumericIndex from a Vec. Panics if not sorted.
    pub fn new(indices: Vec<I>) -> Self {
        if !indices.windows(2).all(|w| w[0] < w[1]) {
            panic!("SparseNumericIndex: indices must be strictly increasing");
        }
        Self {
            indices,
            _phantom: PhantomData,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SparseNumericValue<I, T> {
    pub value: I,
    pub index: usize,
    _phantom: PhantomData<T>,
}

impl<I: Copy, T> Copy for SparseNumericValue<I, T> {}
impl<I: Copy, T> Clone for SparseNumericValue<I, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: Copy, T> SparseNumericValue<I, T> {
    /// Create a new SparseNumericValue with the given value and index.
    pub const fn new(value: I, index: usize) -> Self {
        Self {
            value,
            index,
            _phantom: PhantomData,
        }
    }
}

impl<I: PartialEq, T> PartialEq for SparseNumericValue<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.index == other.index
    }
}
impl<I: Eq, T> Eq for SparseNumericValue<I, T> {}

impl<I: Copy + 'static, T: 'static> VariableRange for SparseNumericIndex<I, T> {
    type Value<'a> = SparseNumericValue<I, T>;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices
            .iter()
            .enumerate()
            .map(move |(index, v)| SparseNumericValue::new(*v, index))
    }

    fn flatten_index_value(&self, value: Self::Value<'_>) -> usize {
        value.index
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
        let idx = SparseNumericIndex::<i64, ()>::new(vec![1, 3, 7, 10]);
        assert_eq!(idx.indices, vec![1, 3, 7, 10]);
    }

    #[test]
    #[should_panic]
    fn test_sparse_numeric_index_invalid() {
        let _ = SparseNumericIndex::<i64, ()>::new(vec![1, 3, 2, 10]);
    }

    #[test]
    fn test_sparse_numeric_index_access() {
        let idx = SparseNumericIndex::<i64, ()>::new(vec![5, 10, 15]);
        let val = idx.unflatten_index_value(1);
        assert_eq!(val.value, 10);
        assert_eq!(val.index, 1);
        let flat = idx.flatten_index_value(val);
        assert_eq!(flat, 1);
    }
}
