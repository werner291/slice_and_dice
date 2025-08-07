use super::VariableRange;
use std::fmt::Debug;
use std::ops::Range;

trait NumericRangeValue: 'static + Copy + Ord + Debug {
    fn next(&self) -> Self;

    fn nth_next(&self, n: usize) -> Self;

    fn distance(&self, other: &Self) -> usize;
}

macro_rules! impl_numeric_range_value {
    ($type:ty) => {
        impl NumericRangeValue for $type {
            fn next(&self) -> Self {
                self + 1
            }

            fn nth_next(&self, n: usize) -> Self {
                self + n as Self
            }

            fn distance(&self, other: &Self) -> usize {
                if *self > *other {
                    (self - other) as usize
                } else {
                    (other - self) as usize
                }
            }
        }
    };
}

impl_numeric_range_value!(usize);
impl_numeric_range_value!(u64);
impl_numeric_range_value!(i64);

/// An index representing a numeric range from `start` to `end` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct NumericRangeIndex<I: Debug> {
    /// The start of the range (inclusive).
    pub start: I,
    /// The end of the range (exclusive).
    pub end: I,
}

impl<I: Debug> NumericRangeIndex<I> {
    pub fn new(start: I, end: I) -> Self {
        Self { start, end }
    }
}

impl<I: Clone + Debug> Clone for NumericRangeIndex<I> {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl<I> VariableRange for NumericRangeIndex<I>
where
    I: NumericRangeValue,
{
    type Value<'a> = I;

    /// Returns an iterator over all numeric values in the range.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let mut next = self.start;
        let end = self.end;

        std::iter::from_fn(move || {
            if next >= end {
                None
            } else {
                let current = next;
                next = next.next();
                Some(current)
            }
        })
    }

    /// Returns the numeric value for a given flat index.
    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        assert!(index < self.size(), "Index out of bounds.");
        self.start.nth_next(index)
    }

    /// Returns the number of values in the numeric range index.
    fn size(&self) -> usize {
        self.end.distance(&self.start)
    }
}
