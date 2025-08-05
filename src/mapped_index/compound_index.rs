#![allow(non_snake_case)]

use crate::mapped_index::MappedIndex;
use crate::tuple_utils::{
    DropFirst, NonEmptyTuple, Prepend, Tuple, TupleAsRefs, TupleCollectOption, TupleFirstElement,
    TuplePrepend,
};

/// An index that combines multiple sub-indices into a compound, multi-dimensional index.
///
/// The flat index is computed by flattening the tuple of indices into a single dimension.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompoundIndex<Indices> {
    /// The tuple of sub-indices.
    pub indices: Indices,
}

impl<Indices> CompoundIndex<Indices> {
    pub const fn new(indices: Indices) -> Self {
        Self { indices }
    }
}

impl<A: MappedIndex> CompoundIndex<(A,)> {
    pub fn collapse_single(self) -> A {
        self.indices.0
    }
}

pub trait IndexRefTuple<'a>: Tuple + Copy {
    type Value: Copy + Tuple;

    fn iter(self) -> impl Iterator<Item = Self::Value> + Clone;

    fn flatten_index_value(self, v: Self::Value) -> usize;

    fn unflatten_index_value(self, index: usize) -> Self::Value;

    fn size(self) -> usize;
}

pub trait IndexRefTupleMinMax<'a>: IndexRefTuple<'a> {
    fn min(self) -> Option<Self::Value>
    where
        Self::Value: Ord;

    fn max(self) -> Option<Self::Value>
    where
        Self::Value: Ord;
}

impl<'a> IndexRefTuple<'a> for () {
    type Value = ();

    fn iter(self) -> impl Iterator<Item = Self::Value> + Clone {
        std::iter::once(())
    }

    fn flatten_index_value(self, _v: Self::Value) -> usize {
        0
    }

    fn unflatten_index_value(self, _index: usize) -> Self::Value {
        ()
    }

    fn size(self) -> usize {
        1
    }
}

impl<'a, B: NonEmptyTuple + TupleFirstElement<First = &'a T> + Copy + 'a, T: MappedIndex + 'a>
    IndexRefTuple<'a> for B
where
    Prepend<<T as MappedIndex>::Value<'a>, <DropFirst<B> as IndexRefTuple<'a>>::Value>:
        Copy
            + TupleFirstElement<
                First = <T as MappedIndex>::Value<'a>,
                Rest = <DropFirst<B> as IndexRefTuple<'a>>::Value,
            >,
    DropFirst<B>: IndexRefTuple<'a>,
{
    type Value = Prepend<T::Value<'a>, <DropFirst<B> as IndexRefTuple<'a>>::Value>;

    fn iter(self) -> impl Iterator<Item = Self::Value> + Clone {
        let (h, t) = TupleFirstElement::split_first(self);
        MappedIndex::iter(h).flat_map(move |hv| t.iter().map(move |tv| tv.prepend(hv)))
    }

    fn flatten_index_value(self, v: Self::Value) -> usize {
        let (h, t) = self.split_first();
        let (vh, vt) = v.split_first();
        let h_flat = h.flatten_index_value(vh);
        let t_flat = t.flatten_index_value(vt);
        h_flat * t.size() + t_flat
    }

    fn unflatten_index_value(self, index: usize) -> Self::Value {
        let (h, t) = self.split_first();
        let h_size = t.size();
        let h_index = index / h_size;
        let t_index = index % h_size;
        let h_value = h.unflatten_index_value(h_index);
        let t_value = t.unflatten_index_value(t_index);
        t_value.prepend(h_value)
    }

    fn size(self) -> usize {
        let (h, t) = self.split_first();
        h.size() * t.size()
    }
}

pub trait IndexTuple: TupleAsRefs {
    type RefTuple<'a>: IndexRefTuple<'a>
    where
        Self: 'a;
    fn as_ref_tuple<'a>(&'a self) -> Self::RefTuple<'a>;
}

impl<Indices> IndexTuple for Indices
where
    Indices: TupleAsRefs,
    for<'a> <Indices as TupleAsRefs>::AsTupleOfRefs<'a>: IndexRefTuple<'a>,
{
    type RefTuple<'a>
        = <Indices as TupleAsRefs>::AsTupleOfRefs<'a>
    where
        Self: 'a;

    fn as_ref_tuple(&self) -> Self::RefTuple<'_> {
        self.as_tuple_of_refs()
    }
}

impl<Indices> MappedIndex for CompoundIndex<Indices>
where
    Indices: IndexTuple,
{
    type Value<'a>
        = <Indices::RefTuple<'a> as IndexRefTuple<'a>>::Value
    where
        Indices: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices.as_ref_tuple().iter()
    }

    fn flatten_index_value<'a>(&'a self, v: Self::Value<'a>) -> usize {
        self.indices.as_ref_tuple().flatten_index_value(v)
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        self.indices.as_ref_tuple().unflatten_index_value(index)
    }

    fn size(&self) -> usize {
        self.indices.as_ref_tuple().size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::{
        categorical_index::CategoricalIndex, numeric_range_index::NumericRangeIndex, MappedIndex,
    };

    #[derive(Debug)]
    struct CatTag;
    #[derive(Debug)]
    struct NumTag;

    #[test]
    fn test_compound_index_size_categorical_numeric() {
        let cat = CategoricalIndex::<i32, CatTag>::new(vec![10, 20]);
        let num = NumericRangeIndex::<i32, NumTag>::new(0, 3);
        let ci = CompoundIndex::new((cat, num));
        assert_eq!(MappedIndex::size(&ci), 2 * 3);
    }

    #[test]
    fn test_compound_index_iter_categorical_numeric() {
        let cat = CategoricalIndex::<i32, CatTag>::new(vec![10, 20]);
        let num = NumericRangeIndex::<i32, NumTag>::new(0, 2);
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
        let num = NumericRangeIndex::<i32, NumTag>::new(0, 3);
        let ci = CompoundIndex::new((cat, num));
        for v in ci.iter() {
            let flat = ci.flatten_index_value(v);
            let round = ci.unflatten_index_value(flat);
            assert_eq!(v, round);
        }
    }
}
