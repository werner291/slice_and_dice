//! Traits for prepending and appending elements to tuples.
//!
//! Provides `TuplePrepend` and `TupleAppend` for constructing new tuples by adding elements at the start or end.

/// Trait for constructing a tuple with an element prepended.
pub trait TuplePrepend {
    type PrependedTuple<A>: super::core::NonEmptyTuple;
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> TuplePrepend for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
    type PrependedTuple<A> = super::core::TooBig;
    fn prepend<A>(self, _head: A) -> Self::PrependedTuple<A> {
        unimplemented!()
    }
}

/// Trait for constructing a tuple with an element appended.
pub trait TupleAppend {
    type AppendedTuple<A>: super::core::NonEmptyTuple;
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> TupleAppend for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
    type AppendedTuple<A> = super::core::TooBig;
    fn append<A>(self, _tail: A) -> Self::AppendedTuple<A> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_tuple_append() {
        let t1 = (1,);
        let t2 = t1.append("a");
        assert_eq!(t2, (1, "a"));
        let t2 = (1, 2);
        let t3 = t2.append(3);
        assert_eq!(t3, (1, 2, 3));
    }
} 