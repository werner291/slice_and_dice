use std::marker::PhantomData;
use super::MappedIndex;
use std::ops::Index;

pub struct NumericValue<'idx, Idx, T> {
    pub index: Idx,
    _phantom: PhantomData<&'idx T>,
}

impl<'idx, Idx: Copy, T> Copy for NumericValue<'idx, Idx, T> {}
impl<'idx, Idx: Copy, T> Clone for NumericValue<'idx, Idx, T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct NumericRangeIndex<T> {
    pub start: i32,
    pub end: i32,
    _phantom: PhantomData<T>,
}

impl<'idx, T: 'idx> MappedIndex<'idx, i32> for NumericRangeIndex<T> {
    type Value = NumericValue<'idx, i32, T>;
    fn get(&'idx self, index: i32) -> Option<Self::Value> {
        if index >= self.start && index < self.end {
            Some(NumericValue { index, _phantom: PhantomData })
        } else {
            None
        }
    }
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        (self.start..self.end)
            .map(move |i| NumericValue { index: i, _phantom: PhantomData })
    }
}

impl<T: 'static> std::ops::Index<i32> for NumericRangeIndex<T> {
    type Output = NumericValue<'static, i32, T>;
    fn index(&self, index: i32) -> &Self::Output {
        panic!("Cannot return a reference to a value by index; use get() or at() instead.");
    }
}

impl<T: 'static> NumericRangeIndex<T> {
    pub fn at(&self, index: i32) -> NumericValue<'static, i32, T> {
        if index >= self.start && index < self.end {
            NumericValue { index, _phantom: PhantomData }
        } else {
            panic!("Index {:?} out of bounds ({:?}..{:?})", index, self.start, self.end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::MappedIndex;

    struct Tag;

    #[test]
    fn test_in_range() {
        let range = NumericRangeIndex { start: 0, end: 10, _phantom: PhantomData::<Tag> };
        for i in 0..10 {
            let value = range.get(i);
            assert!(value.is_some(), "Expected Some for index {}", i);
            assert_eq!(value.unwrap().index, i);
        }
    }

    #[test]
    fn test_out_of_range() {
        let range = NumericRangeIndex { start: 0, end: 10, _phantom: PhantomData::<Tag> };
        assert!(range.get(-1).is_none(), "Expected None for index -1");
        assert!(range.get(10).is_none(), "Expected None for index 10");
        assert!(range.get(100).is_none(), "Expected None for index 100");
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let range = NumericRangeIndex { start: 0, end: 10, _phantom: PhantomData::<Tag> };
        let _ = range.at(100);
    }
    #[test]
    fn test_index_in_bounds() {
        let range = NumericRangeIndex { start: 0, end: 10, _phantom: PhantomData::<Tag> };
        let value = range.at(5);
        assert_eq!(value.index, 5);
    }
} 