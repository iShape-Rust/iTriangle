use crate::int::triangulation::IntTriangulation;
use crate::int::triangulator::{Triangulator, Validation};
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

/// A trait for performing triangulation with custom validation settings.
///
/// Useful when precise control over fill rule, minimum area, or orientation is needed.
/// Accepts a custom [`Validation`] struct to configure triangulation behavior.
///
/// # Implemented For
/// - [`IntContour`]
/// - [`IntShape`]
/// - [`IntShapes`]
pub trait IntCustomTriangulatable {
    /// Triangulates the shape(s) using the given [`Validation`] settings.
    fn custom_triangulate(&self, validation: Validation) -> IntTriangulation;

    /// Triangulates the shape(s), injecting Steiner points and using the specified [`Validation`] settings.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> IntTriangulation;
}

impl IntCustomTriangulatable for IntContour {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> IntTriangulation {
        Triangulator { validation }.triangulate_contour(self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> IntTriangulation {
        Triangulator { validation }.triangulate_contour_with_steiner_points(self, points)
    }
}

impl IntCustomTriangulatable for IntShape {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> IntTriangulation {
        Triangulator { validation }.triangulate_shape(self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> IntTriangulation {
        Triangulator { validation }.triangulate_shape_with_steiner_points(self, points)
    }
}

impl IntCustomTriangulatable for IntShapes {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> IntTriangulation {
        Triangulator { validation }.triangulate_shapes(self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> IntTriangulation {
        Triangulator { validation }.triangulate_shapes_with_steiner_points(self, points)
    }
}
