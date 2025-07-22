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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::{
        MappedIndex, categorical_index::CategoricalIndex, numeric_range_index::NumericRangeIndex,
    };

    #[derive(Debug)]
    struct CatTag;
    #[derive(Debug)]
    struct NumTag;

    #[test]
    fn test_compound_index_size_categorical_numeric() {
        let cat = CategoricalIndex::<i32, CatTag>::new(vec![10, 20]);
        let num = NumericRangeIndex::<NumTag>::new(0, 3);
        let ci = CompoundIndex::new((cat, num));
        assert_eq!(ci.size(), 2 * 3);
    }

    #[test]
    fn test_compound_index_iter_categorical_numeric() {
        let cat = CategoricalIndex::<i32, CatTag>::new(vec![10, 20]);
        let num = NumericRangeIndex::<NumTag>::new(0, 2);
        let ci = CompoundIndex::new((cat.clone(), num.clone()));
        let items: Vec<_> = ci.iter().collect();
        assert_eq!(items.len(), 4);
        let cat_vals: Vec<_> = cat.iter().collect();
        let num_vals: Vec<_> = num.iter().collect();
        assert!(items.contains(&(cat_vals[0], num_vals[0])));
        assert!(items.contains(&(cat_vals[1], num_vals[1])));
    }

    #[test]
    fn test_flatten_unflatten_round_trip_categorical_numeric() {
        let cat = CategoricalIndex::<i32, CatTag>::new(vec![10, 20]);
        let num = NumericRangeIndex::<NumTag>::new(0, 3);
        let ci = CompoundIndex::new((cat, num));
        for v in ci.iter() {
            let flat = ci.flatten_index_value(v);
            let round = ci.unflatten_index_value(flat);
            assert_eq!(v, round);
        }
    }
}
