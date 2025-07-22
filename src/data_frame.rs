use crate::mapped_index::MappedIndex;
use crate::mapped_index::compound_index::CompoundIndex;
use crate::mapped_index::numeric_range_index::NumericRangeIndex;
use std::marker::PhantomData;
use std::ops::Index;
use typenum::{Unsigned, U0, U1};

/// A generic DataFrame type associating an index with a data collection.
///
/// The index must implement MappedIndex, and the data must be indexable by usize (e.g., Vec).
/// This allows efficient access to data by index value or flat index.
pub struct DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// The index structure (categorical, numeric, compound, etc.).
    pub index: I,
    /// The data collection, indexable by flat index.
    pub data: D,
    _phantom: PhantomData<&'idx Idx>,
}

impl<'idx, I, D, Idx> DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    /// Construct a new DataFrame from index and data.
    pub fn new(index: I, data: D) -> Self {
        Self { index, data, _phantom: PhantomData }
    }
    /// Get a reference to the data for a given index value.
    pub fn get(&'idx self, value: I::Value) -> &D::Output {
        &self.data[self.index.to_flat_index(value)]
    }
    /// Get a reference to the data for a given flat index.
    pub fn get_flat(&self, flat_index: usize) -> &D::Output {
        &self.data[flat_index]
    }
    /// Stack an iterator of DataFrames into one DataFrame with a compound index.
    ///
    /// The top-level index selects the original DataFrame, and the lower-level index is from the original DataFrames.
    /// Returns an error if the inner indices are not compatible (i.e., not equal).
    pub fn stack<'a, J, E, It>(dfs: It) -> Result<DataFrame<'idx, CompoundIndex<(J, I)>, Vec<D::Output>, (usize, Idx)>, &'static str>
    where
        I: Clone + 'idx,
        D: Clone,
        D::Output: Clone,
        J: MappedIndex<'idx, usize> + Clone,
        It: IntoIterator<Item = DataFrame<'idx, I, D, Idx>>,
    {
        let dfs: Vec<DataFrame<'idx, I, D, Idx>> = dfs.into_iter().collect();
        if dfs.is_empty() {
            return Err("No dataframes to stack");
        }
        // Check all inner indices are equal
        let first_index = &dfs[0].index;
        for df in &dfs[1..] {
            // Compare by iterating over all values
            if df.index.size() != first_index.size() || !df.index.iter().eq(first_index.iter()) {
                return Err("All inner indices must be equal to stack");
            }
        }
        // Build the compound index: outer is a numeric range, inner is the shared index
        let outer_index = crate::mapped_index::numeric_range_index::NumericRangeIndex::new(0, dfs.len() as i32);
        let compound_index = CompoundIndex { indices: (outer_index, first_index.clone()) };
        // Flatten the data
        let mut data = Vec::new();
        for df in &dfs {
            for i in 0..df.index.size() {
                data.push(df.data[i].clone());
            }
        }
        Ok(DataFrame::new(compound_index, data))
    }
}

impl<'idx, A, B, D, IdxA, IdxB> DataFrame<'idx, CompoundIndex<(A, B)>, D, (IdxA, IdxB)>
where
    A: MappedIndex<'idx, IdxA> + Clone,
    B: MappedIndex<'idx, IdxB> + Clone,
    D: Index<usize>,
{
    /// Aggregate over the dimension specified by typenum (U0 for first, U1 for second).
    pub fn aggregate_over_dim<R, F, N>(&self, mut f: F) -> DataFrame<'idx, _, Vec<R>, _>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
        N: typenum::Unsigned,
        // N = U0 or U1
        // For U1, aggregate over B (as before)
        // For U0, aggregate over A
    {
        if N::USIZE == 1 {
            // Aggregate over B (second dimension)
            let a_index = self.index.indices.0.clone();
            let b_index = self.index.indices.1.clone();
            let mut result = Vec::with_capacity(a_index.size());
            for a_val in a_index.iter() {
                let mut values = (0..b_index.size()).map(|b_i| {
                    let b_val = b_index.from_flat_index(b_i);
                    let idx = (a_val, b_val);
                    &self.data[self.index.to_flat_index(idx)]
                });
                result.push(f(&mut values));
            }
            DataFrame::new(a_index, result)
        } else if N::USIZE == 0 {
            // Aggregate over A (first dimension)
            let a_index = self.index.indices.0.clone();
            let b_index = self.index.indices.1.clone();
            let mut result = Vec::with_capacity(b_index.size());
            for b_val in b_index.iter() {
                let mut values = (0..a_index.size()).map(|a_i| {
                    let a_val = a_index.from_flat_index(a_i);
                    let idx = (a_val, b_val);
                    &self.data[self.index.to_flat_index(idx)]
                });
                result.push(f(&mut values));
            }
            DataFrame::new(b_index, result)
        } else {
            panic!("Only 2D supported for now");
        }
    }
}

// Helper trait to project out the N-th element from a tuple
pub trait ProjectOut<N> {
    type Remaining;
    type Removed;
    fn project_out(self) -> (Self::Removed, Self::Remaining);
}

impl<T, Tail> ProjectOut<U0> for (T, Tail) {
    type Remaining = Tail;
    type Removed = T;
    fn project_out(self) -> (Self::Removed, Self::Remaining) {
        let (head, tail) = self;
        (head, tail)
    }
}

impl<T, Tail, N> ProjectOut<typenum::UInt<N, typenum::B1>> for (T, Tail)
where
    Tail: ProjectOut<N>,
    N: Unsigned,
{
    type Remaining = (T, <Tail as ProjectOut<N>>::Remaining);
    type Removed = <Tail as ProjectOut<N>>::Removed;
    fn project_out(self) -> (Self::Removed, Self::Remaining) {
        let (head, tail) = self;
        let (removed, new_tail) = tail.project_out();
        (removed, (head, new_tail))
    }
}

// Helper trait to project out the N-th index from a CompoundIndex
pub trait ProjectedIndex<N> {
    type Remaining;
    fn project_index(&self) -> Self::Remaining;
}

impl<N, Head, Tail> ProjectedIndex<N> for CompoundIndex<(Head, Tail)>
where
    (Head, Tail): ProjectOut<N>,
    <(Head, Tail) as ProjectOut<N>>::Remaining: Clone,
{
    type Remaining = CompoundIndex<<(Head, Tail) as ProjectOut<N>>::Remaining>;
    fn project_index(&self) -> Self::Remaining {
        let (_, remaining) = self.indices.clone().project_out();
        CompoundIndex::new(remaining)
    }
}

// N-dimensional aggregation for CompoundIndex
impl<'idx, Indices, D, IdxTuple, N> DataFrame<'idx, CompoundIndex<Indices>, D, IdxTuple>
where
    CompoundIndex<Indices>: MappedIndex<'idx, IdxTuple> + ProjectedIndex<N>,
    D: Index<usize>,
    N: Unsigned,
    <CompoundIndex<Indices> as ProjectedIndex<N>>::Remaining: Clone,
{
    pub fn aggregate_over_dim_nd<R, F>(&self, mut f: F) -> DataFrame<'idx, <CompoundIndex<Indices> as ProjectedIndex<N>>::Remaining, Vec<R>, _>
    where
        F: FnMut(&mut dyn Iterator<Item = &D::Output>) -> R,
    {
        let projected_index = self.index.project_index();
        let mut result = Vec::with_capacity(projected_index.size());
        for i in 0..projected_index.size() {
            let proj_val = projected_index.from_flat_index(i);
            // For each value in the projected index, collect all values in the original index that match
            let mut values = (0..self.index.size()).filter_map(|flat| {
                let full_val = self.index.from_flat_index(flat);
                // Project out the N-th element from full_val and compare to proj_val
                let (_, remaining) = full_val.project_out();
                if remaining == proj_val {
                    Some(&self.data[flat])
                } else {
                    None
                }
            });
            result.push(f(&mut values));
        }
        DataFrame::new(projected_index, result)
    }
}

impl<'idx, I, D, Idx> Index<I::Value> for DataFrame<'idx, I, D, Idx>
where
    I: MappedIndex<'idx, Idx>,
    D: Index<usize>,
{
    type Output = D::Output;
    fn index(&self, value: I::Value) -> &Self::Output {
        &self.data[self.index.to_flat_index(value)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapped_index::categorical_index::{CategoricalIndex, CategoricalValue};
    use crate::mapped_index::MappedIndex;
    use crate::mapped_index::compound_index::CompoundIndex;
    use crate::mapped_index::numeric_range_index::NumericRangeIndex;
    use std::marker::PhantomData;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct Tag;

    #[test]
    fn test_dataframe_get_and_index() {
        let index = CategoricalIndex::new(vec!["a", "b", "c"]);
        let data = vec![10, 20, 30];
        let df = DataFrame::new(index, data);
        let val = df.index.from_flat_index(1);
        assert_eq!(df.get(val), &20);
        assert_eq!(df[val], 20);
        assert_eq!(df.get_flat(2), &30);
    }

    #[test]
    fn test_dataframe_round_trip() {
        let index = CategoricalIndex::new(vec!["x", "y", "z"]);
        let data = vec![100, 200, 300];
        let df = DataFrame::new(index, data);
        for flat in 0..df.index.size() {
            let val = df.index.from_flat_index(flat);
            let round = df.index.to_flat_index(val);
            assert_eq!(flat, round);
            assert_eq!(df.get(val), &df.get_flat(flat));
        }
    }

    #[test]
    fn test_stack_success() {
        let index = CategoricalIndex::new(vec!["a", "b"]);
        let df1 = DataFrame::new(index.clone(), vec![1, 2]);
        let df2 = DataFrame::new(index.clone(), vec![3, 4]);
        let stacked = DataFrame::stack([df1, df2]).expect("should stack");
        // Compound index: outer is NumericRangeIndex, inner is CategoricalIndex
        assert_eq!(format!("{:?}", stacked.index.indices.0), format!("{:?}", NumericRangeIndex::new(0, 2)));
        assert_eq!(format!("{:?}", stacked.index.indices.1), format!("{:?}", index));
        assert_eq!(stacked.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_stack_incompatible() {
        let index1 = CategoricalIndex::new(vec!["a", "b"]);
        let index2 = CategoricalIndex::new(vec!["a", "c"]);
        let df1 = DataFrame::new(index1, vec![1, 2]);
        let df2 = DataFrame::new(index2, vec![3, 4]);
        let result = DataFrame::stack([df1, df2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_aggregate_over_b_sum() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b):
        // ("x", 0) = 1, ("x", 1) = 2, ("x", 2) = 3
        // ("y", 0) = 4, ("y", 1) = 5, ("y", 2) = 6
        let data = vec![1, 2, 3, 4, 5, 6];
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| it.copied().sum::<i32>());
        assert_eq!(agg.data, vec![6, 15]); // 1+2+3, 4+5+6
        assert_eq!(agg.index, a);
    }

    #[test]
    fn test_aggregate_over_b_count() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 2);
        let data = vec![10, 20, 30, 40]; // ("x",0)=10, ("x",1)=20, ("y",0)=30, ("y",1)=40
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| it.count());
        assert_eq!(agg.data, vec![2, 2]);
        assert_eq!(agg.index, a);
    }

    #[test]
    fn test_aggregate_over_b_min_max() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b):
        // ("x", 0) = 7, ("x", 1) = 2, ("x", 2) = 5
        // ("y", 0) = 4, ("y", 1) = 9, ("y", 2) = 6
        let data = vec![7, 2, 5, 4, 9, 6];
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_b(|it| {
            let mut it = it.copied();
            let min = it.clone().min().unwrap();
            let max = it.max().unwrap();
            (min, max)
        });
        assert_eq!(agg.data, vec![(2, 7), (4, 9)]); // (min, max) for each a
        assert_eq!(agg.index, a);
    }

    #[test]
    fn test_aggregate_over_a_sum() {
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b):
        // ("x", 0) = 1, ("x", 1) = 2, ("x", 2) = 3
        // ("y", 0) = 4, ("y", 1) = 5, ("y", 2) = 6
        let data = vec![1, 2, 3, 4, 5, 6];
        let compound = CompoundIndex::new((a.clone(), b.clone()));
        let df = DataFrame::new(compound, data);
        let agg = df.aggregate_over_dim::<_, _, U0>(|it| it.copied().sum::<i32>());
        assert_eq!(agg.data, vec![5, 7, 9]); // 1+4, 2+5, 3+6
        assert_eq!(agg.index, b);
    }

    #[test]
    fn test_aggregate_over_middle_dim_3d() {
        use typenum::U1;
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 2);
        let c = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b, c):
        // ("x", 0, 0) = 1, ("x", 0, 1) = 2, ("x", 0, 2) = 3
        // ("x", 1, 0) = 4, ("x", 1, 1) = 5, ("x", 1, 2) = 6
        // ("y", 0, 0) = 7, ("y", 0, 1) = 8, ("y", 0, 2) = 9
        // ("y", 1, 0) = 10, ("y", 1, 1) = 11, ("y", 1, 2) = 12
        let data = vec![1,2,3,4,5,6,7,8,9,10,11,12];
        let inner = CompoundIndex::new((b.clone(), c.clone()));
        let compound = CompoundIndex::new((a.clone(), inner));
        let df = DataFrame::new(compound, data);
        // Aggregate over the middle dimension (b, U1)
        let agg = df.aggregate_over_dim_nd::<_, _, U1>(|it| it.copied().sum::<i32>());
        // For each (a, c): sum over b
        // ("x", 0): 1+4=5, ("x", 1): 2+5=7, ("x", 2): 3+6=9
        // ("y", 0): 7+10=17, ("y", 1): 8+11=19, ("y", 2): 9+12=21
        assert_eq!(agg.data, vec![5,7,9,17,19,21]);
        // The index should be CompoundIndex<(A, C)>
        let projected = CompoundIndex::new((a, c));
        assert_eq!(agg.index, projected);
    }

    #[test]
    fn test_aggregate_over_first_dim_3d_custom() {
        use typenum::U0;
        let a = CategoricalIndex::<&'static str, Tag>::new(vec!["x", "y"]);
        let b = NumericRangeIndex::<Tag>::new(0, 2);
        let c = NumericRangeIndex::<Tag>::new(0, 3);
        // Data: for (a, b, c):
        // ("x", 0, 0) = 1, ("x", 0, 1) = 2, ("x", 0, 2) = 3
        // ("x", 1, 0) = 4, ("x", 1, 1) = 5, ("x", 1, 2) = 6
        // ("y", 0, 0) = 7, ("y", 0, 1) = 8, ("y", 0, 2) = 9
        // ("y", 1, 0) = 10, ("y", 1, 1) = 11, ("y", 1, 2) = 12
        let data = vec![1,2,3,4,5,6,7,8,9,10,11,12];
        let inner = CompoundIndex::new((b.clone(), c.clone()));
        let compound = CompoundIndex::new((a.clone(), inner));
        let df = DataFrame::new(compound, data);
        // Aggregate over the first dimension (a, U0), collect all values into a Vec
        let agg = df.aggregate_over_dim_nd::<_, _, U0>(|it| {
            let v: Vec<i32> = it.copied().collect();
            let min = *v.iter().min().unwrap();
            let max = *v.iter().max().unwrap();
            let sum: i32 = v.iter().sum();
            (min, max, sum, v)
        });
        // For each (b, c): collect all a
        // (0,0): [1,7], (0,1): [2,8], (0,2): [3,9]
        // (1,0): [4,10], (1,1): [5,11], (1,2): [6,12]
        assert_eq!(agg.data,
            vec![ (1,7,8,vec![1,7]), (2,8,10,vec![2,8]), (3,9,12,vec![3,9]),
                  (4,10,14,vec![4,10]), (5,11,16,vec![5,11]), (6,12,18,vec![6,12]) ]
        );
        let projected = CompoundIndex::new((b, c));
        assert_eq!(agg.index, projected);
    }

    #[test]
    fn test_aggregate_over_third_dim_5d_sum() {
        use typenum::U2;
        let a = NumericRangeIndex::<Tag>::new(0, 2); // 0,1
        let b = NumericRangeIndex::<Tag>::new(0, 2); // 0,1
        let c = NumericRangeIndex::<Tag>::new(0, 2); // 0,1
        let d = NumericRangeIndex::<Tag>::new(0, 2); // 0,1
        let e = NumericRangeIndex::<Tag>::new(0, 2); // 0,1
        // Data: for (a, b, c, d, e): value = a*16 + b*8 + c*4 + d*2 + e
        let mut data = Vec::new();
        for ai in 0..2 {
            for bi in 0..2 {
                for ci in 0..2 {
                    for di in 0..2 {
                        for ei in 0..2 {
                            data.push(ai*16 + bi*8 + ci*4 + di*2 + ei);
                        }
                    }
                }
            }
        }
        let idx = CompoundIndex::new((a.clone(), CompoundIndex::new((b.clone(), CompoundIndex::new((c.clone(), CompoundIndex::new((d.clone(), e.clone()))))))));
        let df = DataFrame::new(idx, data);
        // Aggregate over the third dimension (c, U2)
        let agg = df.aggregate_over_dim_nd::<_, _, U2>(|it| it.copied().sum::<i32>());
        // For each (a, b, d, e): sum over c=0 and c=1
        // There are 2^4 = 16 groups, each group is sum of two values
        let mut expected = Vec::new();
        for ai in 0..2 {
            for bi in 0..2 {
                for di in 0..2 {
                    for ei in 0..2 {
                        // c=0
                        let v0 = ai*16 + bi*8 + 0*4 + di*2 + ei;
                        // c=1
                        let v1 = ai*16 + bi*8 + 1*4 + di*2 + ei;
                        expected.push(v0 + v1);
                    }
                }
            }
        }
        assert_eq!(agg.data, expected);
        // The index should be CompoundIndex<(a, CompoundIndex<(b, CompoundIndex<(d, e)>)>)>
        let projected = CompoundIndex::new((a, CompoundIndex::new((b, CompoundIndex::new((d, e))))));
        assert_eq!(agg.index, projected);
    }
} 