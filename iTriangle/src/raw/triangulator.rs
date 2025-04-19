use crate::raw::builder::TriangleNetBuilder;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};
use crate::raw::triangulation::RawTriangulation;
use crate::raw::vertex::{IntoPoints, ToChainVertices};

pub struct Triangulator {
    fill_rule: Option<FillRule>,
    min_area: usize,
}

impl Default for Triangulator {
    fn default() -> Self {
        Self {
            fill_rule: Some(FillRule::NonZero),
            min_area: 0,
        }
    }
}

impl Triangulator {

    pub fn raw_triangulate_shape(&self, shape: &IntShape) -> RawTriangulation {
        let triangles_count =
            shape.iter().fold(0, |s, path| s + path.len() - 2);

        let chain_vertices = shape.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    pub fn raw_triangulate_shape_with_steiner_points(
        &self,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawTriangulation {
        let triangles_count =
            shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * points.len();

        let chain_vertices = shape.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    pub fn raw_triangulate_contour(&self, contour: &IntContour) -> RawTriangulation {
        let triangles_count = contour.len() - 2;

        let chain_vertices = contour.to_chain_vertices();
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }

    pub fn raw_triangulate_contour_with_steiner_points(
        &self,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawTriangulation {
        let triangles_count = contour.len() - 2 + 2 * points.len();

        let chain_vertices = contour.to_chain_vertices_with_steiner_points(points);
        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);
        net_builder.build(&chain_vertices);

        RawTriangulation::new(net_builder.triangles, chain_vertices.into_points())
    }
}

