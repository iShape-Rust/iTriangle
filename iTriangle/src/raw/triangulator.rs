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

#[derive(Debug, Clone, Copy)]
pub struct Triangulator {
    pub validation: Option<Validation>,
}

impl Default for Triangulator {
    fn default() -> Self {
        Self {
            validation: Some(Validation {
                fill_rule: FillRule::NonZero,
                min_area: 0,
            }),
        }
    }
}

impl Triangulator {
    pub fn triangulate_shapes(&self, shapes: &IntShapes) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = shapes.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes(&shapes)
        } else {
            self.triangulate_valid_shapes(shapes)
        }
    }

    pub fn triangulate_shapes_with_steiner_points(
        &self,
        shapes: &IntShapes,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = shapes.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes(&shapes)
        } else {
            self.triangulate_valid_shapes_with_steiner_points(shapes, points)
        }
    }

    pub fn triangulate_shape(&self, shape: &IntShape) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = shape.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes(&shapes)
        } else {
            self.triangulate_valid_shape(shape)
        }
    }

    pub fn triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = shape.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes_with_steiner_points(&shapes, points)
        } else {
            self.triangulate_valid_shape_with_steiner_points(shape, points)
        }
    }

    pub fn triangulate_contour(&self, contour: &IntContour) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = contour.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes(&shapes)
        } else {
            self.triangulate_valid_contour(contour)
        }
    }

    pub fn triangulate_contour_with_steiner_points(
        &self,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if let Some(validation) = self.validation {
            let shapes = contour.simplify(
                validation.fill_rule,
                ContourDirection::CounterClockwise,
                false,
                validation.min_area,
            );
            self.triangulate_valid_shapes_with_steiner_points(&shapes, points)
        } else {
            self.triangulate_valid_contour_with_steiner_points(contour, points)
        }
    }

    fn triangulate_valid_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if shape.len() <= 1 {
            return if let Some(first) = shape.first() {
                self.triangulate_valid_contour_with_steiner_points(first, points)
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

    fn triangulate_valid_shape(&self, shape: &IntShape) -> RawTriangulation {
        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2);

        let chain_vertices = shape.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    fn triangulate_valid_shapes(&self, shapes: &IntShapes) -> RawTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.triangulate_valid_shape(first)
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

            for shape in shapes.iter() {
                let points_offset = points.len();
                let triangle_offset = triangles.len();
                let mut raw = self.triangulate_shape(shape);
                for t in raw.triangles.iter_mut() {
                    t.vertices[0].index += points_offset;
                    t.vertices[1].index += points_offset;
                    t.vertices[2].index += points_offset;
                    t.neighbors[0] += triangle_offset;
                    t.neighbors[1] += triangle_offset;
                    t.neighbors[2] += triangle_offset;
                }
                triangles.append(&mut raw.triangles);
                points.append(&mut raw.points);
            }
        }

        RawTriangulation::new(triangles, points)
    }
    fn triangulate_valid_shapes_with_steiner_points(
        &self,
        shapes: &IntShapes,
        points: &[IntPoint],
    ) -> RawTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                self.triangulate_valid_shape_with_steiner_points(first, points)
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

            for shape in shapes.iter() {
                let points_offset = points.len();
                let triangle_offset = triangles.len();
                let mut raw = self.triangulate_shape(shape);
                for t in raw.triangles.iter_mut() {
                    t.vertices[0].index += points_offset;
                    t.vertices[1].index += points_offset;
                    t.vertices[2].index += points_offset;
                    t.neighbors[0] += triangle_offset;
                    t.neighbors[1] += triangle_offset;
                    t.neighbors[2] += triangle_offset;
                }
                triangles.append(&mut raw.triangles);
                points.append(&mut raw.points);
            }
        }

        RawTriangulation::new(triangles, points)
    }

    fn triangulate_valid_contour(&self, contour: &IntContour) -> RawTriangulation {
        if contour.len() < 3 {
            return RawTriangulation::empty();
        }
        let triangles_count = contour.len() - 2;

        let chain_vertices = contour.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    fn triangulate_valid_contour_with_steiner_points(
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
