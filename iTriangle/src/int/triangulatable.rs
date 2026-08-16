use crate::advanced::delaunay::IntDelaunay;
use crate::int::constraint::{constraint_points, Constrain};
use crate::int::solver::{ContourSolver, ShapeSolver, ShapesSolver};
use crate::int::triangulation::RawIntTriangulation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use i_overlay::string::line::IntLine;
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

/// A trait for building a constrained Delaunay triangulation.
///
/// Unlike holes, constraints do not remove any area. They force the requested internal
/// line segments to appear as edges in the resulting triangle mesh.
pub trait IntConstrainedTriangulatable<I: IntNumber> {
    /// Builds a constrained Delaunay triangulation containing every requested internal edge.
    ///
    /// Constraint endpoints must lie strictly inside the geometry. Constraints may share
    /// endpoints, but must not cross each other or the shape boundary.
    fn triangulate_with_constraints(&self, constraints: &[IntLine<I>]) -> IntDelaunay<I>;
}

impl<I: OverlayInt> IntConstrainedTriangulatable<I> for IntContour<I> {
    #[inline]
    fn triangulate_with_constraints(&self, constraints: &[IntLine<I>]) -> IntDelaunay<I> {
        let points = constraint_points(constraints);
        ContourSolver::triangulate_with_steiner_points(Default::default(), self, &points)
            .into_constrained_delaunay(constraints)
    }
}

impl<I: OverlayInt> IntConstrainedTriangulatable<I> for IntShape<I> {
    #[inline]
    fn triangulate_with_constraints(&self, constraints: &[IntLine<I>]) -> IntDelaunay<I> {
        let points = constraint_points(constraints);
        ShapeSolver::triangulate_with_steiner_points(Default::default(), self, &points)
            .into_constrained_delaunay(constraints)
    }
}

impl<I: OverlayInt> IntConstrainedTriangulatable<I> for IntShapes<I> {
    #[inline]
    fn triangulate_with_constraints(&self, constraints: &[IntLine<I>]) -> IntDelaunay<I> {
        let points = constraint_points(constraints);
        ShapesSolver::triangulate_with_steiner_points(Default::default(), self, &points)
            .into_constrained_delaunay(constraints)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{IntConstrainedTriangulatable, IntTriangulatable};
    use crate::int::triangulation::IntTriangulation;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::shape::IntShapes;
    use i_overlay::i_shape::int_shapes;

    fn has_edge(triangulation: &IntTriangulation<i32, u16>, edge: [IntPoint<i32>; 2]) -> bool {
        let a = triangulation
            .points
            .iter()
            .position(|&point| point == edge[0])
            .unwrap() as u16;
        let b = triangulation
            .points
            .iter()
            .position(|&point| point == edge[1])
            .unwrap() as u16;
        triangulation
            .indices
            .chunks_exact(3)
            .any(|triangle| triangle.contains(&a) && triangle.contains(&b))
    }

    fn assert_has_edge(triangulation: &IntTriangulation<i32, u16>, edge: [IntPoint<i32>; 2]) {
        assert!(
            has_edge(triangulation, edge),
            "missing constraint edge {edge:?}"
        );
    }

    #[test]
    fn test_0() {
        let shapes: IntShapes<i32> = int_shapes![[[[-5, -5], [5, -5], [5, 5], [-5, 5]],],];
        let constraints = [[IntPoint::new(-2, 0), IntPoint::new(2, 0)]];

        let triangulation: IntTriangulation<_, u16> = shapes
            .triangulate_with_constraints(&constraints)
            .into_triangulation();

        assert_has_edge(&triangulation, constraints[0]);

        std::println!("points: {:#?}", triangulation.points);
        std::println!("triangles: {:#?}", triangulation.indices);
    }

    #[test]
    fn multiple_constraints_are_preserved() {
        let shapes: IntShapes<i32> = int_shapes![[[[-10, -10], [10, -10], [10, 10], [-10, 10]],],];
        let constraints = [
            [IntPoint::new(-8, 0), IntPoint::new(8, 0)],
            [IntPoint::new(8, 0), IntPoint::new(0, -4)],
            [IntPoint::new(0, 1), IntPoint::new(0, 4)],
        ];

        let points: alloc::vec::Vec<_> = constraints.iter().flatten().copied().collect();
        let ordinary: IntTriangulation<_, u16> = shapes
            .triangulate_with_steiner_points(&points)
            .into_delaunay()
            .into_triangulation();
        assert!(!has_edge(&ordinary, constraints[0]));

        let triangulation: IntTriangulation<_, u16> = shapes
            .triangulate_with_constraints(&constraints)
            .into_triangulation();

        for edge in constraints {
            assert_has_edge(&triangulation, edge);
        }
    }
}
