#![allow(non_snake_case)]

use crate::mapped_index::VariableRange;
use frunk::hlist::{HList, h_cons};
use frunk::indices::{Here, There};
use frunk::{HCons, HNil};

pub type Dim0 = Here;
pub type Dim1 = There<Dim0>;
pub type Dim2 = There<Dim1>;
pub type Dim3 = There<Dim2>;
pub type Dim4 = There<Dim3>;

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

impl<A: VariableRange> CompoundIndex<(A,)> {
    pub fn collapse_single(self) -> A {
        self.indices.0
    }
}

pub trait IndexHlist: HList + Sync + Clone {
    type Value<'a>: Copy + HList
    where
        Self: 'a;

    type Refs<'a>: RefIndexHList
    where
        Self: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone;

    fn refs(&self) -> Self::Refs<'_>;

    fn size(&self) -> usize;

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_>;
}

pub trait RefIndexHList: HList + Copy {
    fn size(self) -> usize;
}

impl RefIndexHList for HNil {
    fn size(self) -> usize {
        1
    }
}

impl<'a, Head: VariableRange, Tail: RefIndexHList> RefIndexHList for HCons<&'a Head, Tail> {
    fn size(self) -> usize {
        self.head.size() * self.tail.size()
    }
}

impl<Head: VariableRange, Tail: IndexHlist> IndexHlist for HCons<Head, Tail> {
    type Value<'a>
        = HCons<<Head as VariableRange>::Value<'a>, <Tail as IndexHlist>::Value<'a>>
    where
        Self: 'a;

    type Refs<'a>
        = HCons<&'a Head, <Tail as IndexHlist>::Refs<'a>>
    where
        Self: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let head_iter = self.head.iter();

        head_iter.flat_map(move |head| self.tail.iter().map(move |tail| h_cons(head, tail)))
    }

    fn refs(&self) -> Self::Refs<'_> {
        h_cons(&self.head, self.tail.refs())
    }

    fn size(&self) -> usize {
        self.head.size() * self.tail.size()
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        let head = self.head.unflatten_index_value(index / self.tail.size());
        let tail_index = index % self.tail.size();
        h_cons(head, self.tail.unflatten_index_value(tail_index))
    }
}
impl IndexHlist for HNil {
    type Value<'a> = HNil;
    type Refs<'a>
        = HNil
    where
        Self: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        std::iter::once(HNil)
    }

    fn refs(&self) -> Self::Refs<'_> {
        HNil
    }

    fn size(&self) -> usize {
        1
    }

    fn unflatten_index_value(&self, _: usize) -> Self::Value<'_> {
        HNil
    }
}

pub trait HListExt: HList {}

pub trait PluckSplit<At>: HList {
    type Left: HList;
    type Extract;
    type Right: HList;

    fn pluck_split(self) -> (Self::Left, Self::Extract, Self::Right);
}

pub type HLConcat<A, B> = <A as HListConcat<B>>::Concat;

pub trait HListConcat<Other: HList>: HList {
    type Concat: HList;

    fn concat(self, other: Other) -> Self::Concat;
}

impl<Other: HList> HListConcat<Other> for HNil {
    type Concat = Other;

    fn concat(self, other: Other) -> Self::Concat {
        other
    }
}

impl<Head, Tail, Other: HList> HListConcat<Other> for HCons<Head, Tail>
where
    Tail: HListConcat<Other>,
{
    type Concat = HCons<Head, Tail::Concat>;

    fn concat(self, other: Other) -> Self::Concat {
        h_cons(self.head, self.tail.concat(other))
    }
}

impl<Head, Tail> PluckSplit<Here> for HCons<Head, Tail>
where
    Tail: HList,
{
    type Left = HNil;
    type Extract = Head;
    type Right = Tail;

    fn pluck_split(self) -> (Self::Left, Self::Extract, Self::Right) {
        (HNil, self.head, self.tail)
    }
}

impl<Head, Tail: PluckSplit<ThereTail>, ThereTail> PluckSplit<There<ThereTail>>
    for HCons<Head, Tail>
where
    Tail: HList,
{
    type Left = HCons<Head, Tail::Left>;
    type Extract = Tail::Extract;
    type Right = Tail::Right;

    fn pluck_split(self) -> (Self::Left, Self::Extract, Self::Right) {
        let (l, e, r) = self.tail.pluck_split();
        (h_cons(self.head, l), e, r)
    }
}

impl<Indices: IndexHlist> VariableRange for CompoundIndex<Indices> {
    type Value<'a>
        = <Indices as IndexHlist>::Value<'a>
    where
        Indices: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        self.indices.iter()
    }

    fn unflatten_index_value(&self, index: usize) -> Self::Value<'_> {
        self.indices.unflatten_index_value(index)
    }

    fn size(&self) -> usize {
        self.indices.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::VariableRange;
    use crate::mapped_index::categorical_index::{CategoricalRange, SliceCategoricalIndex};
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use crate::mapped_index::singleton_index::SingletonRange;
    use frunk::hlist::HNil;

    #[test]
    fn test_compound_index_size() {
        // Test with a single index
        let singleton = SingletonRange::new(42);
        let indices = h_cons(singleton.clone(), HNil);
        let compound_single = CompoundIndex::new(indices);
        assert_eq!(compound_single.size(), 1);

        // Test with two indices
        let categorical = CategoricalRange::new(vec![1, 2, 3]);
        let indices = h_cons(singleton.clone(), h_cons(categorical.clone(), HNil));
        let compound_two = CompoundIndex::new(indices);
        assert_eq!(compound_two.size(), 3); // 1 * 3 = 3

        // Test with three indices
        let categorical2 = CategoricalRange::new(vec!["a", "b"]);
        let indices = h_cons(singleton, h_cons(categorical, h_cons(categorical2, HNil)));
        let compound_three = CompoundIndex::new(indices);
        assert_eq!(compound_three.size(), 6); // 1 * 3 * 2 = 6
    }

    #[test]
    fn test_compound_index_iteration() {
        // Test with a single index
        let singleton = SingletonRange::new(42);
        let indices = h_cons(singleton.clone(), HNil);
        let compound_single = CompoundIndex::new(indices);

        let values: Vec<_> = compound_single.iter().collect();
        assert_eq!(values.len(), 1);
        assert_eq!(*values[0].head, 42);

        // Test with two indices
        // With the fixed implementation, we get all combinations of values
        // For a singleton (1 value) and a categorical index (3 values), we get 3 values
        let categorical = CategoricalRange::new(vec![1, 2, 3]);
        let indices = h_cons(singleton, h_cons(categorical, HNil));
        let compound_two = CompoundIndex::new(indices);

        let values: Vec<_> = compound_two.iter().collect();
        assert_eq!(values.len(), 3); // 1 * 3 = 3

        // Check the values
        assert_eq!(*values[0].head, 42); // First index value
        assert_eq!(*values[0].tail.head, 1); // First combination

        assert_eq!(*values[1].head, 42); // First index value
        assert_eq!(*values[1].tail.head, 2); // Second combination

        assert_eq!(*values[2].head, 42); // First index value
        assert_eq!(*values[2].tail.head, 3); // Third combination
    }

    #[test]
    fn test_compound_index_unflatten() {
        // Test with two indices
        let singleton = SingletonRange::new(42);
        let categorical = CategoricalRange::new(vec![1, 2, 3]);
        let indices = h_cons(singleton, h_cons(categorical, HNil));
        let compound = CompoundIndex::new(indices);

        // Check unflatten_index_value for each index
        let value0 = compound.unflatten_index_value(0);
        assert_eq!(*value0.head, 42);
        assert_eq!(*value0.tail.head, 1);

        let value1 = compound.unflatten_index_value(1);
        assert_eq!(*value1.head, 42);
        assert_eq!(*value1.tail.head, 2);

        let value2 = compound.unflatten_index_value(2);
        assert_eq!(*value2.head, 42);
        assert_eq!(*value2.tail.head, 3);
    }

    #[test]
    #[should_panic]
    fn test_compound_index_unflatten_out_of_bounds() {
        let singleton = SingletonRange::new(42);
        let categorical = CategoricalRange::new(vec![1, 2, 3]);
        let indices = h_cons(singleton, h_cons(categorical, HNil));
        let compound = CompoundIndex::new(indices);

        // This should panic because the index is out of bounds
        compound.unflatten_index_value(3);
    }

    #[test]
    fn test_compound_index_iteration_issue() {
        // This test reproduces the issue described in the problem statement
        // Create a SliceCategoricalIndex with two values
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct UseFilter(bool);

        let use_filters = [UseFilter(false), UseFilter(true)];
        let solvers_vars = SliceCategoricalIndex::new(&use_filters);

        // Create a NumericRangeIndex with multiple values
        let tree_names_range = NumericRangeIndex::new(0, 3);

        // Create a CompoundIndex with both indices
        let variables = CompoundIndex::new(h_cons(solvers_vars, h_cons(tree_names_range, HNil)));

        // Check the size
        assert_eq!(variables.size(), 6); // 2 * 3 = 6

        // Collect all values from the iterator
        let values: Vec<_> = variables.iter().collect();

        // With the old implementation, we would only get one value due to zip()
        // assert_eq!(values.len(), 1);

        // With the fixed implementation, we should get 6 values (2 * 3)
        assert_eq!(values.len(), 6);
    }
}
