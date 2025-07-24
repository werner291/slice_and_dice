//! Trait for converting tuples to tuples of references.
//!
//! Provides `TupleAsRefs` for obtaining a tuple of references from a tuple of values.

/// Trait for converting a tuple to a tuple of references.
pub trait TupleAsRefs {
    type AsTupleOfRefs<'a>
    where
        Self: 'a;
    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_>;
}

impl<'a> TupleAsRefs for () {
    type AsTupleOfRefs<'b> = () where Self: 'b;
    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> { () }
}

impl<A> TupleAsRefs for (A,) {
    type AsTupleOfRefs<'a> = (&'a A,) where Self: 'a;
    fn as_tuple_of_refs(&self) -> Self::AsTupleOfRefs<'_> { (&self.0,) }
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

#[cfg(test)]
mod tests {
    use super::*;
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