use crate::int::meta::TrianglesCount;
use crate::int::monotone::chain::builder::ChainBuilder;
use crate::int::monotone::chain::vertex::ChainVertex;
use crate::int::monotone::flat::triangulator::FlatTriangulation;
use crate::int::monotone::net::triangulator::NetTriangulation;
use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use alloc::vec::Vec;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};

#[derive(Default)]
pub(crate) struct MonotoneTriangulator {
    vertices: Option<Vec<ChainVertex>>,
    vertices_builder: ChainBuilder,
}

impl MonotoneTriangulator {
    #[inline]
    pub(crate) fn shape_into_net_triangulation(
        &mut self,
        shape: &IntShape,
        points: Option<&[IntPoint]>,
        triangulation: &mut RawIntTriangulation,
    ) {
        let points_count = points.map(|points| points.len()).unwrap_or(0);

        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder
            .shape_to_vertices(shape, points, &mut vertices);

        vertices.net_triangulate_into(shape.triangles_count(points_count), triangulation);

        self.vertices = Some(vertices);
    }

    #[inline]
    pub(crate) fn contour_into_net_triangulation(
        &mut self,
        contour: &IntContour,
        points: Option<&[IntPoint]>,
        triangulation: &mut RawIntTriangulation,
    ) {
        let points_count = points.map(|points| points.len()).unwrap_or(0);

        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder
            .contour_to_vertices(contour, points, &mut vertices);

        vertices.net_triangulate_into(contour.triangles_count(points_count), triangulation);

        self.vertices = Some(vertices);
    }

    #[inline]
    pub(crate) fn flat_into_net_triangulation(
        &mut self,
        flat: &FlatContoursBuffer,
        triangulation: &mut RawIntTriangulation,
    ) {
        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder.flat_to_vertices(flat, &mut vertices);

        vertices.net_triangulate_into(flat.triangles_count(0), triangulation);

        self.vertices = Some(vertices);
    }

    #[inline]
    pub(crate) fn shape_into_flat_triangulation<I: IndexType>(
        &mut self,
        shape: &IntShape,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder
            .shape_to_vertices(shape, None, &mut vertices);

        vertices.flat_triangulate_into(shape.triangles_count(0), triangulation);

        self.vertices = Some(vertices);
    }

    #[inline]
    pub(crate) fn contour_into_flat_triangulation<I: IndexType>(
        &mut self,
        contour: &IntContour,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder
            .contour_to_vertices(contour, None, &mut vertices);

        vertices.flat_triangulate_into(contour.triangles_count(0), triangulation);

        self.vertices = Some(vertices);
    }

    #[inline]
    pub(crate) fn flat_into_flat_triangulation<I: IndexType>(
        &mut self,
        flat: &FlatContoursBuffer,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let mut vertices = self.vertices.take().unwrap_or_default();
        self.vertices_builder.flat_to_vertices(flat, &mut vertices);

        vertices.flat_triangulate_into(flat.triangles_count(0), triangulation);

        self.vertices = Some(vertices);
    }
}
