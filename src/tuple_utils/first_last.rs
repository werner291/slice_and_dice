#![allow(non_snake_case)]
//! Traits for accessing the first and last elements of tuples.
//!
//! Provides `TupleFirstElement` and `TupleLastElement` for splitting tuples at the ends.

/// Trait for tuples with a first element.
pub trait TupleFirstElement {
    type First;
    type Rest: super::core::Tuple;
    fn split_first(self) -> (Self::First, Self::Rest);
}

macro_rules! impl_tuple_first_element {
    ($first:ident, $($rest:ident),*) => {
        impl<$first, $($rest),*> TupleFirstElement for ($first, $($rest),*) {
            type First = $first;
            type Rest = ($($rest),*,);
            fn split_first(self) -> (Self::First, Self::Rest) {
                let ($first, $($rest),*) = self;
                ($first, ($($rest),*,))
            }
        }
    };
}

impl<A> TupleFirstElement for (A,) {
    type First = A;
    type Rest = ();
    fn split_first(self) -> (Self::First, Self::Rest) {
        (self.0, ())
    }
}

impl_tuple_first_element!(T1, T2);
impl_tuple_first_element!(T1, T2, T3);
impl_tuple_first_element!(T1, T2, T3, T4);
impl_tuple_first_element!(T1, T2, T3, T4, T5);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple_first_element!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

/// Trait for tuples with a last element.
pub trait TupleLastElement {
    type Last;
    type Rest: super::core::Tuple;
    fn split_last(self) -> (Self::Rest, Self::Last);
}

macro_rules! impl_tuple_last_element {
    ($($rest:ident),*) => {
        impl<$($rest,)* Last> TupleLastElement for ($($rest,)* Last) {
            type Last = Last;
            type Rest = ($($rest,)*);
            fn split_last(self) -> (Self::Rest, Self::Last) {
                let ($($rest,)* l,) = self;
                (($($rest,)*), l)
            }
        }
    };
}

impl_tuple_last_element!(A);
impl_tuple_last_element!(A, B);
impl_tuple_last_element!(A, B, C);
impl_tuple_last_element!(A, B, C, D);
impl_tuple_last_element!(A, B, C, D, E);
impl_tuple_last_element!(A, B, C, D, E, F);
impl_tuple_last_element!(A, B, C, D, E, F, G);
impl_tuple_last_element!(A, B, C, D, E, F, G, H);
impl_tuple_last_element!(A, B, C, D, E, F, G, H, I);
impl_tuple_last_element!(A, B, C, D, E, F, G, H, I, J);

impl<A> TupleLastElement for (A,) {
    type Last = A;
    type Rest = ();
    fn split_last(self) -> (Self::Rest, Self::Last) {
        ((), self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tuple_first_element() {
        let t2 = (1, "a");
        let (h, tail) = t2.split_first();
        assert_eq!(h, 1);
        assert_eq!(tail, ("a",));
        let t3 = (1, 2, 3);
        let (h, tail) = t3.split_first();
        assert_eq!(h, 1);
        assert_eq!(tail, (2, 3));
    }
    #[test]
    fn test_tuple_last_element() {
        let t1 = (1,);
        let (rest, last) = t1.split_last();
        assert_eq!(rest, ());
        assert_eq!(last, 1);
        let t3 = (1, 2, 3);
        let (rest, last) = t3.split_last();
        assert_eq!(rest, (1, 2));
        assert_eq!(last, 3);
    }
}
