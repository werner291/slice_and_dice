//! Type aliases for common tuple operations.
//!
//! Provides convenient type-level names for tuple element access, concatenation, and extraction.

pub type First<T> = <T as crate::tuple_utils::TupleFirstElement>::First;
pub type DropFirst<T> = <T as crate::tuple_utils::TupleFirstElement>::Rest;
pub type Prepend<A, T> = <T as crate::tuple_utils::TuplePrepend>::PrependedTuple<A>;
pub type Concat<Left, Right> = <Left as crate::tuple_utils::TupleConcat>::ConcatenatedTuple<Right>;
pub type Extract<N, T> = <T as crate::tuple_utils::TupleExtract<N>>::Result;
pub type ExtractLeft<N, T> = <T as crate::tuple_utils::TupleExtract<N>>::Before;
pub type ExtractRight<N, T> = <T as crate::tuple_utils::TupleExtract<N>>::After;
pub type ExtractRemainder<N, T> = Concat<ExtractLeft<N, T>, ExtractRight<N, T>>; 