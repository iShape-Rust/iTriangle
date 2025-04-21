use crate::int::triangulation::IntTriangulation;
use crate::int::triangulator::Triangulator;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

/// A trait for performing triangulation with default validation settings.
///
/// Provides a simplified interface for converting shapes or contours into triangle meshes.
/// Internally applies the default [`Triangulator`] settings:
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
/// Returns an [`IntTriangulation`] containing vertex indices and point data.
///
/// # Steiner Points
/// Use [`triangulate_with_steiner_points`] to inject additional internal points during triangulation.
pub trait IntTriangulatable {
    /// Triangulates the shape(s) with automatic validation and cleanup.
    ///
    /// Uses the default [`Triangulator`] (non-zero fill rule, zero area threshold).
    fn triangulate(&self) -> IntTriangulation;

    /// Triangulates the shape(s) with inserted Steiner points.
    ///
    /// Points must lie within the shape's valid interior area (not on edges).
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation;
}

impl IntTriangulatable for IntContour {
    #[inline]
    fn triangulate(&self) -> IntTriangulation {
        Triangulator::default().triangulate_contour(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        Triangulator::default().triangulate_contour_with_steiner_points(self, points)
    }
}

impl IntTriangulatable for IntShape {
    #[inline]
    fn triangulate(&self) -> IntTriangulation {
        Triangulator::default().triangulate_shape(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        Triangulator::default().triangulate_shape_with_steiner_points(self, points)
    }
}

impl IntTriangulatable for IntShapes {
    #[inline]
    fn triangulate(&self) -> IntTriangulation {
        Triangulator::default().triangulate_shapes(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> IntTriangulation {
        Triangulator::default().triangulate_shapes_with_steiner_points(self, points)
    }
}