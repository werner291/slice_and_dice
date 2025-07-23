#![allow(non_snake_case)]

// Trait for tuples with a first element.
pub trait TupleHead {
    type Head;
    type Tail;

    fn split_head(self) -> (Self::Head, Self::Tail);
}

macro_rules! impl_tuple_head {
    ($head:ident, $($tail:ident),*) => {
        impl<$head, $($tail),*> TupleHead for ($head, $($tail),*) {
            type Head = $head;
            type Tail = ($($tail),*);

            fn split_head(self) -> (Self::Head, Self::Tail) {
                let ($head, $($tail),*) = self;
                ($head, ($($tail),*))
            }
        }
    };
}

impl<A> TupleHead for (A,) {
    type Head = A;
    type Tail = ();

    fn split_head(self) -> (Self::Head, Self::Tail) {
        (self.0, ())
    }
}

impl_tuple_head!(A, B);
impl_tuple_head!(A, B, C);
impl_tuple_head!(A, B, C, D);
impl_tuple_head!(A, B, C, D, E);
impl_tuple_head!(A, B, C, D, E, F);
impl_tuple_head!(A, B, C, D, E, F, G);
impl_tuple_head!(A, B, C, D, E, F, G, H);
impl_tuple_head!(A, B, C, D, E, F, G, H, J);

// Trait for constructing one greater-size tuple
pub trait TupleCons {
    type TupleCons<A>;

    fn prepend<A>(self, head: A) -> Self::TupleCons<A>;
}

macro_rules! impl_tuple_cons {
    ($($tail:ident),*) => {
        impl<$($tail),*> TupleCons for ($($tail),*) {
            type TupleCons<Head> = (Head, $($tail),*);

            fn prepend<Head>(self, head: Head) -> Self::TupleCons<Head> {
                let ($($tail),*) = self;
                (head, $($tail),*)
            }
        }
    };
}

impl_tuple_cons!();
impl<A> TupleCons for (A,) {
    type TupleCons<Head> = (Head, A);

    fn prepend<Head>(self, head: Head) -> Self::TupleCons<Head> {
        let (A,) = self;
        (head, A)
    }
}

impl_tuple_cons!(A);
impl_tuple_cons!(A, B);
impl_tuple_cons!(A, B, C);
impl_tuple_cons!(A, B, C, D);
impl_tuple_cons!(A, B, C, D, E);
impl_tuple_cons!(A, B, C, D, E, F);
impl_tuple_cons!(A, B, C, D, E, F, G);
impl_tuple_cons!(A, B, C, D, E, F, G, H);
impl_tuple_cons!(A, B, C, D, E, F, G, H, J);

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

macro_rules! impl_tuple_as_refs {
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

impl_tuple_as_refs!(
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I)
);
