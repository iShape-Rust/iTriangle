use crate::float::delaunay::Delaunay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

impl<P: FloatPointCompatible> Delaunay<P> {
    #[inline]
    pub fn refine_with_circumcenters(mut self, min_area: P::Scalar) -> Self {
        self.refine_with_circumcenters_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle(mut self, min_area: P::Scalar) -> Self {
        self.refine_with_circumcenters_by_obtuse_angle_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_mut(&mut self, min_area: P::Scalar) {
        let int_area = self.adapter.sqr_float_to_int(min_area);
        self.delaunay.refine_with_circumcenters_mut(int_area);
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle_mut(&mut self, min_area: P::Scalar) {
        let int_area = self.adapter.sqr_float_to_int(min_area);
        self.delaunay
            .refine_with_circumcenters_by_obtuse_angle_mut(int_area);
    }
}
