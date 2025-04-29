use crate::float::delaunay::Delaunay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

impl<P: FloatPointCompatible<T>, T: FloatNumber> Delaunay<P, T> {
    #[inline]
    pub fn refine_with_circumcenters(mut self, min_area: T) -> Self {
        let int_area = self.adapter.sqr_float_to_int(min_area);
        self.delaunay = self.delaunay.refine_with_circumcenters(int_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle(mut self, min_area: T) -> Self {
        let int_area = self.adapter.sqr_float_to_int(min_area);
        self.delaunay = self.delaunay.refine_with_circumcenters_by_obtuse_angle(int_area);
        self
    }
}