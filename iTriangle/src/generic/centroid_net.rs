use crate::generic::adapter::PointAdapter;
use crate::generic::delaunay::Delaunay;
use alloc::vec::Vec;
use i_overlay::i_shape::base::data::Contour;

impl<A: PointAdapter> Delaunay<A> {
    #[inline]
    pub fn to_centroid_net(&self, min_area: A::Measure) -> Vec<Contour<A::Point>> {
        let int_area = self.adapter.measure_to_int_area(min_area);
        self.delaunay
            .centroid_net(int_area)
            .into_iter()
            .map(|contour| self.adapter.points_from_int(&contour))
            .collect()
    }
}
