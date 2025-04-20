use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::raw::triangulation::RawTriangulation;
use crate::raw::triangulator::Triangulator;

pub trait Triangulatable {
    fn triangulate(&self) -> RawTriangulation;
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation;
}

impl Triangulatable for IntContour {

    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_contour(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_contour_with_steiner_points(self, points)
    }
}

impl Triangulatable for IntShape {

    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_shape(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_shape_with_steiner_points(self, points)
    }
}

impl Triangulatable for IntShapes {

    #[inline]
    fn triangulate(&self) -> RawTriangulation {
        Triangulator::default().triangulate_shapes(self)
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[IntPoint]) -> RawTriangulation {
        Triangulator::default().triangulate_shapes_with_steiner_points(self, points)
    }
}