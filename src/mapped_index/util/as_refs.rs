//! Provides the AsRefs trait for converting heterogeneous lists into references.
//!
//! # Example
//!
//! ```
//! use frunk::{HList, hlist};
//! use slice_and_dice::mapped_index::util::as_refs::AsRefs;
//!
//! let list = hlist![1, "hello"];
//! let refs = list.as_refs();
//! assert_eq!(*refs.head, 1);
//! assert_eq!(*refs.tail.head, "hello");
//! ```
//!
use frunk::{HCons, HNil};

pub trait AsRefs {
    type AsRefs<'a>: Copy
    where
        Self: 'a;

    fn as_refs(&self) -> Self::AsRefs<'_>;
}

impl AsRefs for HNil {
    type AsRefs<'a> = HNil;

    fn as_refs(&self) -> Self::AsRefs<'_> {
        HNil
    }
}

impl<Head, Tail> AsRefs for HCons<Head, Tail>
where
    Tail: AsRefs,
{
    type AsRefs<'a>
        = HCons<&'a Head, Tail::AsRefs<'a>>
    where
        Head: 'a,
        Tail: 'a;

    fn as_refs(&self) -> Self::AsRefs<'_> {
        HCons {
            head: &self.head,
            tail: self.tail.as_refs(),
        }
    }
}

pub type HRefs<'a, H> = <H as AsRefs>::AsRefs<'a>;
