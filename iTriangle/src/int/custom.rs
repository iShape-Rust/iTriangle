use crate::int::solver::ShapesSolver;
use crate::int::solver::{ContourSolver, ShapeSolver};
use crate::int::triangulation::RawIntTriangulation;
use crate::int::validation::Validation;
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
    fn custom_triangulate(&self, validation: Validation) -> RawIntTriangulation;

    /// Triangulates the shape(s), injecting Steiner points and using the specified [`Validation`] settings.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> RawIntTriangulation;
}

impl IntCustomTriangulatable for IntContour {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> RawIntTriangulation {
        ContourSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> RawIntTriangulation {
        ContourSolver::triangulate_with_steiner_points(validation, self, points)
    }
}

impl IntCustomTriangulatable for IntShape {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> RawIntTriangulation {
        ShapeSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> RawIntTriangulation {
        ShapeSolver::triangulate_with_steiner_points(validation, self, points)
    }
}

impl IntCustomTriangulatable for IntShapes {
    #[inline]
    fn custom_triangulate(&self, validation: Validation) -> RawIntTriangulation {
        ShapesSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint],
        validation: Validation,
    ) -> RawIntTriangulation {
        ShapesSolver::triangulate_with_steiner_points(validation, self, points)
    }
}
