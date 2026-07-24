use crate::generic::adapter::PointAdapter;
use crate::generic::delaunay::Delaunay;

impl<A: PointAdapter> Delaunay<A> {
    #[inline]
    pub fn refine_with_circumcenters(mut self, min_area: A::Measure) -> Self {
        self.refine_with_circumcenters_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle(mut self, min_area: A::Measure) -> Self {
        self.refine_with_circumcenters_by_obtuse_angle_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_mut(&mut self, min_area: A::Measure) {
        let int_area = self.adapter.measure_to_int_area(min_area);
        self.delaunay.refine_with_circumcenters_mut(int_area);
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle_mut(&mut self, min_area: A::Measure) {
        let int_area = self.adapter.measure_to_int_area(min_area);
        self.delaunay
            .refine_with_circumcenters_by_obtuse_angle_mut(int_area);
    }
}
