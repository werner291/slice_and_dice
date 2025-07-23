#![allow(non_snake_case)]

use peano::{NonNeg, Succ, Zero};

// Trait for tuples with a first element.
pub trait TupleFirstElement {
    type First;
    type Rest;

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

impl_tuple_first_element!(A, B);
impl_tuple_first_element!(A, B, C);
impl_tuple_first_element!(A, B, C, D);
impl_tuple_first_element!(A, B, C, D, E);
impl_tuple_first_element!(A, B, C, D, E, F);
impl_tuple_first_element!(A, B, C, D, E, F, G);
impl_tuple_first_element!(A, B, C, D, E, F, G, H);
impl_tuple_first_element!(A, B, C, D, E, F, G, H, J);

pub trait TupleLastElement {
    type Last;
    type Rest;

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
    type PrependedTuple<A>;

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
impl_tuple_prepend!(A, B);
impl_tuple_prepend!(A, B, C);
impl_tuple_prepend!(A, B, C, D);
impl_tuple_prepend!(A, B, C, D, E);
impl_tuple_prepend!(A, B, C, D, E, F);
impl_tuple_prepend!(A, B, C, D, E, F, G);
impl_tuple_prepend!(A, B, C, D, E, F, G, H);
impl_tuple_prepend!(A, B, C, D, E, F, G, H, J);

pub trait TupleAppend {
    type AppendedTuple<A>;

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

impl_tuple_append!(A, B);
impl_tuple_append!(A, B, C);
impl_tuple_append!(A, B, C, D);
impl_tuple_append!(A, B, C, D, E);
impl_tuple_append!(A, B, C, D, E, F);
impl_tuple_append!(A, B, C, D, E, F, G);
impl_tuple_append!(A, B, C, D, E, F, G, H);
impl_tuple_append!(A, B, C, D, E, F, G, H, I);

pub trait TupleAsRefsTuple {
    type AsTupleOfRefs<'a>
    where
        Self: 'a;

    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_>;
}

impl<'a> TupleAsRefsTuple for () {
    type AsTupleOfRefs<'b>
        = ()
    where
        Self: 'b;

    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> {
        ()
    }
}

impl<A> TupleAsRefsTuple for (A,) {
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
            impl<$($name),*> TupleAsRefsTuple for ($($name),*) {
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
    type Before;
    type Result;
    type After;

    fn extract(self) -> (Self::Before, Self::Result, Self::After);
}

impl<T: TupleFirstElement> TupleExtract<Zero> for T {
    type Before = ();
    type Result = T::First;
    type After = T::Rest;

    fn extract(self) -> (Self::Before, Self::Result, Self::After) {
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

    fn extract(self) -> (Self::Before, Self::Result, Self::After) {
        let (h, t) = self.split_first();
        let (b, r, a) = t.extract();
        (b.prepend(h), r, a)
    }
}

// Helper method to avoid having to write out the trait every time.
pub fn extract<N: NonNeg, T: TupleExtract<N>>(tuple: T) -> (T::Before, T::Result, T::After) {
    tuple.extract()
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
            let (_, extracted, _) = extract::<peano::P1, _>(tuple);
            assert_eq!(extracted, "a");

            // Extract the fourth element (index 3)
            let (_, extracted, _) = extract::<peano::P3, _>(tuple);
            assert_eq!(extracted, true);
        }
    }
}
