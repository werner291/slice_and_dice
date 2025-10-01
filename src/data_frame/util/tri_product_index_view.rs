//! TriProductIndexView: a zero-copy view over a 3D logical layout selecting a fixed middle index.
use crate::data_frame::core::FrameData;
use std::ops::Index;

/// A view that iterates data in the same order as:
/// iproduct!(0..l_size, 0..r_size).map(|(l_i, r_i)| data[l_i*(m_size*r_size) + m_i*r_size + r_i])
/// for a fixed m_i.
pub struct TriProductIndexView<'a, D>
where
    D: FrameData + 'a,
{
    pub l_size: usize,
    pub m_size: usize,
    pub r_size: usize,
    pub m_i: usize,
    pub data: &'a D,
}

impl<'a, D> TriProductIndexView<'a, D>
where
    D: FrameData,
{
    /// Construct a new view. Panics if m_i >= m_size or if computed indices could go out of bounds.
    pub fn new(l_size: usize, m_size: usize, r_size: usize, m_i: usize, data: &'a D) -> Self {
        assert!(m_i < m_size, "m_i out of bounds");
        let len = l_size
            .checked_mul(r_size)
            .expect("l_size * r_size overflow");
        if len > 0 {
            // The last element accessed is when k = len-1 => l_i=(len-1)/r_size, r_i=(len-1)%r_size
            let l_i = (len - 1) / r_size;
            let r_i = (len - 1) % r_size;
            let flat_index = l_i
                .checked_mul(
                    m_size
                        .checked_mul(r_size)
                        .expect("m_size * r_size overflow"),
                )
                .and_then(|v| v.checked_add(m_i * r_size))
                .and_then(|v| v.checked_add(r_i))
                .expect("flat index overflow");
            assert!(flat_index < data.len(), "view exceeds data bounds");
        }
        Self {
            l_size,
            m_size,
            r_size,
            m_i,
            data,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.l_size * self.r_size
    }
}

impl<'a, D> Index<usize> for TriProductIndexView<'a, D>
where
    D: FrameData,
{
    type Output = <D as Index<usize>>::Output;

    #[inline]
    fn index(&self, k: usize) -> &Self::Output {
        debug_assert!(k < self.len());
        // Map linear k into (l_i, r_i) where r changes fastest
        let l_i = k / self.r_size;
        let r_i = k % self.r_size;
        let flat_index = l_i * (self.m_size * self.r_size) + self.m_i * self.r_size + r_i;
        &self.data[flat_index]
    }
}

impl<'a, D> FrameData for TriProductIndexView<'a, D>
where
    D: FrameData,
{
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::iproduct;
    use std::ops::Index as _;

    #[test]
    fn tri_product_index_view_matches_iproduct_order() {
        let l_size = 3;
        let m_size = 2;
        let r_size = 4;
        let m_i = 1; // Fix middle index
        // Build an example backing buffer with obvious values
        let total = l_size * m_size * r_size;
        let data: Vec<usize> = (0..total).collect();

        let view = TriProductIndexView::new(l_size, m_size, r_size, m_i, &data);
        let expected: Vec<&usize> = iproduct!(0..l_size, 0..r_size)
            .map(|(l_i, r_i)| l_i * (m_size * r_size) + m_i * r_size + r_i)
            .map(|ix| &data[ix])
            .collect();
        let got: Vec<&usize> = (0..view.len())
            .map(|k| std::ops::Index::index(&view, k))
            .collect();
        assert_eq!(got, expected);
    }
}
