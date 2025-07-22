use std::marker::PhantomData;
use super::MappedIndex;

#[derive(Debug, PartialEq, Eq)]
pub struct CategoricalValue<'idx, T, Tag> {
    pub value: &'idx T,
    index: usize,
    _phantom: PhantomData<&'idx Tag>,
}

impl<'idx, T: Copy, Tag> Copy for CategoricalValue<'idx, T, Tag> {}
impl<'idx, T: Copy, Tag> Clone for CategoricalValue<'idx, T, Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct CategoricalIndex<T, Tag> {
    pub values: Vec<T>,
    pub _phantom: PhantomData<Tag>,
}

impl<'idx, T: Copy + 'idx, Tag: 'idx> MappedIndex<'idx, usize> for CategoricalIndex<T, Tag> {
    type Value = CategoricalValue<'idx, T, Tag>;
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        self.values.iter().enumerate().map(move |(index, v)| CategoricalValue { value: v, index, _phantom: PhantomData })
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        value.index
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        CategoricalValue { value: &self.values[index], index, _phantom: PhantomData }
    }
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl<'idx, T: Copy + 'idx, Tag: 'idx> MappedIndex<'idx, usize> for &CategoricalIndex<T, Tag> {
    type Value = <CategoricalIndex<T, Tag> as MappedIndex<'idx, usize>>::Value;
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        (*self).iter()
    }
    fn to_flat_index(&self, value: Self::Value) -> usize {
        (*self).to_flat_index(value)
    }
    fn from_flat_index(&'idx self, index: usize) -> Self::Value {
        (*self).from_flat_index(index)
    }
    fn size(&self) -> usize {
        (*self).size()
    }
}

impl<T: Copy, Tag> CategoricalIndex<T, Tag> {
    pub fn at<'idx>(&'idx self, cat_value: CategoricalValue<'idx, T, Tag>) -> &'idx T {
        &self.values[cat_value.index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::MappedIndex;

    struct Tag;

    #[test]
    fn test_flat_index_round_trip() {
        let index = CategoricalIndex { values: vec![1, 2, 3], _phantom: PhantomData::<Tag> };
        let cat_val = index.from_flat_index(2);
        let flat = index.to_flat_index(cat_val);
        let round = index.from_flat_index(flat);
        assert_eq!(cat_val.index, round.index);
        assert_eq!(*cat_val.value, *round.value);
    }
} 