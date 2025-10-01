//! StridedIndexView: an iterator for strided access into indexable data.
use crate::data_frame::core::FrameData;

/// An iterator that yields references to elements in a strided pattern from a base index.
pub struct StridedIndexView<'a, D> {
    base: usize,
    stride: usize,
    pub n_strides: usize,
    view_into: &'a D,
}

impl<'a, D> StridedIndexView<'a, D>
where
    D: FrameData,
{
    /// Creates a new StridedIndexView with the given parameters.
    ///
    /// # Arguments
    /// * `base` - Starting index position
    /// * `stride` - Number of elements to skip between each access
    /// * `n_strides` - Number of elements to yield
    /// * `view_into` - Reference to the data being viewed
    pub fn new(base: usize, stride: usize, n_strides: usize, view_into: &'a D) -> Self {
        // Check that the indices produced by the view are within bounds.
        if n_strides > 0 {
            let max_index = base + stride * (n_strides - 1);
            assert!(max_index < view_into.len());
        }

        Self {
            base,
            stride,
            n_strides,
            view_into,
        }
    }

    pub fn len(&self) -> usize {
        self.n_strides
    }
}

impl<'a, D> Iterator for StridedIndexView<'a, D>
where
    D: FrameData,
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
    D: FrameData,
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
