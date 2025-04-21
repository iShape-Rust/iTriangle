use crate::raw::binder::SteinerInference;
use crate::raw::triangulation::RawTriangulation;
use crate::raw::triangulator::Triangulator;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

/// A convenience trait for performing triangulation on validated input.
///
/// Provides simplified access to shape triangulation using default [`Triangulator`] settings.
/// Internally applies geometric validation (e.g., fill rule, orientation, area threshold)
/// before constructing the triangle mesh.
///
/// # Implemented For
/// - [`IntContour`]
/// - [`IntShape`]
/// - [`IntShapes`]
///
/// # Output
/// Returns a [`RawTriangulation`] mesh containing triangle indices and vertex data.
///
/// # Steiner Points
/// Use [`triangulate_with_steiner_points`] to inject additional internal vertices
/// before triangulation.
pub trait Triangulatable {
    /// Triangulates the shape(s) with automatic validation and cleanup.
    ///
    /// Uses the default [`Triangulator`] (non-zero fill rule, zero area threshold).
    fn triangulate(&self) -> RawTriangulation;

    /// Triangulates the shape(s) with inserted Steiner points.
    ///
    /// Points must lie within the shape's valid interior area (not on edges).
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation;
}

/// A trait for fast triangulation of already validated input geometry.
///
/// Assumes all contours and holes follow correct orientation and have no
/// self-intersections. Offers a performance advantage when validation is not needed.
///
/// # Safety Contract
/// - Outer contours must be counter-clockwise.
/// - Holes must be clockwise.
/// - Shapes must not self-intersect.
/// - Steiner points must be strictly inside their associated geometry.
///
/// # Implemented For
/// - [`IntContour`]
/// - [`IntShape`]
/// - [`IntShapes`]
pub trait UncheckedTriangulatable {
    /// Performs triangulation without applying any shape simplification or validation.
    fn unchecked_triangulate(&self) -> RawTriangulation;

    /// Performs triangulation without validation, inserting the given Steiner points.
    ///
    /// Points are grouped and applied based on their target shape.
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation;
}

impl Triangulatable for IntContour {
    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_contour(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_contour_with_steiner_points(self, points)
    }
}

impl Triangulatable for IntShape {
    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_shape(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_shape_with_steiner_points(self, points)
    }
}

impl Triangulatable for IntShapes {
    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_shapes(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_shapes_with_steiner_points(self, points)
    }
}

impl UncheckedTriangulatable for IntContour {
    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation {
        Triangulator::default().unchecked_triangulate_contour(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().unchecked_triangulate_contour_with_steiner_points(self, points)
    }
}

impl UncheckedTriangulatable for IntShape {
    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation {
        Triangulator::default().unchecked_triangulate_shape(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().unchecked_triangulate_shape_with_steiner_points(self, points)
    }
}

impl UncheckedTriangulatable for IntShapes {
    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation {
        Triangulator::default().unchecked_triangulate_shapes(self)
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        let group = self.group_by_shapes(points);
        Triangulator::default().unchecked_triangulate_shapes_with_steiner_points(self, &group)
    }
}
