//! Trait for concatenating tuples.
//!
//! Provides `TupleConcat` for combining two tuples into one.

use crate::tuple_utils::aliases::{DropFirst, First, Prepend};
use crate::tuple_utils::core::Tuple;
use crate::tuple_utils::first_last::TupleFirstElement;
use crate::tuple_utils::prepend_append::TuplePrepend;

/// Trait for concatenating two tuples.
pub trait TupleConcat {
    type ConcatenatedTuple<T: Tuple>: Tuple;
    fn concat<Right: Tuple>(self, other: Right) -> Self::ConcatenatedTuple<Right>;
}

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
    type ConcatenatedTuple<Right: Tuple> =
        Prepend<First<Self>, <DropFirst<Self> as TupleConcat>::ConcatenatedTuple<Right>>;
    fn concat<Right: Tuple>(self, other: Right) -> Self::ConcatenatedTuple<Right>
    where
        <DropFirst<Left> as TupleConcat>::ConcatenatedTuple<Right>: TuplePrepend,
    {
        let (first, rest) = self.split_first();
        rest.concat(other).prepend(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
