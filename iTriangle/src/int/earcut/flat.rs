use i_overlay::i_float::int::point::IntPoint;
use crate::int::earcut::earcut_64::{Bit, EarcutStore};
use crate::int::triangulation::{IndexType, IntTriangulation};

pub(super) struct FlatEarcutStore<'a, I> {
    triangulation: &'a mut IntTriangulation<I>,
}

impl<'a, I: IndexType> FlatEarcutStore<'a, I> {
    #[inline]
    pub(super) fn new(triangulation: &'a mut IntTriangulation<I>) -> Self {
        Self { triangulation }
    }
}

impl<I: IndexType> EarcutStore for FlatEarcutStore<'_, I> {
    #[inline]
    fn collect_triangles(&mut self, _: &[IntPoint], start: usize, bits: u64, count: u32) {
        let mut i = start;
        let a = unsafe { I::try_from(i).unwrap_unchecked() };
        i = bits.next_wrapped_index(i);
        let mut b = unsafe { I::try_from(i).unwrap_unchecked() };

        for _ in 0..count {
            i = bits.next_wrapped_index(i);
            let c = unsafe { I::try_from(i).unwrap_unchecked() };
            self.triangulation.indices.push(a);
            self.triangulation.indices.push(b);
            self.triangulation.indices.push(c);

            b = c;
        }
    }
}
