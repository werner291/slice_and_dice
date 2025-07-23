#![allow(non_snake_case)]

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
            type Rest = ($($rest),*);

            fn split_first(self) -> (Self::First, Self::Rest) {
                let ($first, $($rest),*) = self;
                ($first, ($($rest),*))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuple_first_element() {
        let t2 = (1, "a");
        let (h, tail) = t2.split_first();
        assert_eq!(h, 1);
        assert_eq!(tail, "a");

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
}
