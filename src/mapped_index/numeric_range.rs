use super::VariableRange;

/// A value in a numeric range index, representing a position in the range.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct NumericRange<I: std::fmt::Debug> {
    /// The numeric index value.
    pub index: I,
}

impl<I: PartialEq + std::fmt::Debug> PartialEq for NumericRange<I> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<I: Eq + std::fmt::Debug> Eq for NumericRange<I> {}

impl<I: Copy + std::fmt::Debug> Copy for NumericRange<I> {}
impl<I: Copy + std::fmt::Debug> Clone for NumericRange<I> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An index representing a numeric range from `start` to `end` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct NumericRangeIndex<I: std::fmt::Debug> {
    /// The start of the range (inclusive).
    pub start: I,
    /// The end of the range (exclusive).
    pub end: I,
}

impl<I: Clone + std::fmt::Debug> Clone for NumericRangeIndex<I> {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl<I: PartialEq + std::fmt::Debug> PartialEq for NumericRangeIndex<I> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl<I: Eq + std::fmt::Debug> Eq for NumericRangeIndex<I> {}

impl<I> NumericRangeIndex<I>
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
    pub const fn new(start: I, end: I) -> Self {
        Self { start, end }
    }
    pub fn at(&self, index: I) -> NumericRange<I> {
        if index >= self.start && index < self.end {
            NumericRange { index }
        } else {
            panic!(
                "Index {:?} out of bounds ({:?}..{:?})",
                index, self.start, self.end
            );
        }
    }
}

impl<I> VariableRange for NumericRangeIndex<I>
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
    type Value<'a> = NumericRange<I>;

    /// Returns an iterator over all numeric values in the range.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let start = self.start;
        let end = self.end;
        let start_usize: usize = start.try_into().unwrap();
        let end_usize: usize = end.try_into().unwrap();
        (start_usize..end_usize).map(move |i| NumericRange {
            index: I::try_from(i).unwrap_or_else(|e| panic!("Failed to convert: {:?}", e)),
        })
    }

    /// Returns the numeric value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        NumericRange {
            index: self.start
                + I::try_from(index).unwrap_or_else(|e| panic!("Failed to convert: {:?}", e)),
        }
    }

    /// Returns the number of values in the numeric range index.
    fn size(&self) -> usize {
        let start: usize = self.start.try_into().unwrap();
        let end: usize = self.end.try_into().unwrap();
        end - start
    }
}

impl<I: Copy + std::fmt::Debug + 'static> std::ops::Index<I> for NumericRangeIndex<I> {
    type Output = NumericRange<I>;
    fn index(&self, _index: I) -> &Self::Output {
        panic!("Cannot return a reference to a value by index; use get() or at() instead.");
    }
}

impl<'idx, I: std::fmt::Debug> NumericRange<I> {
    /// Create a new NumericValue from an index.
    pub const fn new(index: I) -> Self {
        Self { index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::VariableRange;
    #[test]
    fn test_flat_index_round_trip() {
        let _range: NumericRangeIndex<i32> = NumericRangeIndex::new(10, 20);
        let val: NumericRange<i32> = NumericRange::new(13);
        let flat = (val.index - _range.start) as usize;
        let round: NumericRange<i32> = NumericRange::new(flat as i32 + _range.start);
        assert_eq!(val.index, round.index);
    }
}
