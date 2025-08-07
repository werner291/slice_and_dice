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
        self.head
            .iter()
            .zip(self.tail.iter())
            .map(|(head, tail)| h_cons(head, tail))
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
