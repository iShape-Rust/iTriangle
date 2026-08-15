use crate::advanced::delaunay::IntDelaunay;
use crate::int::unchecked::IntUncheckedTriangulatable;
use crate::tessellation::split::SliceContour;
use crate::tessellation::uniform::IntUniformGrid;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::integer::OverlayInt;
use i_overlay::core::overlay::IntOverlayOptions;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

/// Builds a boundary-conforming Delaunay mesh from a uniform triangular lattice.
///
/// Boundary edges are split first. The split geometry is then normalized while
/// preserving every inserted collinear point. Interior lattice points that are too
/// close to any boundary edge are discarded before triangulation.
pub trait IntUniformTriangulatable<I: OverlayInt> {
    /// Triangulates using `edge_length` both as the maximum boundary segment length
    /// and as the horizontal spacing of the interior lattice.
    fn uniform_triangulate(&self, edge_length: I::WideUInt) -> IntDelaunay<I>;
}

impl<I: OverlayInt> IntUniformTriangulatable<I> for IntContour<I> {
    #[inline]
    fn uniform_triangulate(&self, edge_length: I::WideUInt) -> IntDelaunay<I> {
        validate_edge_length::<I>(edge_length);
        let sliced = self.slice_contour(edge_length);
        build_uniform(
            sliced.simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points()),
            edge_length,
        )
    }
}

impl<I: OverlayInt> IntUniformTriangulatable<I> for IntShape<I> {
    #[inline]
    fn uniform_triangulate(&self, edge_length: I::WideUInt) -> IntDelaunay<I> {
        validate_edge_length::<I>(edge_length);
        let sliced = self.slice_contour(edge_length);
        build_uniform(
            sliced.simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points()),
            edge_length,
        )
    }
}

impl<I: OverlayInt> IntUniformTriangulatable<I> for IntShapes<I> {
    #[inline]
    fn uniform_triangulate(&self, edge_length: I::WideUInt) -> IntDelaunay<I> {
        validate_edge_length::<I>(edge_length);
        let sliced = self.slice_contour(edge_length);
        build_uniform(
            sliced.simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points()),
            edge_length,
        )
    }
}

#[inline]
fn validate_edge_length<I: OverlayInt>(edge_length: I::WideUInt) {
    assert!(
        edge_length > I::WideUInt::ONE && edge_length <= I::WideUInt::HALF_MASK,
        "edge_length must be greater than one and fit the integer coordinate budget"
    );
}

#[inline]
fn build_uniform<I: OverlayInt>(shapes: IntShapes<I>, edge_length: I::WideUInt) -> IntDelaunay<I> {
    let steiner_points = shapes.uniform_grid(edge_length);
    shapes
        .uncheck_triangulate_with_steiner_points(&steiner_points)
        .into_delaunay()
}

#[cfg(test)]
mod tests {
    use super::IntUniformTriangulatable;
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn preserves_split_boundary_points() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];

        let delaunay = contour.uniform_triangulate(20u64);

        for x in [20, 40, 60, 80] {
            assert!(delaunay.points.contains(&IntPoint::new(x, 0)));
        }
        assert!(!delaunay.triangles.is_empty());
    }

    #[test]
    fn triangulates_shape_with_hole() {
        let shape = vec![
            vec![
                IntPoint::new(0, 0),
                IntPoint::new(100, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
            ],
            vec![
                IntPoint::new(40, 40),
                IntPoint::new(40, 60),
                IntPoint::new(60, 60),
                IntPoint::new(60, 40),
            ],
        ];

        let delaunay = shape.uniform_triangulate(10u64);

        assert!(!delaunay.triangles.is_empty());
        assert!(delaunay
            .points
            .iter()
            .all(|p| p.x <= 40 || 60 <= p.x || p.y <= 40 || 60 <= p.y));
    }
}
