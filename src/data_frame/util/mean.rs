/// Trait to compute the mean of a sequence of values.
///
/// Implementors define how to compute the mean from an iterator of references to values.
/// Returns None if the iterator is empty.
pub trait Mean: Sized {
    /// Compute the mean of the given iterator of references to values.
    fn mean_from_iter<'a, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = &'a Self>,
        Self: 'a;
}

impl Mean for f64 {
    fn mean_from_iter<'a, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = &'a Self>,
        Self: 'a,
    {
        let mut iter = iter.into_iter();
        let first = *iter.next()?;
        let mut sum = first;
        let mut count: usize = 1;
        for v in iter {
            sum += *v;
            count += 1;
        }
        Some(sum / (count as f64))
    }
}

impl Mean for f32 {
    fn mean_from_iter<'a, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = &'a Self>,
        Self: 'a,
    {
        let mut iter = iter.into_iter();
        let first = *iter.next()?;
        let mut sum = first;
        let mut count: usize = 1;
        for v in iter {
            sum += *v;
            count += 1;
        }
        Some(sum / (count as f32))
    }
}
