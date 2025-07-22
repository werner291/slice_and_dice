use crate::mapped_index::MappedIndex;

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
    pub fn new(indices: Indices) -> Self {
        Self { indices }
    }
}

impl<A, B> MappedIndex for CompoundIndex<(A, B)>
where
    A: MappedIndex,
    B: MappedIndex,
{
    type Value<'a>
        = (A::Value<'a>, B::Value<'a>)
    where
        A: 'a,
        B: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let (ia, ib) = &self.indices;
        itertools::iproduct!(ia.iter(), ib.iter())
    }

    fn flatten_index_value(&self, (va, vb): Self::Value<'_>) -> usize {
        let (ia, ib) = &self.indices;
        ia.flatten_index_value(va) * ib.size() + ib.flatten_index_value(vb)
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        let (ia, ib) = &self.indices;

        (
            ia.unflatten_index_value(index / ib.size()),
            ib.unflatten_index_value(index % ib.size()),
        )
    }
    fn size(&self) -> usize {
        let (ia, ib) = &self.indices;
        ia.size() * ib.size()
    }
}
