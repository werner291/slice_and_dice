#![allow(non_snake_case)]

use peano::{NonNeg, Succ, Zero};

/// A placeholder dummy type for "a tuple that is too big"
pub struct TooBig;

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

// Trait for tuples with a first element.
pub trait TupleFirstElement {
    type First;
    type Rest: Tuple;

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

pub trait TupleLastElement {
    type Last;
    type Rest: Tuple;

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

// Trait for constructing one greater-size tuple
pub trait TuplePrepend {
    type PrependedTuple<A>: NonEmptyTuple;

    fn prepend<A>(self, head: A) -> Self::PrependedTuple<A>;
}

macro_rules! impl_tuple_prepend {
    ($($rest:ident),*) => {
        impl<$($rest),*> TuplePrepend for ($($rest),*) {
            type PrependedTuple<Head> = (Head, $($rest),*);

            fn prepend<Head>(self, head: Head) -> Self::PrependedTuple<Head> {
                let ($($rest),*) = self;
                (head, $($rest),*)
            }
        }
    };
}

impl_tuple_prepend!();
impl<A> TuplePrepend for (A,) {
    type PrependedTuple<Head> = (Head, A);

    fn prepend<Head>(self, head: Head) -> Self::PrependedTuple<Head> {
        let (A,) = self;
        (head, A)
    }
}
impl_tuple_prepend!(T1, T2);
impl_tuple_prepend!(T1, T2, T3);
impl_tuple_prepend!(T1, T2, T3, T4);
impl_tuple_prepend!(T1, T2, T3, T4, T5);
impl_tuple_prepend!(T1, T2, T3, T4, T5, T6);
impl_tuple_prepend!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_prepend!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_prepend!(T1, T2, T3, T4, T5, T6, T7, T8, T9);

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> TuplePrepend
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
{
    type PrependedTuple<A> = TooBig;

    fn prepend<A>(self, head: A) -> Self::PrependedTuple<A> {
        unimplemented!()
    }
}

impl TupleAppend for TooBig {
    type AppendedTuple<A> = TooBig;

    fn append<A>(self, _tail: A) -> Self::AppendedTuple<A> {
        unimplemented!()
    }
}

impl TuplePrepend for TooBig {
    type PrependedTuple<A> = TooBig;

    fn prepend<A>(self, _head: A) -> Self::PrependedTuple<A> {
        unimplemented!()
    }
}

impl TupleFirstElement for TooBig {
    type First = TooBig;
    type Rest = ();

    fn split_first(self) -> (Self::First, Self::Rest) {
        unimplemented!()
    }
}

pub trait TupleAppend {
    type AppendedTuple<A>: NonEmptyTuple;

    fn append<A>(self, tail: A) -> Self::AppendedTuple<A>;
}

macro_rules! impl_tuple_append {
    ($($name:ident),*) => {
        impl<$($name),*> TupleAppend for ($($name),*) {
            type AppendedTuple<Tail> = ($($name,)* Tail,);

            fn append<Tail>(self, tail: Tail) -> Self::AppendedTuple<Tail> {
                let ($($name),*) = self;
                ($($name,)* tail,)
            }
        }
    };
}

impl_tuple_append!();
impl<A> TupleAppend for (A,) {
    type AppendedTuple<Tail> = (A, Tail);

    fn append<Tail>(self, tail: Tail) -> Self::AppendedTuple<Tail> {
        (self.0, tail)
    }
}

impl_tuple_append!(T1, T2);
impl_tuple_append!(T1, T2, T3);
impl_tuple_append!(T1, T2, T3, T4);
impl_tuple_append!(T1, T2, T3, T4, T5);
impl_tuple_append!(T1, T2, T3, T4, T5, T6);
impl_tuple_append!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_append!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_append!(T1, T2, T3, T4, T5, T6, T7, T8, T9);

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> TupleAppend
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
{
    type AppendedTuple<A> = TooBig;

    fn append<A>(self, _tail: A) -> Self::AppendedTuple<A> {
        unimplemented!()
    }
}

pub trait TupleAsRefs {
    type AsTupleOfRefs<'a>
    where
        Self: 'a;

    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_>;
}

impl<'a> TupleAsRefs for () {
    type AsTupleOfRefs<'b>
        = ()
    where
        Self: 'b;

    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> {
        ()
    }
}

impl<A> TupleAsRefs for (A,) {
    type AsTupleOfRefs<'a>
        = (&'a A,)
    where
        Self: 'a;

    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> {
        (&self.0,)
    }
}

macro_rules! impl_tuple_as_refs_tuple {
    ($(($($name:ident),*)),*) => {
        $(
            impl<$($name),*> TupleAsRefs for ($($name),*) {
                type AsTupleOfRefs<'a> = ($(&'a $name),*) where Self: 'a;

                fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> {
                    let ($($name),*) = self;
                    ($(&$name),*)
                }
            }
        )*
    };
}

impl_tuple_as_refs_tuple!(
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I)
);

/// A trait for extracting a given member of a tuple using type-level numbers.
pub trait TupleExtract<N: NonNeg> {
    type Before: Tuple;
    type Result;
    type After: Tuple;

    fn extract_recursive(self) -> (Self::Before, Self::Result, Self::After);
}

impl<T: TupleFirstElement> TupleExtract<Zero> for T {
    type Before = ();
    type Result = T::First;
    type After = T::Rest;

    fn extract_recursive(self) -> (Self::Before, Self::Result, Self::After) {
        self.split_first().prepend(())
    }
}

impl<T, N> TupleExtract<Succ<N>> for T
where
    T::Rest: TupleAppend,
    T: TupleFirstElement + TupleExtract<N>,
    N: NonNeg,
    <T as TupleFirstElement>::Rest: TupleExtract<N>,
    <<T as TupleFirstElement>::Rest as TupleExtract<N>>::Before: TuplePrepend,
{
    type Before = <<<T as TupleFirstElement>::Rest as TupleExtract<N>>::Before as TuplePrepend>::PrependedTuple<T::First>;
    type Result = <<T as TupleFirstElement>::Rest as TupleExtract<N>>::Result;
    type After = <<T as TupleFirstElement>::Rest as TupleExtract<N>>::After;

    fn extract_recursive(self) -> (Self::Before, Self::Result, Self::After) {
        let (h, t) = self.split_first();
        let (b, r, a) = t.extract_recursive();
        (b.prepend(h), r, a)
    }
}

pub trait ExtractAt {
    fn extract_at<N: NonNeg>(
        self,
    ) -> (
        <Self as TupleExtract<N>>::Before,
        <Self as TupleExtract<N>>::Result,
        <Self as TupleExtract<N>>::After,
    )
    where
        Self: TupleExtract<N>;
}

impl<T> ExtractAt for T {
    fn extract_at<N: NonNeg>(
        self,
    ) -> (
        <Self as TupleExtract<N>>::Before,
        <Self as TupleExtract<N>>::Result,
        <Self as TupleExtract<N>>::After,
    )
    where
        Self: TupleExtract<N>,
    {
        self.extract_recursive()
    }
}

/// A trait for concatenating two tuples.
pub trait TupleConcat {
    type ConcatenatedTuple<T: Tuple>: Tuple;
    fn concat<Right: Tuple>(self, other: Right) -> Self::ConcatenatedTuple<Right>;
}

pub type First<T: TupleFirstElement> = T::First;
pub type DropFirst<T: TupleFirstElement> = T::Rest;
pub type Prepend<A, T: TuplePrepend> = T::PrependedTuple<A>;
pub type Concat<Left: TupleConcat, Right> = Left::ConcatenatedTuple<Right>;

pub type Extract<N, T> = <T as TupleExtract<N>>::Result;
pub type ExtractLeft<N, T> = <T as TupleExtract<N>>::Before;
pub type ExtractRight<N, T> = <T as TupleExtract<N>>::After;
pub type ExtractRemainder<N, T> = Concat<ExtractLeft<N, T>, ExtractRight<N, T>>;

// Base case: Empty tuple concatenated with any tuple
impl TupleConcat for () {
    type ConcatenatedTuple<Right: Tuple> = Right;
    fn concat<Right: Tuple>(self, other: Right) -> Self::ConcatenatedTuple<Right> {
        other
    }
}

impl<Left> TupleConcat for Left
where
    Left: TupleFirstElement,
    DropFirst<Left>: TupleConcat,
{
    type ConcatenatedTuple<Right: Tuple> = Prepend<First<Self>, Concat<DropFirst<Self>, Right>>;

    fn concat<Right: Tuple>(self, other: Right) -> Self::ConcatenatedTuple<Right>
    where
        Concat<DropFirst<Left>, Right>: TuplePrepend,
    {
        let (first, rest) = self.split_first();
        rest.concat(other).prepend(first)
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
    fn test_tuple_prepend() {
        let t1 = ("b",);
        let t2 = t1.prepend(1);
        assert_eq!(t2, (1, "b"));

        let t2 = (2, 3);
        let t3 = t2.prepend(1);
        assert_eq!(t3, (1, 2, 3));
    }

    #[test]
    fn test_tuple_as_refs_tuple() {
        let t1 = (42,);
        let refs = t1.as_tuple_of_refs();
        assert_eq!(*refs.0, 42);

        let t3 = (1, 2, 3);
        let refs = t3.as_tuple_of_refs();
        assert_eq!(*refs.0, 1);
        assert_eq!(*refs.1, 2);
        assert_eq!(*refs.2, 3);
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

    #[test]
    fn test_tuple_append() {
        let t1 = (1,);
        let t2 = t1.append("a");
        assert_eq!(t2, (1, "a"));

        let t2 = (1, 2);
        let t3 = t2.append(3);
        assert_eq!(t3, (1, 2, 3));
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_tuple_extract() {
            let tuple = (1, "a", 3.14, true, 'x');

            // Extract the second element (index 1)
            let (_, extracted, _) = tuple.extract_at::<peano::P1>();
            assert_eq!(extracted, "a");

            // Extract the fourth element (index 3)
            let (_, extracted, _) = tuple.extract_at::<peano::P3>();
            assert_eq!(extracted, true);
        }
    }

    #[test]
    fn test_tuple_concat() {
        // Empty tuple tests
        let t1 = ();
        let t2 = (1, 2);
        assert_eq!(t1.concat(t2), (1, 2));
        assert_eq!(t2.concat(t1), (1, 2));

        // Single element tests
        let t1 = (1,);
        let t2 = (2,);
        assert_eq!(t1.concat(t2), (1, 2));

        // Multiple element tests
        let t1 = (1, 2);
        let t2 = (3, 4);
        assert_eq!(t1.concat(t2), (1, 2, 3, 4));

        // Mixed type tests
        let t1 = (1, "hello");
        let t2 = (true, 3.14);
        let result = t1.concat(t2);
        assert_eq!(result, (1, "hello", true, 3.14));
    }
}
