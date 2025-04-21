use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::binder::SteinerInference;
use crate::int::triangulation::IntTriangulation;
use crate::int::triangulator::Triangulator;

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
    fn unchecked_triangulate(&self) -> IntTriangulation;

    /// Performs triangulation without validation, inserting the given Steiner points.
    ///
    /// Points are grouped and applied based on their target shape.
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation;
}

impl IntUncheckedTriangulatable for IntContour {
    #[inline]
    fn unchecked_triangulate(&self) -> IntTriangulation {
        Triangulator::default().unchecked_triangulate_contour(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        Triangulator::default().unchecked_triangulate_contour_with_steiner_points(self, points)
    }
}

impl IntUncheckedTriangulatable for IntShape {
    #[inline]
    fn unchecked_triangulate(&self) -> IntTriangulation {
        Triangulator::default().unchecked_triangulate_shape(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        Triangulator::default().unchecked_triangulate_shape_with_steiner_points(self, points)
    }
}

impl IntUncheckedTriangulatable for IntShapes {
    #[inline]
    fn unchecked_triangulate(&self) -> IntTriangulation {
        Triangulator::default().unchecked_triangulate_shapes(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        let group = self.group_by_shapes(points);
        Triangulator::default().unchecked_triangulate_shapes_with_steiner_points(self, &group)
    }
}