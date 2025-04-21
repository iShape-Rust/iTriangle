use crate::raw::binder::SteinerInference;
use crate::raw::builder::TriangleNetBuilder;
use crate::raw::triangulation::RawTriangulation;
use crate::raw::vertex::{IntoPoints, ToChainVertices};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ContourDirection;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::count::PointsCount;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

#[derive(Debug, Clone, Copy)]
pub struct Validation {
    pub fill_rule: FillRule,
    pub min_area: usize,
}

/// A reusable configuration object for performing 2D triangulation on shapes, contours, and paths.
///
/// Supports both validated and unchecked triangulation methods. Shapes are expected to use integer
/// coordinates, and complex geometry (with holes or Steiner points) is supported.
///
/// # Default Validation
/// - `fill_rule`: [`FillRule::NonZero`]
/// - `min_area`: `0` (include all areas)
///
/// Use `.triangulate_*` for auto-validation and `.unchecked_triangulate_*` if you guarantee valid input.
#[derive(Debug, Clone, Copy)]
pub struct Triangulator {
    pub validation: Validation,
}

impl Default for Triangulator {
    fn default() -> Self {
        Self {
            validation: Validation {
                fill_rule: FillRule::NonZero,
                min_area: 0,
            },
        }
    }
}

impl Triangulator {

    /// Triangulates a list of shapes after validating and simplifying them.
    ///
    /// Applies the configured fill rule, contour direction, and area threshold before triangulation.
    ///
    /// # Returns
    /// A [`RawTriangulation`] containing the resulting triangle mesh.
    ///
    /// # See Also
    /// - [`Triangulator::unchecked_triangulate_shapes`] for raw input without validation.
    pub fn triangulate_shapes(&self, shapes: &IntShapes) -> RawTriangulation {
        let shapes = shapes.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        self.unchecked_triangulate_shapes(&shapes)
    }

    /// Triangulates a list of shapes without any validation or correction.
    ///
    /// # Safety Requirements
    /// - Outer contours must be **counter-clockwise**.
    /// - Holes must be **clockwise**.
    /// - Shapes must not self-intersect.
    /// - Holes may only touch their parent contour **at a shared point**.
    /// - Steiner points must be strictly **inside** the shape (not on edges).
    ///
    /// # Returns
    /// - A flat triangulation result combining all input shapes.
    pub fn unchecked_triangulate_shapes(&self, shapes: &IntShapes) -> RawTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.unchecked_triangulate_shape(first)
            } else {
                RawTriangulation::empty()
            };
        }

        let mut triangles_count = 0;
        let mut points_count = 0;
        for shape in shapes.iter() {
            triangles_count += shape.iter().fold(0, |s, path| s + path.len() - 2);
            points_count += shape.points_count();
        }

        let mut triangles = Vec::with_capacity(triangles_count);
        let mut points = Vec::with_capacity(points_count);

        let mut iter = shapes.iter();
        if let Some(first) = iter.next() {
            let mut first_raw = self.triangulate_shape(first);
            triangles.append(&mut first_raw.triangles);
            points.append(&mut first_raw.points);

            for shape in iter {
                let points_offset = points.len();
                let triangle_offset = triangles.len();
                let mut raw = self.triangulate_shape(shape);
                raw.shift(points_offset, triangle_offset);

                triangles.append(&mut raw.triangles);
                points.append(&mut raw.points);
            }
        }

        RawTriangulation::new(triangles, points)
    }

    /// Triangulates a list of shapes, inserting user-provided Steiner points before processing.
    ///
    /// Performs validation, simplification, and grouping of Steiner points into corresponding shapes.
    pub fn triangulate_shapes_with_steiner_points(
        &self,
        shapes: &IntShapes,
        points: &[IntPoint],
    ) -> RawTriangulation {
        let shapes = shapes.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        let groups = shapes.group_by_shapes(points);
        self.unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
    }

    /// Performs triangulation on shapes and associated Steiner points without any validation.
    ///
    /// # Safety
    /// Same rules apply as [`unchecked_triangulate_shapes`], with additional constraints:
    /// - All Steiner points in each group must be **strictly inside** their assigned shape.
    pub fn unchecked_triangulate_shapes_with_steiner_points(
        &self,
        shapes: &IntShapes,
        groups: &[Vec<IntPoint>],
    ) -> RawTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.unchecked_triangulate_shape_with_steiner_points(first, &groups[0])
            } else {
                RawTriangulation::empty()
            };
        }

        let mut triangles_count = 0;
        let mut points_count = 0;
        for (i, shape) in shapes.iter().enumerate() {
            let steiner_points_count = groups[i].len();
            triangles_count +=
                shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * steiner_points_count;
            points_count += shape.points_count() + steiner_points_count;
        }

        let mut triangles = Vec::with_capacity(triangles_count);
        let mut points = Vec::with_capacity(points_count);

        let mut first_raw =
            self.unchecked_triangulate_shape_with_steiner_points(&shapes[0], &groups[0]);
        triangles.append(&mut first_raw.triangles);
        points.append(&mut first_raw.points);

        let mut i = 1;
        while i < shapes.len() {
            let shape = &shapes[i];
            let steiner_points = &groups[i];
            i += 1;

            let points_offset = points.len();
            let triangle_offset = triangles.len();
            let mut raw =
                self.unchecked_triangulate_shape_with_steiner_points(shape, steiner_points);
            raw.shift(points_offset, triangle_offset);

            triangles.append(&mut raw.triangles);
            points.append(&mut raw.points);
        }

        RawTriangulation::new(triangles, points)
    }
}

impl Triangulator {

    /// Triangulates a single shape after validation and simplification.
    pub fn triangulate_shape(&self, shape: &IntShape) -> RawTriangulation {
        let shapes = shape.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        self.unchecked_triangulate_shapes(&shapes)
    }

    /// Triangulates a single valid shape without simplification or validation.
    pub fn unchecked_triangulate_shape(&self, shape: &IntShape) -> RawTriangulation {
        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2);

        let chain_vertices = shape.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    /// Triangulates a shape with Steiner points after simplifying and validating it.
    pub fn triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawTriangulation {
        let shapes = shape.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        let groups = shapes.group_by_shapes(points);
        self.unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
    }

    /// Triangulates a single shape with associated Steiner points, assuming everything is valid.
    pub fn unchecked_triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if shape.len() <= 1 {
            return if let Some(first) = shape.first() {
                self.unchecked_triangulate_contour_with_steiner_points(first, points)
            } else {
                RawTriangulation::empty()
            };
        }

        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * points.len();

        let chain_vertices = shape.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }
}

impl Triangulator {

    /// Triangulates a single closed contour after simplification.
    /// Converts it into a valid shape before processing.
    pub fn triangulate_contour(&self, contour: &IntContour) -> RawTriangulation {
        let shapes = contour.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        self.unchecked_triangulate_shapes(&shapes)
    }

    /// Triangulates a single closed contour assuming it is valid and oriented correctly.
    pub fn unchecked_triangulate_contour(&self, contour: &IntContour) -> RawTriangulation {
        if contour.len() < 3 {
            return RawTriangulation::empty();
        }
        let triangles_count = contour.len() - 2;

        let chain_vertices = contour.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    /// Triangulates a closed contour with Steiner points after validation.
    pub fn triangulate_contour_with_steiner_points(
        &self,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawTriangulation {
        let shapes = contour.simplify(
            self.validation.fill_rule,
            ContourDirection::CounterClockwise,
            false,
            self.validation.min_area,
        );
        let groups = shapes.group_by_shapes(points);
        self.unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
    }

    /// Triangulates a closed contour with Steiner points, assuming all inputs are valid.
    ///
    /// # Note
    /// - Expects at least 3 contour points.
    /// - Steiner points must be strictly inside.
    pub fn unchecked_triangulate_contour_with_steiner_points(
        &self,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if contour.len() < 3 {
            return RawTriangulation::empty();
        }
        let triangles_count = contour.len() - 2 + 2 * points.len();

        let chain_vertices = contour.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }
}
