use super::VariableRange;

/// A one-to-many range that associates each element of a left-hand range
/// with a corresponding range from a vector, then flattens the associated
/// right-hand ranges in the order of the left-hand elements.
///
/// In other words, for each left index i, we take `rights[i]` and append all
/// of its values to the resulting sequence. The total size is the sum of the
/// sizes of all right-hand ranges.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct OneToManyRange<L: VariableRange, R: VariableRange> {
    /// The left-hand range used to define association and order.
    pub left: L,
    /// The right-hand ranges, one per element of `left`.
    pub rights: Vec<R>,
}

impl<L: VariableRange, R: VariableRange> OneToManyRange<L, R> {
    /// Create a new OneToManyRange from a left-hand range and a vector of right-hand ranges.
    ///
    /// Panics if `rights.len()` is not equal to `left.size()`.
    pub fn new(left: L, rights: Vec<R>) -> Self {
        assert_eq!(
            rights.len(),
            left.size(),
            "rights length must match left size"
        );
        Self { left, rights }
    }

    /// Number of associations (i.e., size of the left-hand range).
    pub fn associations(&self) -> usize {
        self.left.size()
    }

    /// Number of right-hand ranges.
    pub fn len(&self) -> usize {
        self.rights.len()
    }

    /// Returns true if there are no right-hand ranges (which implies empty left).
    pub fn is_empty(&self) -> bool {
        self.rights.is_empty()
    }
}

impl<L: VariableRange, R: VariableRange> VariableRange for OneToManyRange<L, R> {
    type Value<'a>
        = R::Value<'a>
    where
        R: 'a,
        L: 'a;

    /// Iterate over all values in all associated right-hand ranges in order of the left-hand range.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        let total = self.size();
        (0..total).map(move |i| self.unflatten_index_value(i))
    }

    /// Return the value corresponding to a flat index by scanning through the right-hand ranges.
    fn unflatten_index_value(&self, mut index: usize) -> Self::Value<'_> {
        for r in &self.rights {
            let sz = r.size();
            if index < sz {
                return r.unflatten_index_value(index);
            }
            index -= sz;
        }
        panic!("Index out of bounds: {} (size: {})", index, self.size());
    }

    /// Sum of sizes of all right-hand ranges.
    fn size(&self) -> usize {
        self.rights.iter().map(|r| r.size()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use crate::mapped_index::singleton_index::{CopySingletonRange, SingletonRange};

    #[test]
    fn test_one_to_many_size_and_iteration() {
        // left has size 2
        let left = NumericRangeIndex::new(0usize, 2usize); // 0,1
        let r0 = NumericRangeIndex::new(10usize, 12usize); // 10,11
        let r1 = NumericRangeIndex::new(20usize, 23usize); // 20,21,22
        let otm = OneToManyRange::new(left, vec![r0, r1]);

        assert_eq!(otm.associations(), 2);
        assert_eq!(otm.size(), 5);
        let vals: Vec<_> = otm.iter().collect();
        assert_eq!(vals, vec![10usize, 11, 20, 21, 22]);
    }

    #[test]
    fn test_one_to_many_unflatten() {
        let left = NumericRangeIndex::new(5usize, 8usize); // size 3
        let r0 = NumericRangeIndex::new(0usize, 1usize); // 0
        let r1 = NumericRangeIndex::new(30usize, 32usize); // 30,31
        let r2 = NumericRangeIndex::new(40usize, 43usize); // 40,41,42
        let otm = OneToManyRange::new(left, vec![r0, r1, r2]);

        assert_eq!(otm.size(), 6);
        assert_eq!(otm.unflatten_index_value(0), 0);
        assert_eq!(otm.unflatten_index_value(1), 30);
        assert_eq!(otm.unflatten_index_value(2), 31);
        assert_eq!(otm.unflatten_index_value(3), 40);
        assert_eq!(otm.unflatten_index_value(4), 41);
        assert_eq!(otm.unflatten_index_value(5), 42);
    }

    #[test]
    #[should_panic]
    fn test_one_to_many_out_of_bounds() {
        let left = NumericRangeIndex::new(0usize, 1usize);
        let r0 = NumericRangeIndex::new(0usize, 0usize); // empty
        let otm = OneToManyRange::new(left, vec![r0]);
        // size is 0, index 0 should panic
        let _ = otm.unflatten_index_value(0);
    }

    #[test]
    fn test_one_to_many_with_ref_values() {
        let left = NumericRangeIndex::new(0usize, 2usize); // size 2
        let a = SingletonRange::new(String::from("a"));
        let b = SingletonRange::new(String::from("b"));
        let otm = OneToManyRange::new(left, vec![a, b]);
        assert_eq!(otm.size(), 2);
        let vals: Vec<&String> = otm.iter().collect();
        assert_eq!(vals[0], "a");
        assert_eq!(vals[1], "b");
    }

    #[test]
    fn test_one_to_many_with_copy_singleton() {
        let left = NumericRangeIndex::new(0usize, 2usize);
        let a = CopySingletonRange::new(42u32);
        let b = CopySingletonRange::new(7u32);
        let otm = OneToManyRange::new(left, vec![a, b]);

        assert_eq!(otm.size(), 2);
        let vals: Vec<u32> = otm.iter().collect();
        assert_eq!(vals, vec![42, 7]);
    }

    #[test]
    #[should_panic(expected = "rights length must match left size")]
    fn test_one_to_many_mismatched_lengths() {
        let left = NumericRangeIndex::new(0usize, 2usize); // size 2
        let r0 = NumericRangeIndex::new(0usize, 1usize);
        // Provide only one right range; should panic
        let _ = OneToManyRange::new(left, vec![r0]);
    }
}
