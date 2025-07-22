use std::marker::PhantomData;
use super::MappedIndex;

/// A value in a numeric range index, representing a position in the range.
#[derive(Debug, PartialEq, Eq)]
pub struct NumericValue<'idx, Idx, T> {
    /// The numeric index value.
    pub index: Idx,
    _phantom: PhantomData<&'idx T>,
}

impl<'idx, Idx: Copy, T> Copy for NumericValue<'idx, Idx, T> {}
impl<'idx, Idx: Copy, T> Clone for NumericValue<'idx, Idx, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index representing a numeric range from `start` to `end` (exclusive).
#[derive(Debug, Clone)]
pub struct NumericRangeIndex<T> {
    /// The start of the range (inclusive).
    pub start: i32,
    /// The end of the range (exclusive).
    pub end: i32,
    pub _phantom: PhantomData<T>,
}

impl<T> PartialEq for NumericRangeIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl<T> Eq for NumericRangeIndex<T> {}

impl<'idx, T: 'idx> MappedIndex<'idx, i32> for NumericRangeIndex<T> {
    type Value = NumericValue<'idx, i32, T>;
    /// Returns an iterator over all numeric values in the range.
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        (self.start..self.end)
            .map(move |i| NumericValue { index: i, _phantom: PhantomData })
    }
    /// Returns the flat index for a numeric value (its position in the range).
    fn to_flat_index(&self, value: Self::Value) -> usize {
        value.index as usize
    }
    /// Returns the numeric value for a given flat index.
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        NumericValue { index: index as i32, _phantom: PhantomData }
    }
    /// Returns the number of values in the numeric range index.
    fn size(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl<T: 'static> std::ops::Index<i32> for NumericRangeIndex<T> {
    type Output = NumericValue<'static, i32, T>;
    fn index(&self, _index: i32) -> &Self::Output {
        panic!("Cannot return a reference to a value by index; use get() or at() instead.");
    }
}

impl<T> NumericRangeIndex<T> {
    /// Create a new NumericRangeIndex from start and end.
    pub fn new(start: i32, end: i32) -> Self {
        Self { start, end, _phantom: PhantomData }
    }
    pub fn at(&self, index: i32) -> NumericValue<'static, i32, T> {
        if index >= self.start && index < self.end {
            NumericValue { index, _phantom: PhantomData }
        } else {
            panic!("Index {:?} out of bounds ({:?}..{:?})", index, self.start, self.end);
        }
    }
}

impl<'idx, Idx, T> NumericValue<'idx, Idx, T> {
    /// Create a new NumericValue from an index.
    pub fn new(index: Idx) -> Self {
        Self { index, _phantom: PhantomData }
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

    #[test]
    fn test_flat_index_round_trip() {
        let _range: NumericRangeIndex<i32> = NumericRangeIndex::new(10, 20);
        let val = NumericValue::new(13);
        let flat = _range.to_flat_index(val);
        let round = _range.from_flat_index(flat);
        assert_eq!(val.index, round.index);
    }
} 