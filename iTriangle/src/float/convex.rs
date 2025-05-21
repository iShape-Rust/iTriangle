use alloc::vec::Vec;
use crate::float::delaunay::Delaunay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::base::data::Contour;
use i_overlay::i_shape::float::adapter::ShapeToFloat;

impl<P: FloatPointCompatible<T>, T: FloatNumber> Delaunay<P, T> {
    /// Groups triangles into non-overlapping convex polygons in counter-clockwise order.
    ///
    /// Returns a list of float-based [`Contour<P>`]s.
    #[inline]
    pub fn to_convex_polygons(&self) -> Vec<Contour<P>> {
        self.delaunay.to_convex_polygons().to_float(&self.adapter)
    }
}
