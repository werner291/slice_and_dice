//! Core DataFrame struct and basic methods.
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::CompoundIndex;
use frunk::{HCons, HList, HNil};
use std::ops::Index;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DataFrame<I, D>
where
    I: VariableRange,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
}

impl<I, D> DataFrame<I, D>
where
    I: VariableRange,
    D: Index<usize> + IntoIterator,
{
    pub const fn new(index: I, data: D) -> Self {
        Self { index, data }
    }

    pub fn iter(&self) -> impl Iterator<Item = (I::Value<'_>, &D::Output)> + '_ {
        self.index
            .iter()
            .enumerate()
            .map(|(i, v)| (v, &self.data[i]))
    }
}

impl<I, T> DataFrame<I, Vec<T>>
where
    I: VariableRange + Clone,
{
    pub fn map<U, F>(&self, mut f: F) -> DataFrame<I, Vec<U>>
    where
        F: FnMut(&T) -> U,
    {
        let data = self.data.iter().map(|v| f(v)).collect();
        DataFrame {
            index: self.index.clone(),
            data,
        }
    }

    pub fn build_from_index<F>(index: &I, mut f: F) -> DataFrame<I, Vec<T>>
    where
        F: FnMut(I::Value<'_>) -> T,
    {
        let data = index.iter().map(|v| f(v)).collect();
        DataFrame {
            index: index.clone(),
            data,
        }
    }
}

impl<I, D> DataFrame<CompoundIndex<HList![I]>, D>
where
    I: VariableRange + 'static,
    D: Index<usize>,
{
    pub fn collapse_single_index(self) -> DataFrame<I, D> {
        DataFrame {
            index: self.index.indices.head,
            data: self.data,
        }
    }
}
