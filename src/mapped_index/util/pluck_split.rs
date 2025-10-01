//! This module implements pluck split functionality for heterogeneous lists, allowing extraction
//! of an element at a specific index position while splitting the list into left and right parts.
//!
//! # Example
//! ```rust
//! use frunk::hlist;
//! use slice_and_dice::mapped_index::util::pluck_split::PluckSplit;
//! use slice_and_dice::mapped_index::compound_index::Dim0;
//!
//! let list = hlist![1, "hello", 3.14];
//! let (left, extracted, right) = list.pluck_split::<Dim0>();
//! assert_eq!(left, hlist![]);
//! assert_eq!(extracted, 1);
//! assert_eq!(right, hlist!["hello", 3.14]);
//! ```

use crate::mapped_index::util::concat::HLConcat;
use frunk::hlist::{HList, h_cons};
use frunk::indices::{Here, There};
use frunk::{HCons, HNil};

pub trait PluckSplit {
    fn pluck_split<At>(self) -> (Self::Left, Self::Extract, Self::Right)
    where
        Self: PluckSplitImpl<At>,
        Self::Left: HList,
        Self::Right: HList;
}

pub type PluckAt<At, List> = <List as PluckSplitImpl<At>>::Extract;
pub type PluckLeft<At, List> = <List as PluckSplitImpl<At>>::Left;
pub type PluckRight<At, List> = <List as PluckSplitImpl<At>>::Right;
pub type PluckRemainder<At, List> = HLConcat<PluckLeft<At, List>, PluckRight<At, List>>;

impl<T> PluckSplit for T {
    fn pluck_split<At>(
        self,
    ) -> (
        <Self as PluckSplitImpl<At>>::Left,
        <Self as PluckSplitImpl<At>>::Extract,
        <Self as PluckSplitImpl<At>>::Right,
    )
    where
        Self: PluckSplitImpl<At>,
        <Self as PluckSplitImpl<At>>::Left: HList,
        <Self as PluckSplitImpl<At>>::Right: HList,
    {
        self.pluck_split_impl()
    }
}

pub trait PluckSplitImpl<At> {
    type Left: HList;
    type Extract;
    type Right: HList;

    fn pluck_split_impl(self) -> (Self::Left, Self::Extract, Self::Right);
}

impl<Head, Tail> PluckSplitImpl<Here> for HCons<Head, Tail>
where
    Tail: HList,
{
    type Left = HNil;
    type Extract = Head;
    type Right = Tail;

    fn pluck_split_impl(self) -> (Self::Left, Self::Extract, Self::Right) {
        (HNil, self.head, self.tail)
    }
}

impl<Head, Tail: PluckSplitImpl<ThereTail>, ThereTail> PluckSplitImpl<There<ThereTail>>
    for HCons<Head, Tail>
where
    Tail: HList,
{
    type Left = HCons<Head, Tail::Left>;
    type Extract = Tail::Extract;
    type Right = Tail::Right;

    fn pluck_split_impl(self) -> (Self::Left, Self::Extract, Self::Right) {
        let (l, e, r) = self.tail.pluck_split_impl();
        (h_cons(self.head, l), e, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::compound_index::{Dim0, Dim1, Dim2};
    use frunk::hlist;

    #[test]
    fn test_pluck_first() {
        let list = hlist![1, "hello", 3.14];
        let (left, extracted, right) = list.pluck_split::<Dim0>();
        assert_eq!(left, hlist![]);
        assert_eq!(extracted, 1);
        assert_eq!(right, hlist!["hello", 3.14]);
    }

    #[test]
    fn test_pluck_second() {
        let list = hlist![1, "hello", 3.14];
        let (left, extracted, right) = list.pluck_split::<Dim1>();
        assert_eq!(left, hlist![1]);
        assert_eq!(extracted, "hello");
        assert_eq!(right, hlist![3.14]);
    }

    #[test]
    fn test_pluck_third() {
        let list = hlist![1, "hello", 3.14];
        let (left, extracted, right) = list.pluck_split::<Dim2>();
        assert_eq!(left, hlist![1, "hello"]);
        assert_eq!(extracted, 3.14);
        assert_eq!(right, hlist![]);
    }
}
