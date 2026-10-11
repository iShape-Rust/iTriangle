pub use crate::advanced::convex::ConvexSplitStrategy;
use crate::float::delaunay::Delaunay;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::base::data::Contour;
use i_overlay::i_shape::float::adapter::ShapeToFloat;

impl<P: FloatPointCompatible, I: IntNumber> Delaunay<P, I> {
    /// Groups triangles into non-overlapping convex polygons in counter-clockwise order.
    ///
    /// Returns a list of float-based [`Contour<P>`]s.
    #[inline]
    pub fn to_convex_polygons(&self) -> Vec<Contour<P>> {
        self.delaunay.to_convex_polygons().to_float(&self.adapter)
    }

    /// Returns counter-clockwise convex polygons with at most `max_vertices.max(3)` vertices.
    ///
    /// Values of `max_vertices` below 3 are treated as 3.
    /// Pass [`ConvexSplitStrategy::default()`] to use ear clipping.
    /// Splitting is performed in integer space before conversion to float coordinates.
    /// Only oversized regions are split. See [`ConvexSplitStrategy`] for details.
    #[inline]
    pub fn to_convex_polygons_with_max_vertices(
        &self,
        max_vertices: usize,
        strategy: ConvexSplitStrategy,
    ) -> Vec<Contour<P>> {
        self.delaunay
            .to_convex_polygons_with_max_vertices(max_vertices, strategy)
            .to_float(&self.adapter)
    }
}
