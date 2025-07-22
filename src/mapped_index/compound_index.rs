use super::MappedIndex;

pub struct CompoundIndex<A, B> {
    pub a: A,
    pub b: B,
}

impl<'idx, A, B, IdxA, IdxB> MappedIndex<'idx, (IdxA, IdxB)> for CompoundIndex<A, B>
where
    A: MappedIndex<'idx, IdxA> + Sized,
    B: MappedIndex<'idx, IdxB> + Sized,
    A::Value: Copy + PartialEq + core::fmt::Debug,
    B::Value: Copy + PartialEq + core::fmt::Debug,
{
    type Value = (A::Value, B::Value);
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        // Not implemented for simplicity
        std::iter::empty()
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        let a_idx = self.a.to_flat_index(value.0);
        let b_idx = self.b.to_flat_index(value.1);
        a_idx * self.b.size() + b_idx
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        let b_size = self.b.size();
        let a_idx = index / b_size;
        let b_idx = index % b_size;
        let a_val = self.a.from_flat_index(a_idx);
        let b_val = self.b.from_flat_index(b_idx);
        (a_val, b_val)
    }
    fn size(&self) -> usize {
        self.a.size() * self.b.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::categorical_index::CategoricalIndex;
    use crate::mapped_index::numeric_range_index::NumericRangeIndex;
    use crate::mapped_index::MappedIndex;
    #[test]
    fn test_compound_index_get() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct TagA;
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct TagB;
        let cat = CategoricalIndex { values: vec![10, 20], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let compound = CompoundIndex { a: &cat, b: &num };
        let idx = (1usize, 1i32);
        let val = compound.from_flat_index(compound.to_flat_index((cat.from_flat_index(idx.0), num.from_flat_index(idx.1 as usize))));
        let (cat_val, num_val) = val;
        assert_eq!(*cat_val.value, 20);
        assert_eq!(num_val.index, 1);
    }
    #[test]
    fn test_flat_index_round_trip() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct TagA;
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct TagB;
        let cat = CategoricalIndex { values: vec![10, 20], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let compound = CompoundIndex { a: &cat, b: &num };
        let idx = (1usize, 1i32);
        let val = compound.from_flat_index(compound.to_flat_index((cat.from_flat_index(idx.0), num.from_flat_index(idx.1 as usize))));
        let flat = compound.to_flat_index(val);
        let round = compound.from_flat_index(flat);
        assert_eq!(compound.to_flat_index(round), flat);
        assert_eq!(round, val);
    }
} 