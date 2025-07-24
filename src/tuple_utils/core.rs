//! Core tuple marker traits and types for generic tuple manipulation.
//!
//! Provides the `Tuple` and `NonEmptyTuple` marker traits, and the `TooBig` type for compile-time bounds.
use peano::{NonNeg, Succ, Zero};
use crate::tuple_utils::{TupleAppend, TuplePrepend, TupleConcat, TupleFirstElement};

/// A placeholder dummy type for "a tuple that is too big"
pub struct TooBig;

/// Marker trait for tuples supporting append, prepend, and concat operations.
pub trait Tuple: TupleAppend + TuplePrepend + TupleConcat {}

impl Tuple for () {}
macro_rules! impl_tuple {
    // Recursive case: Implement for a tuple and reduce its size
    (($head:ident, $($tail:ident),+)) => {
        impl<$head, $($tail),+> Tuple for ($head, $($tail),+,) {}
        impl_tuple!(($($tail),+));
    };
    // Base case: Stop recursion when only one element is left
    (($head:ident)) => {
        impl<$head> Tuple for ($head,) {}
    };
}

// Generate implementations for tuples of size 2 and up
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10));

impl Tuple for TooBig {}

// Implementations for TooBig to satisfy trait bounds
impl crate::tuple_utils::prepend_append::TupleAppend for TooBig {
    type AppendedTuple<A> = TooBig;
    fn append<A>(self, _tail: A) -> Self::AppendedTuple<A> { unimplemented!() }
}
impl crate::tuple_utils::prepend_append::TuplePrepend for TooBig {
    type PrependedTuple<A> = TooBig;
    fn prepend<A>(self, _head: A) -> Self::PrependedTuple<A> { unimplemented!() }
}
impl crate::tuple_utils::first_last::TupleFirstElement for TooBig {
    type First = TooBig;
    type Rest = ();
    fn split_first(self) -> (Self::First, Self::Rest) { unimplemented!() }
}

/// Marker trait for non-empty tuples.
pub trait NonEmptyTuple: Tuple + TupleFirstElement {}

impl NonEmptyTuple for TooBig {}

macro_rules! impl_non_empty_tuple {
    // Recursive case: Implement for a tuple and reduce its size
    (($head:ident, $($tail:ident),*)) => {
        impl<$head, $($tail),*> NonEmptyTuple for ($head, $($tail),*) {}
        impl_non_empty_tuple!(($($tail),*));
    };
    // Base case: Implement for a single-element tuple
    (($head:ident)) => {
        impl<$head> NonEmptyTuple for ($head,) {}
    };
}

// Generate implementations for non-empty tuples of size 1 and up
impl_non_empty_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10));

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tuple_marker_traits() {
        fn assert_tuple<T: Tuple>() {}
        fn assert_non_empty<T: NonEmptyTuple>() {}
        assert_tuple::<()>();
        assert_tuple::<(i32, f64)>();
        assert_non_empty::<(i32,)>();
        assert_non_empty::<(i32, f64)>();
    }
} 