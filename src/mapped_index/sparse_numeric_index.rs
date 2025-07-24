use super::MappedIndex;
use std::marker::PhantomData;

/// A sparse numeric index, holding a sorted Vec of i32 indices.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SparseNumericIndex<T> {
    pub indices: Vec<i64>,
    pub _phantom: PhantomData<T>,
}

impl<T> Clone for SparseNumericIndex<T> {
    fn clone(&self) -> Self {
        Self {
            indices: self.indices.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for SparseNumericIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}
impl<T> Eq for SparseNumericIndex<T> {}

impl<T> SparseNumericIndex<T> {
    /// Create a new SparseNumericIndex from a Vec. Panics if not sorted.
    pub fn new(indices: Vec<i64>) -> Self {
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
pub struct SparseNumericValue<T> {
    pub value: i64,
    pub index: usize,
    _phantom: PhantomData<T>,
}

impl<T> Copy for SparseNumericValue<T> {}
impl<T> Clone for SparseNumericValue<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            index: self.index,
            _phantom: Default::default(),
        }
    }
}

impl<T> PartialEq for SparseNumericValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.index == other.index
    }
}
impl<T> Eq for SparseNumericValue<T> {}

impl<T: 'static> MappedIndex for SparseNumericIndex<T> {
    type Value<'a> = SparseNumericValue<T>;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices
            .iter()
            .enumerate()
            .map(move |(index, v)| SparseNumericValue {
                value: *v,
                index,
                _phantom: PhantomData,
            })
    }

    fn flatten_index_value(&self, value: Self::Value<'_>) -> usize {
        value.index
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        SparseNumericValue {
            value: self.indices[index],
            index,
            _phantom: PhantomData,
        }
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
    fn test_sparse_numeric_index_access() {
        let idx = SparseNumericIndex::<()>::new(vec![5, 10, 15]);
        let val = idx.unflatten_index_value(1);
        assert_eq!(val.value, 10);
        assert_eq!(val.index, 1);
        let flat = idx.flatten_index_value(val);
        assert_eq!(flat, 1);
    }
}
