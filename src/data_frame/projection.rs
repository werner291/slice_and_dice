//! Projection traits and helpers for DataFrame, split from aggregation.rs for maintainability.
use crate::mapped_index::compound_index::CompoundIndex;
use typenum::Unsigned;

// Helper trait to project out the N-th element from a tuple
pub trait ProjectOut<N> {
    type Remaining;
    type Removed;
    fn project_out(self) -> (Self::Removed, Self::Remaining);
}

impl<T, Tail> ProjectOut<typenum::U0> for (T, Tail) {
    type Remaining = Tail;
    type Removed = T;
    fn project_out(self) -> (Self::Removed, Self::Remaining) {
        let (head, tail) = self;
        (head, tail)
    }
}

impl<T, Tail, N> ProjectOut<typenum::UInt<N, typenum::B1>> for (T, Tail)
where
    Tail: ProjectOut<N>,
    N: Unsigned,
{
    type Remaining = (T, <Tail as ProjectOut<N>>::Remaining);
    type Removed = <Tail as ProjectOut<N>>::Removed;
    fn project_out(self) -> (Self::Removed, Self::Remaining) {
        let (head, tail) = self;
        let (removed, new_tail) = tail.project_out();
        (removed, (head, new_tail))
    }
}

// Helper trait to project out the N-th index from a CompoundIndex
pub trait ProjectedIndex<N> {
    type Remaining;
    fn project_index(&self) -> Self::Remaining;
}

impl<N, Head, Tail> ProjectedIndex<N> for CompoundIndex<(Head, Tail)>
where
    (Head, Tail): ProjectOut<N>,
    <(Head, Tail) as ProjectOut<N>>::Remaining: Clone,
{
    type Remaining = CompoundIndex<<(Head, Tail) as ProjectOut<N>>::Remaining>;
    fn project_index(&self) -> Self::Remaining {
        let (_, remaining) = self.indices.clone().project_out();
        CompoundIndex::new(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::compound_index::CompoundIndex;
    use typenum::{U0, U1, U2};

    #[test]
    fn test_project_out_tuple_first() {
        let tup = (1, (2, (3, ())));
        let (removed, remaining): (i32, (i32, (i32, ()))) = ProjectOut::<U0>::project_out(tup);
        assert_eq!(removed, 1);
        assert_eq!(remaining, (2, (3, ())))
    }

    #[test]
    fn test_project_out_tuple_middle() {
        let tup = (1, (2, (3, (4, ()))));
        let (removed, remaining): (i32, (i32, (i32, ()))) = ProjectOut::<U1>::project_out(tup);
        assert_eq!(removed, 2);
        assert_eq!(remaining, (1, (3, (4, ()))));
    }

    #[test]
    fn test_project_out_tuple_last() {
        let tup = (1, (2, (3, (4, ()))));
        let (removed, remaining): (i32, (i32, (i32, ()))) = ProjectOut::<U2>::project_out(tup);
        assert_eq!(removed, 3);
        assert_eq!(remaining, (1, (2, (4, ()))));
    }

    #[test]
    fn test_projected_index_first() {
        let idx = CompoundIndex::new((10, (20, (30, ()))));
        let projected = idx.project_index::<U0>();
        assert_eq!(projected.indices, (20, (30, ())))
    }

    #[test]
    fn test_projected_index_middle() {
        let idx = CompoundIndex::new((10, (20, (30, (40, ())))));
        let projected = idx.project_index::<U1>();
        assert_eq!(projected.indices, (10, (30, (40, ()))));
    }

    #[test]
    fn test_projected_index_last() {
        let idx = CompoundIndex::new((10, (20, (30, (40, ())))));
        let projected = idx.project_index::<U2>();
        assert_eq!(projected.indices, (10, (20, (40, ()))));
    }
} 