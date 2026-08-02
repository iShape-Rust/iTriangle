use crate::int::solver::ShapesSolver;
use crate::int::solver::{ContourSolver, ShapeSolver};
use crate::int::triangulation::RawIntTriangulation;
use crate::int::validation::Validation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::number::int::IntNumber;
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
pub trait IntCustomTriangulatable<I: IntNumber> {
    /// Triangulates the shape(s) using the given [`Validation`] settings.
    fn custom_triangulate(&self, validation: Validation<I>) -> RawIntTriangulation<I>;

    /// Triangulates the shape(s), injecting Steiner points and using the specified [`Validation`] settings.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawIntTriangulation<I>;
}

impl<I: OverlayInt> IntCustomTriangulatable<I> for IntContour<I> {
    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawIntTriangulation<I> {
        ContourSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawIntTriangulation<I> {
        ContourSolver::triangulate_with_steiner_points(validation, self, points)
    }
}

impl<I: OverlayInt> IntCustomTriangulatable<I> for IntShape<I> {
    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawIntTriangulation<I> {
        ShapeSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawIntTriangulation<I> {
        ShapeSolver::triangulate_with_steiner_points(validation, self, points)
    }
}

impl<I: OverlayInt> IntCustomTriangulatable<I> for IntShapes<I> {
    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawIntTriangulation<I> {
        ShapesSolver::triangulate(validation, self)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawIntTriangulation<I> {
        ShapesSolver::triangulate_with_steiner_points(validation, self, points)
    }
}
