//! Traits for extracting elements from tuples by type-level index.
//!
//! Provides `TupleExtract` and `ExtractAt` for type-safe, generic extraction of tuple elements.

use peano::{NonNeg, Succ, Zero};
use crate::tuple_utils::{Tuple, TupleFirstElement, TupleAppend, TuplePrepend, DropFirst};

/// Trait for extracting a given member of a tuple using type-level numbers.
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

/// Trait for extracting at a type-level index.
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

#[cfg(test)]
mod tests {
    use super::*;
    use peano::{P1, P3};
    #[test]
    fn test_tuple_extract() {
        let tuple = (1, "a", 3.14, true, 'x');
        // Extract the second element (index 1)
        let (_, extracted, _) = tuple.extract_at::<P1>();
        assert_eq!(extracted, "a");
        // Extract the fourth element (index 3)
        let (_, extracted, _) = tuple.extract_at::<P3>();
        assert_eq!(extracted, true);
    }
} 