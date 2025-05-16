use crate::int::triangulation::RawIntTriangulation;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::solver::{ContourSolver, ShapeSolver, ShapesSolver};
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
pub trait IntTriangulatable {
    /// Triangulates the shape(s) with automatic validation and cleanup.
    ///
    /// Uses the default [`DisposableTriangulator`] (non-zero fill rule, zero area threshold).
    fn triangulate(&self) -> RawIntTriangulation;

    /// Triangulates the shape(s) with inserted Steiner points.
    ///
    /// Points must lie within the shape's valid interior area (not on edges).
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation;
}

impl IntTriangulatable for IntContour {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation {
        ContourSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        ContourSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}

impl IntTriangulatable for IntShape {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation {
        ShapeSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        ShapeSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}

impl IntTriangulatable for IntShapes {
    #[inline]
    fn triangulate(&self) -> RawIntTriangulation {
        ShapesSolver::triangulate(Default::default(), self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        ShapesSolver::triangulate_with_steiner_points(Default::default(), self, points)
    }
}