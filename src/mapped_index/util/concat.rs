//! HList concatenation utilities for combining heterogeneous lists.
//!
//! # Examples
//!
//! ```
//! use frunk::{hlist, HList};
//! use slice_and_dice::mapped_index::util::concat::HListConcat;
//!
//! let list1 = hlist![1, "hello"];
//! let list2 = hlist![true, 42.0];
//!
//! let combined = list1.concat(list2);
//! assert_eq!(combined, hlist![1, "hello", true, 42.0]);
//! ```

use frunk::hlist::{HList, h_cons};
use frunk::{HCons, HNil};

pub type HLConcat<A, B> = <A as HListConcat<B>>::Concat;

pub trait HListConcat<Other: HList> {
    type Concat: HList;

    fn concat(self, other: Other) -> Self::Concat;
}

impl<Other: HList> HListConcat<Other> for HNil {
    type Concat = Other;

    fn concat(self, other: Other) -> Self::Concat {
        other
    }
}

impl<Head, Tail, Other: HList> HListConcat<Other> for HCons<Head, Tail>
where
    Tail: HListConcat<Other>,
{
    type Concat = HCons<Head, Tail::Concat>;

    fn concat(self, other: Other) -> Self::Concat {
        h_cons(self.head, self.tail.concat(other))
    }
}
