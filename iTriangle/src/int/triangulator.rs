use crate::int::monotone::mesh_builder::TriangleMeshBuilder;
use crate::int::binder::SteinerInference;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::monotone::chain_vertex::IntoPoints;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::IntOverlayOptions;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::count::PointsCount;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::monotone::chain_builder::ToChainVertices;

#[derive(Debug, Clone, Copy)]
pub struct Validation {
    pub fill_rule: FillRule,
    pub options: IntOverlayOptions,
}

/// A reusable configuration object for performing 2D triangulation on shapes, contours, and paths.
///
/// Supports both validated and unchecked triangulation methods. Shapes are expected to use integer
/// coordinates, and complex geometry (with holes or Steiner points) is supported.
///
/// # Default Validation
/// - `fill_rule`: [`FillRule::NonZero`]
/// - `options`: Adjust custom behavior.
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
                options: IntOverlayOptions::keep_all_points(),
            },
        }
    }
}

impl Triangulator {
    pub fn with_fill_rule(fill_rule: FillRule) -> Self {
        Self {
            validation: Validation {
                fill_rule,
                options: IntOverlayOptions::keep_output_points(),
            }
        }
    }

    /// Triangulates a list of shapes after validating and simplifying them.
    ///
    /// Applies the configured fill rule, contour direction, and area threshold before triangulation.
    ///
    /// # Returns
    /// A [`RawIntTriangulation`] containing the resulting triangle mesh.
    ///
    /// # See Also
    /// - [`Triangulator::unchecked_triangulate_shapes`] for int input without validation.
    pub fn triangulate_shapes(&self, shapes: &IntShapes) -> RawIntTriangulation {
        let shapes = shapes.simplify(self.validation.fill_rule, self.validation.options);
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
    pub fn unchecked_triangulate_shapes(&self, shapes: &IntShapes) -> RawIntTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.unchecked_triangulate_shape(first)
            } else {
                RawIntTriangulation::empty()
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
            let mut first_raw = self.unchecked_triangulate_shape(first);
            triangles.append(&mut first_raw.triangles);
            points.append(&mut first_raw.points);

            for shape in iter {
                let points_offset = points.len();
                let triangle_offset = triangles.len();
                let mut raw = self.unchecked_triangulate_shape(shape);
                raw.shift(points_offset, triangle_offset);

                triangles.append(&mut raw.triangles);
                points.append(&mut raw.points);
            }
        }

        RawIntTriangulation::new(triangles, points)
    }

    /// Triangulates a list of shapes, inserting user-provided Steiner points before processing.
    ///
    /// Performs validation, simplification, and grouping of Steiner points into corresponding shapes.
    pub fn triangulate_shapes_with_steiner_points(
        &self,
        shapes: &IntShapes,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        let shapes = shapes.simplify(self.validation.fill_rule, self.validation.options);
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
    ) -> RawIntTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.unchecked_triangulate_shape_with_steiner_points(first, &groups[0])
            } else {
                RawIntTriangulation::empty()
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

        RawIntTriangulation::new(triangles, points)
    }
}

impl Triangulator {
    /// Triangulates a single shape after validation and simplification.
    pub fn triangulate_shape(&self, shape: &IntShape) -> RawIntTriangulation {
        let shapes = shape.simplify(self.validation.fill_rule, self.validation.options);
        self.unchecked_triangulate_shapes(&shapes)
    }

    /// Triangulates a single valid shape without simplification or validation.
    pub fn unchecked_triangulate_shape(&self, shape: &IntShape) -> RawIntTriangulation {
        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2);

        let chain_vertices = shape.to_chain_vertices();
        let mut net_builder = TriangleMeshBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawIntTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    /// Triangulates a shape with Steiner points after simplifying and validating it.
    pub fn triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        let shapes = shape.simplify(self.validation.fill_rule, self.validation.options);
        let groups = shapes.group_by_shapes(points);
        self.unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
    }

    /// Triangulates a single shape with associated Steiner points, assuming everything is valid.
    pub fn unchecked_triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        if shape.len() <= 1 {
            return if let Some(first) = shape.first() {
                self.unchecked_triangulate_contour_with_steiner_points(first, points)
            } else {
                RawIntTriangulation::empty()
            };
        }

        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * points.len();

        let chain_vertices = shape.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleMeshBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawIntTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }
}

impl Triangulator {
    /// Triangulates a single closed contour after simplification.
    /// Converts it into a valid shape before processing.
    pub fn triangulate_contour(&self, contour: &IntContour) -> RawIntTriangulation {
        let shapes = contour.simplify(self.validation.fill_rule, self.validation.options);
        self.unchecked_triangulate_shapes(&shapes)
    }

    /// Triangulates a single closed contour assuming it is valid and oriented correctly.
    pub fn unchecked_triangulate_contour(&self, contour: &IntContour) -> RawIntTriangulation {
        if contour.len() < 3 {
            return RawIntTriangulation::empty();
        }
        let triangles_count = contour.len() - 2;

        let chain_vertices = contour.to_chain_vertices();
        let mut net_builder = TriangleMeshBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawIntTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    /// Triangulates a closed contour with Steiner points after validation.
    pub fn triangulate_contour_with_steiner_points(
        &self,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        let shapes = contour.simplify(self.validation.fill_rule, self.validation.options);
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
    ) -> RawIntTriangulation {
        if contour.len() < 3 {
            return RawIntTriangulation::empty();
        }
        let triangles_count = contour.len() - 2 + 2 * points.len();

        let chain_vertices = contour.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleMeshBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawIntTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }
}
