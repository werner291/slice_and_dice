use super::MappedIndex;
use std::marker::PhantomData;

/// A value in a numeric range index, representing a position in the range.
#[derive(Debug)]
pub struct NumericValue<Idx, T> {
    /// The numeric index value.
    pub index: Idx,
    _phantom: PhantomData<T>,
}

impl<Idx: PartialEq, T> PartialEq for NumericValue<Idx, T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<Idx: Eq, T> Eq for NumericValue<Idx, T> {}

impl<Idx: Copy, T> Copy for NumericValue<Idx, T> {}
impl<Idx: Copy, T> Clone for NumericValue<Idx, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index representing a numeric range from `start` to `end` (exclusive).
#[derive(Debug)]
pub struct NumericRangeIndex<T> {
    /// The start of the range (inclusive).
    pub start: i32, // TODO: make this type not hardcoded
    /// The end of the range (exclusive).
    pub end: i32,
    pub _phantom: PhantomData<T>,
}

impl<T> Clone for NumericRangeIndex<T> {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for NumericRangeIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl<T> Eq for NumericRangeIndex<T> {}

impl<T: 'static> MappedIndex for NumericRangeIndex<T> {
    type Value<'a> = NumericValue<i32, T>;

    /// Returns an iterator over all numeric values in the range.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        (self.start..self.end).map(move |i| NumericValue {
            index: i,
            _phantom: PhantomData,
        })
    }

    /// Returns the flat index for a numeric value (its position in the range).
    fn flatten_index_value(&self, value: Self::Value<'_>) -> usize {
        value.index as usize
    }

    /// Returns the numeric value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        NumericValue {
            index: index as i32,
            _phantom: PhantomData,
        }
    }

    /// Returns the number of values in the numeric range index.
    fn size(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl<T: 'static> std::ops::Index<i32> for NumericRangeIndex<T> {
    type Output = NumericValue<i32, T>;
    fn index(&self, _index: i32) -> &Self::Output {
        panic!("Cannot return a reference to a value by index; use get() or at() instead.");
    }
}

impl<T> NumericRangeIndex<T> {
    /// Create a new NumericRangeIndex from start and end.
    pub fn new(start: i32, end: i32) -> Self {
        Self {
            start,
            end,
            _phantom: PhantomData,
        }
    }
    pub fn at(&self, index: i32) -> NumericValue<i32, T> {
        if index >= self.start && index < self.end {
            NumericValue {
                index,
                _phantom: PhantomData,
            }
        } else {
            panic!(
                "Index {:?} out of bounds ({:?}..{:?})",
                index, self.start, self.end
            );
        }
    }
}

impl<'idx, Idx, T> NumericValue<Idx, T> {
    /// Create a new NumericValue from an index.
    pub fn new(index: Idx) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
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
        let flat = _range.flatten_index_value(val);
        let round = _range.unflatten_index_value(flat);
        assert_eq!(val.index, round.index);
    }
}
