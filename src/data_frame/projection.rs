//! Projection traits and helpers for DataFrame, split from aggregation.rs for maintainability.
use crate::mapped_index::compound_index::CompoundIndex;
use typenum::Unsigned;

// Helper trait to project out the N-th element from a tuple or CompoundIndex
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

// Implement ProjectOut for CompoundIndex, so projecting out a dimension returns a CompoundIndex with that sub-index removed
impl<N, Indices> ProjectOut<N> for CompoundIndex<Indices>
where
    Indices: ProjectOut<N>,
    <Indices as ProjectOut<N>>::Remaining: Clone,
{
    type Remaining = CompoundIndex<<Indices as ProjectOut<N>>::Remaining>;
    type Removed = <Indices as ProjectOut<N>>::Removed;
    fn project_out(self) -> (Self::Removed, Self::Remaining) {
        let (removed, remaining) = self.indices.project_out();
        (removed, CompoundIndex::new(remaining))
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
    fn test_project_out_compound_index_first() {
        let idx = CompoundIndex::new((10, (20, (30, ()))));
        let (removed, projected): (i32, CompoundIndex<(i32, (i32, ()))>) = ProjectOut::<U0>::project_out(idx);
        assert_eq!(removed, 10);
        assert_eq!(projected.indices, (20, (30, ())))
    }

    #[test]
    fn test_project_out_compound_index_middle() {
        let idx = CompoundIndex::new((10, (20, (30, (40, ())))));
        let (removed, projected): (i32, CompoundIndex<(i32, (i32, ()))>) = ProjectOut::<U1>::project_out(idx);
        assert_eq!(removed, 20);
        assert_eq!(projected.indices, (10, (30, (40, ()))));
    }

    #[test]
    fn test_project_out_compound_index_last() {
        let idx = CompoundIndex::new((10, (20, (30, (40, ())))));
        let (removed, projected): (i32, CompoundIndex<(i32, (i32, ()))>) = ProjectOut::<U2>::project_out(idx);
        assert_eq!(removed, 30);
        assert_eq!(projected.indices, (10, (20, (40, ()))));
    }
} 