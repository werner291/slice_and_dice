//! StridedIndexView: an iterator for strided access into indexable data.
use std::ops::Index;

/// An iterator that yields references to elements in a strided pattern from a base index.
pub struct StridedIndexView<'a, D: Index<usize>> {
    pub(crate) base: usize,
    pub(crate) stride: usize,
    pub(crate) n_strides: usize,
    pub(crate) view_into: &'a D,
}

impl<'a, D> Iterator for StridedIndexView<'a, D>
where
    D: Index<usize>,
{
    type Item = &'a D::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n_strides == 0 {
            None
        } else {
            let item = &self.view_into[self.base];
            self.base += self.stride;
            self.n_strides -= 1;
            Some(item)
        }
    }
}

impl<'a, D> ExactSizeIterator for StridedIndexView<'a, D>
where
    D: Index<usize>,
{
    fn len(&self) -> usize {
        self.n_strides
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strided_index_view() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let view = StridedIndexView {
            base: 1,
            stride: 3,
            n_strides: 3,
            view_into: &data,
        };
        let collected: Vec<_> = view.collect();
        assert_eq!(collected, vec![&1, &4, &7]);
    }
}
