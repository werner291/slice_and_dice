use super::MappedIndex;
use std::marker::PhantomData;

/// A value in a numeric range index, representing a position in the range.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct NumericValue<I: std::fmt::Debug, T> {
    /// The numeric index value.
    pub index: I,
    _phantom: PhantomData<T>,
}

impl<I: PartialEq + std::fmt::Debug, T> PartialEq for NumericValue<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<I: Eq + std::fmt::Debug, T> Eq for NumericValue<I, T> {}

impl<I: Copy + std::fmt::Debug, T> Copy for NumericValue<I, T> {}
impl<I: Copy + std::fmt::Debug, T> Clone for NumericValue<I, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index representing a numeric range from `start` to `end` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct NumericRangeIndex<I: std::fmt::Debug, T> {
    /// The start of the range (inclusive).
    pub start: I,
    /// The end of the range (exclusive).
    pub end: I,
    pub _phantom: PhantomData<T>,
}

impl<I: Clone + std::fmt::Debug, T> Clone for NumericRangeIndex<I, T> {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            end: self.end.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I: PartialEq + std::fmt::Debug, T> PartialEq for NumericRangeIndex<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl<I: Eq + std::fmt::Debug, T> Eq for NumericRangeIndex<I, T> {}

impl<I, T> NumericRangeIndex<I, T>
where
    I: Copy
        + PartialOrd
        + std::ops::Add<Output = I>
        + std::ops::Sub<Output = I>
        + std::fmt::Debug
        + TryFrom<usize>
        + TryInto<usize>
        + 'static,
    <I as TryFrom<usize>>::Error: std::fmt::Debug,
    <I as TryInto<usize>>::Error: std::fmt::Debug,
{
    /// Create a new NumericRangeIndex from start and end.
    pub fn new(start: I, end: I) -> Self {
        Self {
            start,
            end,
            _phantom: PhantomData,
        }
    }
    pub fn at(&self, index: I) -> NumericValue<I, T> {
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

impl<I, T> MappedIndex for NumericRangeIndex<I, T>
where
    I: Copy
        + PartialOrd
        + std::ops::Add<Output = I>
        + std::ops::Sub<Output = I>
        + std::fmt::Debug
        + TryFrom<usize>
        + TryInto<usize>
        + 'static,
    <I as TryFrom<usize>>::Error: std::fmt::Debug,
    <I as TryInto<usize>>::Error: std::fmt::Debug,
    T: 'static,
{
    type Value<'a> = NumericValue<I, T>;

    /// Returns an iterator over all numeric values in the range.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let start = self.start;
        let end = self.end;
        let start_usize: usize = start.try_into().unwrap();
        let end_usize: usize = end.try_into().unwrap();
        (start_usize..end_usize).map(move |i| NumericValue {
            index: I::try_from(i).unwrap_or_else(|e| panic!("Failed to convert: {:?}", e)),
            _phantom: PhantomData,
        })
    }

    /// Returns the flat index for a numeric value (its position in the range).
    fn flatten_index_value(&self, value: Self::Value<'_>) -> usize {
        let idx: usize = value.index.try_into().unwrap();
        let start: usize = self.start.try_into().unwrap();
        idx - start
    }

    /// Returns the numeric value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        NumericValue {
            index: self.start
                + I::try_from(index).unwrap_or_else(|e| panic!("Failed to convert: {:?}", e)),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of values in the numeric range index.
    fn size(&self) -> usize {
        let start: usize = self.start.try_into().unwrap();
        let end: usize = self.end.try_into().unwrap();
        end - start
    }

    fn min<'a>(&'a self) -> Option<Self::Value<'a>>
    where
        Self::Value<'a>: Ord,
    {
        self.iter().min()
    }

    fn max<'a>(&'a self) -> Option<Self::Value<'a>>
    where
        Self::Value<'a>: Ord,
    {
        self.iter().max()
    }
}

impl<I: Copy + std::fmt::Debug + 'static, T: 'static> std::ops::Index<I>
    for NumericRangeIndex<I, T>
{
    type Output = NumericValue<I, T>;
    fn index(&self, _index: I) -> &Self::Output {
        panic!("Cannot return a reference to a value by index; use get() or at() instead.");
    }
}

impl<'idx, I: std::fmt::Debug, T> NumericValue<I, T> {
    /// Create a new NumericValue from an index.
    pub fn new(index: I) -> Self {
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
        let _range: NumericRangeIndex<i32, ()> = NumericRangeIndex::new(10, 20);
        let val: NumericValue<i32, ()> = NumericValue::new(13);
        let flat = (val.index - _range.start) as usize;
        let round: NumericValue<i32, ()> = NumericValue::new(flat as i32 + _range.start);
        assert_eq!(val.index, round.index);
    }
}
