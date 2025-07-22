use super::MappedIndex;
use tuple_list::{TupleList, Tuple};

/// An index that combines multiple sub-indices into a compound, multi-dimensional index.
///
/// The flat index is computed by flattening the tuple of indices into a single dimension.
#[derive(Debug, Clone)]
pub struct CompoundIndex<Indices> {
    /// The tuple of sub-indices.
    pub indices: Indices,
}

impl<Indices: PartialEq> PartialEq for CompoundIndex<Indices> {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}

impl<Indices: Eq> Eq for CompoundIndex<Indices> {}

impl<Indices> CompoundIndex<Indices> {
    pub fn to_flat_index<'idx, IdxTuple>(&self, value: <Self as MappedIndex<'idx, IdxTuple>>::Value) -> usize
    where
        Self: MappedIndex<'idx, IdxTuple>,
    {
        <Self as MappedIndex<'idx, IdxTuple>>::to_flat_index(self, value)
    }
    pub fn from_flat_index<'idx, IdxTuple>(&'idx self, index: usize) -> <Self as MappedIndex<'idx, IdxTuple>>::Value
    where
        Self: MappedIndex<'idx, IdxTuple>,
    {
        <Self as MappedIndex<'idx, IdxTuple>>::from_flat_index(self, index)
    }
    pub fn size<'idx, IdxTuple>(&self) -> usize
    where
        Self: MappedIndex<'idx, IdxTuple>,
    {
        <Self as MappedIndex<'idx, IdxTuple>>::size(self)
    }
}

// Recursive MappedIndex implementation for tuples of indices
impl<'idx> MappedIndex<'idx, ()> for CompoundIndex<()> {
    type Value = ();
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> { std::iter::empty() }
    fn to_flat_index(&self, _value: Self::Value) -> usize { 0 }
    fn from_flat_index(&'idx self, _index: usize) -> Self::Value { () }
    fn size(&self) -> usize { 1 }
}

impl<'idx, Head, Tail, IdxHead, IdxTail> MappedIndex<'idx, (IdxHead, IdxTail)> for CompoundIndex<(Head, Tail)>
where
    Head: MappedIndex<'idx, IdxHead> + Eq + PartialEq + Clone,
    Tail: MappedIndex<'idx, IdxTail> + Eq + PartialEq + Clone,
    Head::Value: Copy,
    Tail::Value: Copy,
{
    type Value = (Head::Value, Tail::Value);
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> { std::iter::empty() }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        let (head_idx, tail_idx) = &self.indices;
        let (head_val, tail_val) = value;
        let head_flat = head_idx.to_flat_index(head_val);
        let tail_flat = tail_idx.to_flat_index(tail_val);
        head_flat * tail_idx.size() + tail_flat
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        let (head_idx, tail_idx) = &self.indices;
        let tail_size = tail_idx.size();
        let head_flat = index / tail_size;
        let tail_flat = index % tail_size;
        let head_val = head_idx.from_flat_index(head_flat);
        let tail_val = tail_idx.from_flat_index(tail_flat);
        (head_val, tail_val)
    }
    fn size(&self) -> usize {
        let (head_idx, tail_idx) = &self.indices;
        head_idx.size() * tail_idx.size()
    }
}

// Helper trait to get the Value tuple type for a tuple of indices and their Idx types
pub trait CompoundIndexValue<'idx, IdxTuple> {
    type Value: Tuple;
}

impl<'idx> CompoundIndexValue<'idx, ()> for () {
    type Value = ();
}

impl<'idx, Head, Tail, IdxHead, IdxTail> CompoundIndexValue<'idx, (IdxHead, IdxTail)> for (Head, Tail)
where
    Head: MappedIndex<'idx, IdxHead>,
    Tail: CompoundIndexValue<'idx, IdxTail>,
{
    type Value = (Head::Value, <Tail as CompoundIndexValue<'idx, IdxTail>>::Value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::categorical_index::{CategoricalIndex, CategoricalValue};
    use crate::mapped_index::numeric_range_index::{NumericRangeIndex, NumericValue};
    use crate::mapped_index::MappedIndex;
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct TagA;
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct TagB;
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct TagC;

    #[test]
    fn test_compound_index_get() {
        let cat = CategoricalIndex { values: vec![10, 20], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let compound = CompoundIndex { indices: (cat.clone(), num.clone()) };
        assert_eq!(compound.indices.0, cat);
        assert_eq!(compound.indices.1, num);
    }

    #[test]
    fn test_flat_index_round_trip_2d() {
        let cat = CategoricalIndex { values: vec![10, 20], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let compound = CompoundIndex { indices: (cat.clone(), num.clone()) };
        for i in 0..cat.size() {
            for j in 0..num.size() {
                let val = (cat.from_flat_index(i), num.from_flat_index(j));
                let flat = compound.to_flat_index(val);
                let round = compound.from_flat_index(flat);
                assert_eq!(compound.to_flat_index(round), flat);
                assert_eq!(round, val);
            }
        }
    }

    #[test]
    fn test_size_2d() {
        let cat = CategoricalIndex { values: vec![10, 20, 30], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 4, _phantom: std::marker::PhantomData::<TagB> };
        let compound = CompoundIndex { indices: (cat.clone(), num.clone()) };
        assert_eq!(compound.size(), cat.size() * num.size());
    }

    #[test]
    fn test_flat_index_round_trip_3d() {
        let cat = CategoricalIndex { values: vec![1, 2], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let cat2 = CategoricalIndex { values: vec![5, 6], _phantom: std::marker::PhantomData::<TagC> };
        let inner = CompoundIndex { indices: (num.clone(), cat2.clone()) };
        let compound = CompoundIndex { indices: (cat.clone(), inner) };
        for i in 0..cat.size() {
            for j in 0..num.size() {
                for k in 0..cat2.size() {
                    let val = (cat.from_flat_index(i), (num.from_flat_index(j), cat2.from_flat_index(k)));
                    let flat = compound.to_flat_index(val);
                    let round = compound.from_flat_index(flat);
                    assert_eq!(compound.to_flat_index(round), flat);
                    assert_eq!(round, val);
                }
            }
        }
    }

    #[test]
    fn test_size_3d() {
        let cat = CategoricalIndex { values: vec![1, 2, 3], _phantom: std::marker::PhantomData::<TagA> };
        let num = NumericRangeIndex { start: 0, end: 2, _phantom: std::marker::PhantomData::<TagB> };
        let cat2 = CategoricalIndex { values: vec![5, 6], _phantom: std::marker::PhantomData::<TagC> };
        let inner = CompoundIndex { indices: (num.clone(), cat2.clone()) };
        let compound = CompoundIndex { indices: (cat.clone(), inner) };
        assert_eq!(compound.size(), cat.size() * num.size() * cat2.size());
    }
} 