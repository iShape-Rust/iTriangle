use crate::int::solver::{ContourSolver, ShapeSolver, ShapesSolver};
use crate::int::triangulation::RawIntTriangulation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
/// A trait for performing triangulation with default validation settings.
///
/// Provides a simplified interface for converting shapes or contours into triangle meshes.
/// Internally applies the default [`DisposableTriangulator`] settings:
/// - [`FillRule::NonZero`]
/// - Minimum area = `0`
/// - Orientation = counter-clockwise for outer contours, clockwise for holes
///
/// # Implemented For
/// - [`IntContour`]
/// - [`IntShape`]
/// - [`IntShapes`]
///
/// # Output
/// Returns an [`RawIntTriangulation`] containing vertex indices and point data.
///
/// # Steiner Points
/// Use [`triangulate_with_steiner_points`] to inject additional internal points during triangulation.
pub trait IntTriangulatable<I: IntNumber> {
    /// Triangulates the shape(s) with automatic validation and cleanup.
    ///
    /// Uses the default [`DisposableTriangulator`] (non-zero fill rule, zero area threshold).
    fn triangulate(&self) -> RawIntTriangulation<I>;

    /// Triangulates the shape(s) with inserted Steiner points.
    ///
    /// Points must lie within the shape's valid interior area (not on edges).
    fn triangulate_with_steiner_points(&self, points: &[IntPoint<I>]) -> RawIntTriangulation<I>;
}

impl<I: OverlayInt> IntTriangulatable<I> for IntContour<I> {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation<I> {
        ContourSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint<I>]) -> RawIntTriangulation<I> {
        ContourSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}

impl<I: OverlayInt> IntTriangulatable<I> for IntShape<I> {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation<I> {
        ShapeSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint<I>]) -> RawIntTriangulation<I> {
        ShapeSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}

impl<I: OverlayInt> IntTriangulatable<I> for IntShapes<I> {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation<I> {
        ShapesSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint<I>]) -> RawIntTriangulation<I> {
        ShapesSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}
