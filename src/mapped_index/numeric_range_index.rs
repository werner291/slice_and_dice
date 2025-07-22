use std::marker::PhantomData;
use super::MappedIndex;
use std::ops::Index;

#[derive(Debug, PartialEq, Eq)]
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
    pub _phantom: PhantomData<T>,
}

impl<'idx, T: 'idx> MappedIndex<'idx, i32> for NumericRangeIndex<T> {
    type Value = NumericValue<'idx, i32, T>;
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        (self.start..self.end)
            .map(move |i| NumericValue { index: i, _phantom: PhantomData })
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        value.index as usize
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        NumericValue { index: index as i32, _phantom: PhantomData }
    }
    fn size(&self) -> usize {
        (self.end - self.start) as usize
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

impl<'idx, T: 'idx> MappedIndex<'idx, i32> for &NumericRangeIndex<T> {
    type Value = <NumericRangeIndex<T> as MappedIndex<'idx, i32>>::Value;
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        (*self).iter()
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        (*self).to_flat_index(value)
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        (*self).from_flat_index(index)
    }
    fn size(&self) -> usize {
        (*self).size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::MappedIndex;

    struct Tag;

    #[test]
    fn test_flat_index_round_trip() {
        let range = NumericRangeIndex { start: 10, end: 20, _phantom: PhantomData::<Tag> };
        let val = range.from_flat_index(13);
        let flat = range.to_flat_index(val);
        let round = range.from_flat_index(flat);
        assert_eq!(val.index, round.index);
    }
} 