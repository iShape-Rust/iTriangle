use crate::generic::adapter::PointAdapter;
use crate::generic::delaunay::Delaunay;
use alloc::vec::Vec;
use i_overlay::i_shape::base::data::Contour;

impl<A: PointAdapter> Delaunay<A> {
    /// Groups triangles into non-overlapping convex polygons in counter-clockwise order.
    ///
    /// Returns a list of adapter-mapped [`Contour`]s.
    #[inline]
    pub fn to_convex_polygons(&self) -> Vec<Contour<A::Point>> {
        self.delaunay
            .to_convex_polygons()
            .into_iter()
            .map(|contour| self.adapter.points_from_int(&contour))
            .collect()
    }
}
