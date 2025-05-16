use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::validation::Validation;

pub struct Triangulator {
    validation: Validation
}

impl Triangulator {
    pub fn triangulate_contour<I: IndexType>(contour: IntContour, delaunay: bool) -> IntTriangulation<I> {
        
        
        0
    }

    pub fn reuse_triangulate_contour<I: IndexType>(contour: IntContour, delaunay: bool, triangulation: &mut IntTriangulation<I>) {

    }

    pub fn triangulate_shape<I: IndexType>(shape: IntShape, delaunay: bool) -> IntTriangulation<I> {
        0
    }

    pub fn reuse_triangulate_shape<I: IndexType>(shape: IntShape, delaunay: bool, triangulation: &mut IntTriangulation<I>) {

    }

    pub fn triangulate_shapes<I: IndexType>(shapes: IntShapes, delaunay: bool) -> IntTriangulation<I> {
        0
    }

    pub fn reuse_triangulate_shapes<I: IndexType>(shapes: IntShapes, delaunay: bool, triangulation: &mut IntTriangulation<I>) {

    }
}