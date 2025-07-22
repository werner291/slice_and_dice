use super::MappedIndex;
use std::marker::PhantomData;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[derive(Debug)]
pub struct SparseNumericIndex<T> {
    pub indices: Vec<i32>,
    pub _phantom: PhantomData<T>,
}

impl<T> PartialEq for SparseNumericIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}
impl<T> Eq for SparseNumericIndex<T> {}

impl<T> SparseNumericIndex<T> {
    /// Create a new SparseNumericIndex from a Vec. Panics if not sorted.
    pub fn new(indices: Vec<i32>) -> Self {
        if !indices.windows(2).all(|w| w[0] < w[1]) {
            panic!("SparseNumericIndex: indices must be strictly increasing");
        }
        Self { indices, _phantom: PhantomData }
    }
    /// Create from an iterator, validating order.
    pub fn from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
        let indices: Vec<i32> = iter.into_iter().collect();
        Self::new(indices)
    }
}

#[derive(Debug)]
pub struct SparseNumericValue<'idx, T> {
    pub value: &'idx i32,
    pub index: usize,
    _phantom: PhantomData<&'idx T>,
}

impl<'idx, T> Copy for SparseNumericValue<'idx, T> {}
impl<'idx, T> Clone for SparseNumericValue<'idx, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'idx, T> PartialEq for SparseNumericValue<'idx, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.index == other.index
    }
}
impl<'idx, T> Eq for SparseNumericValue<'idx, T> {}

impl<'idx, T: 'idx> MappedIndex<'idx, i32> for SparseNumericIndex<T> {
    type Value = SparseNumericValue<'idx, T>;
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        self.indices.iter().enumerate().map(move |(index, v)| SparseNumericValue { value: v, index, _phantom: PhantomData })
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        value.index
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        SparseNumericValue { value: &self.indices[index], index, _phantom: PhantomData }
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
        let idx = SparseNumericIndex::<()>::new(vec![1, 3, 7, 10]);
        assert_eq!(idx.indices, vec![1, 3, 7, 10]);
    }
    #[test]
    #[should_panic]
    fn test_sparse_numeric_index_invalid() {
        let _ = SparseNumericIndex::<()>::new(vec![1, 3, 2, 10]);
    }
    #[test]
    fn test_sparse_numeric_index_from_iter() {
        let idx = SparseNumericIndex::<()>::from_iter([2, 4, 6, 8]);
        assert_eq!(idx.indices, vec![2, 4, 6, 8]);
    }
    #[test]
    fn test_sparse_numeric_index_access() {
        let idx = SparseNumericIndex::<()>::new(vec![5, 10, 15]);
        let val = idx.from_flat_index(1);
        assert_eq!(*val.value, 10);
        assert_eq!(val.index, 1);
        let flat = idx.to_flat_index(val);
        assert_eq!(flat, 1);
    }
} 