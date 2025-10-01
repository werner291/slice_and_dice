use super::VariableRange;
use std::fmt::Debug;

pub trait NumericRangeValue: 'static + Copy + Ord + Debug + Sync + Send {
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
impl_numeric_range_value!(i32);
impl_numeric_range_value!(u32);

#[macro_export]
macro_rules! nrange_newtype {
    ($type:ident, $inner:ty) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub struct $type(pub $inner);

        impl $type {
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl NumericRangeValue for $type {
            fn next(&self) -> Self {
                Self(self.0.next())
            }

            fn nth_next(&self, n: usize) -> Self {
                Self(self.0.nth_next(n))
            }

            fn distance(&self, other: &Self) -> usize {
                self.0.distance(&other.0)
            }
        }
    };

    ($type:ident, $inner:ty, $($trait:path),+) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, $($trait),+)]
        pub struct $type(pub $inner);

        impl $type {
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl NumericRangeValue for $type {
            fn next(&self) -> Self {
                Self(self.0.next())
            }

            fn nth_next(&self, n: usize) -> Self {
                Self(self.0.nth_next(n))
            }

            fn distance(&self, other: &Self) -> usize {
                self.0.distance(&other.0)
            }
        }
    };
}

/// An index representing a numeric range from `start` to `end` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct NumericRangeIndex<I: Debug> {
    /// The start of the range (inclusive).
    pub start: I,
    /// The end of the range (exclusive).
    pub end: I,
}

impl<I: Debug + Ord> NumericRangeIndex<I> {
    /// Create a new numeric range index [start, end) (end exclusive).
    ///
    /// # Examples
    /// ```
    /// use slice_and_dice::NumericRangeIndex;
    /// use slice_and_dice::mapped_index::VariableRange;
    /// let idx = NumericRangeIndex::<i32>::new(0, 3);
    /// assert_eq!(idx.size(), 3);
    /// ```
    pub fn new(start: I, end: I) -> Self {
        assert!(start < end, "Start must be less than end.");
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
        self.start.distance(&self.end)
    }
}
