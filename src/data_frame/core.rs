//! Core DataFrame struct and basic methods.
use crate::mapped_index::VariableRange;
use crate::mapped_index::compound_index::CompoundIndex;
use frunk::{HCons, HList, HNil};
use std::ops::Index;

pub trait FrameData: Index<usize> {
    fn len(&self) -> usize;

    fn iter(&self) -> impl Iterator<Item = &Self::Output> + '_ {
        (0..self.len()).map(|i| &self[i])
    }
}

/// Macro to allow direct field access for tests and internal code.
/// This is used to avoid having to update all the direct field accesses in the codebase.
#[macro_export]
macro_rules! allow_direct_field_access {
    () => {
        #[cfg(any(test, feature = "internal"))]
        pub struct DataFrame<I, D>
        where
            I: VariableRange,
            D: FrameData,
        {
            /// The index structure (categorical, numeric, compound, etc.).
            pub index: I,
            /// The data collection, indexable by flat index.
            pub data: D,
        }

        #[cfg(not(any(test, feature = "internal")))]
        pub struct DataFrame<I, D>
        where
            I: VariableRange,
            D: FrameData,
        {
            /// The index structure (categorical, numeric, compound, etc.).
            index: I,
            /// The data collection, indexable by flat index.
            data: D,
        }
    };
}

impl<D> FrameData for Vec<D> {
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    /// The index structure (categorical, numeric, compound, etc.).
    #[cfg(test)]
    pub index: I,
    #[cfg(not(test))]
    index: I,
    /// The data collection, indexable by flat index.
    #[cfg(test)]
    pub data: D,
    #[cfg(not(test))]
    data: D,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::numeric_range::NumericRangeIndex;

    #[test]
    fn test_dataframe_constructor_validation() {
        // Test with matching lengths
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        assert_eq!(df.index().size(), 3);
        assert_eq!(df.data().len(), 3);

        // Test with mismatched lengths - should panic
        let index = NumericRangeIndex::<i32>::new(0, 3); // [0, 1, 2]
        let data = vec![10, 20]; // Only 2 elements
        let result = std::panic::catch_unwind(|| {
            DataFrame::new(index, data);
        });
        assert!(
            result.is_err(),
            "DataFrame::new should panic when index and data lengths don't match"
        );
    }
}

impl<I, D> DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    pub fn new(index: I, data: D) -> Self {
        assert_eq!(
            index.size(),
            data.len(),
            "Index and data must have the same length"
        );
        Self { index, data }
    }

    /// Returns a reference to the index.
    pub fn index(&self) -> &I {
        &self.index
    }

    /// Returns a reference to the data.
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Returns a mutable reference to the data.
    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }

    /// Returns a reference to the data at the given index.
    pub fn data_at(&self, index: usize) -> &D::Output {
        &self.data[index]
    }

    /// Returns a reference to the internal data structure.
    /// This is for internal use only and should not be used by external code.
    #[doc(hidden)]
    pub fn internal_data(&self) -> &D {
        &self.data
    }

    /// Returns a reference to the internal index structure.
    /// This is for internal use only and should not be used by external code.
    #[doc(hidden)]
    pub fn internal_index(&self) -> &I {
        &self.index
    }

    pub fn iter(&self) -> impl Iterator<Item = (I::Value<'_>, &D::Output)> + '_ {
        self.index.iter().zip(self.data.iter())
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
        let data = self.data().iter().map(|v| f(v)).collect();
        DataFrame::new(self.index().clone(), data)
    }

    pub fn build_from_index<F>(index: &I, mut f: F) -> DataFrame<I, Vec<T>>
    where
        F: FnMut(I::Value<'_>) -> T,
    {
        let data = index.iter().map(|v| f(v)).collect();
        DataFrame::new(index.clone(), data)
    }
}

impl<I, D> DataFrame<CompoundIndex<HList![I]>, D>
where
    I: VariableRange + 'static,
    D: FrameData,
{
    pub fn collapse_single_index(self) -> DataFrame<I, D> {
        DataFrame::new(self.index.indices.head, self.data)
    }
}

impl<I, D> std::ops::Index<usize> for DataFrame<I, D>
where
    I: VariableRange,
    D: FrameData,
{
    type Output = D::Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
