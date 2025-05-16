use crate::int::binder::SteinerInference;
use crate::int::solver::{ContourSolver, ShapeSolver, ShapesSolver};
use crate::int::triangulation::RawIntTriangulation;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

/// A trait for performing triangulation on already validated geometry.
///
/// Skips all shape simplification and orientation checks for maximum performance.
/// Useful when the input is known to be valid (e.g., preprocessed or generated data).
///
/// # ⚠️ Safety Requirements
/// - Outer contours must be in **counter-clockwise** order
/// - Holes must be in **clockwise** order
/// - Shapes must not have self-intersections
/// - Steiner points must be **strictly inside** the shape
///
/// # Implemented For
/// - [`IntContour`]
/// - [`IntShape`]
/// - [`IntShapes`]
pub trait IntUncheckedTriangulatable {
    /// Performs triangulation without applying any shape simplification or validation.
    fn uncheck_triangulate(&self) -> RawIntTriangulation;

    /// Performs triangulation without validation, inserting the given Steiner points.
    ///
    /// Points are grouped and applied based on their target shape.
    fn uncheck_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation;
}

impl IntUncheckedTriangulatable for IntContour {
    #[inline]
    fn uncheck_triangulate(&self) -> RawIntTriangulation {
        ContourSolver::uncheck_triangulate(self)
    }

    #[inline]
    fn uncheck_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        ContourSolver::uncheck_triangulate_with_steiner_points(self, points)
    }
}

impl IntUncheckedTriangulatable for IntShape {
    #[inline]
    fn uncheck_triangulate(&self) -> RawIntTriangulation {
        ShapeSolver::uncheck_triangulate(self)
    }

    #[inline]
    fn uncheck_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        ShapeSolver::uncheck_triangulate_with_steiner_points(self, points)
    }
}

impl IntUncheckedTriangulatable for IntShapes {
    #[inline]
    fn uncheck_triangulate(&self) -> RawIntTriangulation {
        ShapesSolver::uncheck_triangulate(self)
    }

    #[inline]
    fn uncheck_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawIntTriangulation {
        let group = self.group_by_shapes(points);
        ShapesSolver::uncheck_triangulate_with_steiner_points(self, &group)
    }
}
