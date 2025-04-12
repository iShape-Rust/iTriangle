use crate::plain::builder::TriangleNetBuilder;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::i_float::int::point::IntPoint;

use i_shape::int::shape::IntShape;
use crate::plain::vertex::ShapeToVertices;
use crate::triangulation::int::Triangulation;

pub struct Triangulator {
    validate_rule: Option<FillRule>,
    min_area: usize,
}

impl Default for Triangulator {
    fn default() -> Self {
        Self {
            validate_rule: Some(FillRule::NonZero),
            min_area: 0,
        }
    }
}

impl Triangulator {
    pub fn triangulate_with_inner_points(
        &self,
        shape: &IntShape,
        inner_points: &[IntPoint],
    ) -> Triangulation {
        let triangles_count =
            shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * inner_points.len();

        let chain_vertices = shape.to_chain_vertices(inner_points);
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        let indices = net_builder.triangle_indices();
        let points = chain_vertices.into_iter().map(|v|v.this).collect();

        Triangulation { points, indices }
    }

    #[inline]
    pub fn triangulate(&self, shape: &IntShape) -> Triangulation {
        self.triangulate_with_inner_points(shape, &[])
    }
}

