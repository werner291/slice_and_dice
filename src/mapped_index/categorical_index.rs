use std::marker::PhantomData;
use super::MappedIndex;

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
    _phantom: PhantomData<Tag>,
}

impl<'idx, T: Copy + 'idx, Tag: 'idx> MappedIndex<'idx, usize> for CategoricalIndex<T, Tag> {
    type Value = CategoricalValue<'idx, T, Tag>;
    fn get(&'idx self, index: usize) -> Option<Self::Value> {
        self.values.get(index).map(|v| CategoricalValue { value: v, index, _phantom: PhantomData })
    }
    fn iter(&'idx self) -> impl Iterator<Item = Self::Value> {
        self.values.iter().enumerate().map(move |(index, v)| CategoricalValue { value: v, index, _phantom: PhantomData })
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
    fn test_get_and_at() {
        let index = CategoricalIndex { values: vec![1, 2, 3], _phantom: PhantomData::<Tag> };
        let cat_val = index.get(1).unwrap();
        assert_eq!(*cat_val.value, 2);
        assert_eq!(index.at(cat_val), &2);
    }

    #[test]
    #[should_panic]
    fn test_at_out_of_bounds() {
        let index = CategoricalIndex { values: vec![1, 2, 3], _phantom: PhantomData::<Tag> };
        let cat_val = CategoricalValue { value: &0, index: 10, _phantom: PhantomData::<&Tag> };
        let _ = index.at(cat_val);
    }
} 