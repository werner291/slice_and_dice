#![allow(non_snake_case)]

use crate::mapped_index::MappedIndex;
use crate::mapped_index::tuple_utils::{TupleAsRefs, TupleCons, TupleHead};
use itertools::iproduct;
use tuple_list::AsTupleOfRefs;

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

pub trait IndexRefTuple<'a> {
    type Value: Copy;

    fn iter(self) -> impl Iterator<Item = Self::Value> + Clone;

    fn flatten_index_value(self, v: Self::Value) -> usize;

    fn unflatten_index_value(self, index: usize) -> Self::Value;

    fn size(self) -> usize;
}

impl<'a, A: MappedIndex + 'a, B: MappedIndex + 'a> IndexRefTuple<'a> for (&'a A, &'a B) {
    type Value = (A::Value<'a>, B::Value<'a>);

    fn iter(self) -> impl Iterator<Item = Self::Value> + Clone {
        iproduct!(self.0.iter(), self.1.iter())
    }

    fn flatten_index_value(self, v: Self::Value) -> usize {
        let a_flat = self.0.flatten_index_value(v.0);
        let b_flat = self.1.flatten_index_value(v.1);
        a_flat * self.1.size() + b_flat
    }

    fn unflatten_index_value(self, index: usize) -> Self::Value {
        let a_size = self.1.size();
        let a_index = index / a_size;
        let b_index = index % a_size;
        let a = self.0.unflatten_index_value(a_index);
        let b = self.1.unflatten_index_value(b_index);
        (a, b)
    }

    fn size(self) -> usize {
        self.0.size() * self.1.size()
    }
}

macro_rules! impl_index_ref_tuple {
    (($head:ident, $($tail:ident),*)) => {
        impl<'a, $head: MappedIndex, $($tail: MappedIndex),*> IndexRefTuple<'a> for (&'a $head, $(&'a $tail),*) {
            type Value = <($($tail::Value<'a> ),*) as TupleCons>::TupleCons<$head::Value<'a>>;

            fn iter(self) -> impl Iterator<Item = Self::Value> + Clone {
                let (h, t) = TupleHead::split_head(self);
                iproduct!(h.iter(), t.iter())
                    .map(|(hv, tv)| tv.prepend(hv))
            }

            fn flatten_index_value(self, v: Self::Value) -> usize {
                let (h, t) = self.split_head();
                let h_flat = h.flatten_index_value(v.split_head().0);
                let t_flat = t.flatten_index_value(v.split_head().1);
                h_flat * t.size() + t_flat
            }

            fn unflatten_index_value(self, index: usize) -> Self::Value {
                let (h, t) = self.split_head();
                let h_size = t.size();
                let h_index = index / h_size;
                let t_index = index % h_size;
                let h_value = h.unflatten_index_value(h_index);
                let t_value = t.unflatten_index_value(t_index);
                t_value.prepend(h_value)
            }

            fn size(self) -> usize {
                let (h, t) = self.split_head();
                h.size() * t.size()
            }
        }

    };
}

impl_index_ref_tuple!((A, B, C));

impl<Indices> MappedIndex for CompoundIndex<Indices>
where
    Indices: TupleAsRefs,
    for<'a> <Indices as TupleAsRefs>::AsTupleOfRefs<'a>: IndexRefTuple<'a>,
{
    type Value<'a>
        = <<Indices as TupleAsRefs>::AsTupleOfRefs<'a> as IndexRefTuple<'a>>::Value
    where
        Indices: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices.as_tuple_of_refs().iter()
    }

    fn flatten_index_value<'a>(&'a self, v: Self::Value<'a>) -> usize {
        let refs = self.indices.as_tuple_of_refs();
        refs.flatten_index_value(v)
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        self.indices.as_tuple_of_refs().unflatten_index_value(index)
    }

    fn size(&self) -> usize {
        self.indices.as_tuple_of_refs().size()
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
        assert_eq!(MappedIndex::size(&ci), 2 * 3);
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
