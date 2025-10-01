use super::VariableRange;

/// A range that is the concatenation (union) of multiple other ranges
/// with the same value type.
///
/// The union is evaluated in order: the first range comes first, then the
/// second, and so on. Sizes are summed. Indexing is delegated to the
/// appropriate inner range based on cumulative sizes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UnionRange<R: VariableRange> {
    /// The inner ranges that form the union.
    pub ranges: Vec<R>,
}

impl<R: VariableRange> UnionRange<R> {
    /// Create a new UnionRange from a vector of ranges.
    pub const fn new(ranges: Vec<R>) -> Self {
        Self { ranges }
    }

    /// Push a new range to the end of the union.
    pub fn push(&mut self, range: R) {
        self.ranges.push(range);
    }

    /// Number of inner ranges.
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Returns true if there are no inner ranges.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

impl<R: VariableRange> VariableRange for UnionRange<R> {
    type Value<'a>
        = R::Value<'a>
    where
        R: 'a;

    /// Iterate over all values in all inner ranges in order.
    fn iter(&self) -> impl Iterator<Item = Self::Value<'_>> + Clone {
        // Use a simple map over indices to ensure the iterator is Clone.
        // Range<usize> is Clone and the closure captures &self, which is Copy/Clone,
        // therefore the resulting Map is Clone.
        let total = self.size();
        (0..total).map(move |i| self.unflatten_index_value(i))
    }

    /// Find the appropriate inner range based on cumulative sizes and
    /// return the corresponding value.
    fn unflatten_index_value(&self, mut index: usize) -> Self::Value<'_> {
        for r in &self.ranges {
            let sz = r.size();
            if index < sz {
                return r.unflatten_index_value(index);
            }
            index -= sz;
        }
        panic!("Index out of bounds: {} (size: {})", index, self.size());
    }

    /// Sum of sizes of all inner ranges.
    fn size(&self) -> usize {
        self.ranges.iter().map(|r| r.size()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range::NumericRangeIndex;
    use crate::mapped_index::singleton_index::{CopySingletonRange, SingletonRange};

    #[test]
    fn test_union_size_and_iteration() {
        let r1 = NumericRangeIndex::new(0usize, 3usize); // 0,1,2
        let r2 = NumericRangeIndex::new(10usize, 12usize); // 10,11
        let union = UnionRange::new(vec![r1, r2]);

        assert_eq!(union.size(), 5);
        let vals: Vec<_> = union.iter().collect();
        assert_eq!(vals, vec![0usize, 1, 2, 10, 11]);
    }

    #[test]
    fn test_union_unflatten() {
        let r1 = NumericRangeIndex::new(5usize, 7usize); // 5,6
        let r2 = NumericRangeIndex::new(0usize, 1usize); // 0
        let r3 = NumericRangeIndex::new(20usize, 23usize); // 20,21,22
        let union = UnionRange::new(vec![r1, r2, r3]);

        assert_eq!(union.size(), 6);
        assert_eq!(union.unflatten_index_value(0), 5);
        assert_eq!(union.unflatten_index_value(1), 6);
        assert_eq!(union.unflatten_index_value(2), 0);
        assert_eq!(union.unflatten_index_value(3), 20);
        assert_eq!(union.unflatten_index_value(4), 21);
        assert_eq!(union.unflatten_index_value(5), 22);
    }

    #[test]
    #[should_panic]
    fn test_union_out_of_bounds() {
        let r1 = NumericRangeIndex::new(0usize, 1usize);
        let union = UnionRange::new(vec![r1]);
        // size is 1, index 1 should panic
        let _ = union.unflatten_index_value(1);
    }

    #[test]
    fn test_union_with_ref_values() {
        // Ensure it works with ranges that yield references as values.
        let a = SingletonRange::new(String::from("a"));
        let b = SingletonRange::new(String::from("b"));
        let union = UnionRange::new(vec![a, b]);

        assert_eq!(union.size(), 2);
        let vals: Vec<&String> = union.iter().collect();
        assert_eq!(vals[0], "a");
        assert_eq!(vals[1], "b");
    }

    #[test]
    fn test_union_with_copy_singleton() {
        let a = CopySingletonRange::new(42u32);
        let b = CopySingletonRange::new(7u32);
        let union = UnionRange::new(vec![a, b]);

        assert_eq!(union.size(), 2);
        let vals: Vec<u32> = union.iter().collect();
        assert_eq!(vals, vec![42, 7]);
    }
}
