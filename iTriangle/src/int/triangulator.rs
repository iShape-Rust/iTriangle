use alloc::vec::Vec;
use crate::int::monotone::builder::TrianglesBuilder;
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::validation::Validation;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub struct IntTriangulator {
    validation: Validation,
    builder: TrianglesBuilder,
}

impl IntTriangulator {
    pub fn new(validation: Validation, avg_points_count: usize) -> Self {
        Self {
            validation,
            builder: TrianglesBuilder::with_capacity(3 * avg_points_count, avg_points_count),
        }
    }
}

impl Default for IntTriangulator {
    fn default() -> Self {
        Self {
            validation: Default::default(),
            builder: TrianglesBuilder::with_capacity(3 * 64, 64),
        }
    }
}

impl IntTriangulator {
    #[inline]
    pub fn triangulate_contour<I: IndexType>(
        &mut self,
        contour: IntContour,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let shapes = contour.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes(&shapes, delaunay)
    }

    #[inline]
    pub fn triangulate_contour_into<I: IndexType>(
        &mut self,
        contour: IntContour,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let shapes = contour.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes_into(&shapes, delaunay, triangulation);
    }

    #[inline]
    pub fn triangulate_shape<I: IndexType>(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let shapes = shape.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes(&shapes, delaunay)
    }

    #[inline]
    pub fn triangulate_shape_into<I: IndexType>(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let shapes = shape.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes_into(&shapes, delaunay, triangulation);
    }

    #[inline]
    pub fn triangulate_shapes<I: IndexType>(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let shapes = shapes.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes(&shapes, delaunay)
    }

    #[inline]
    pub fn triangulate_shapes_into<I: IndexType>(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let shapes = shapes.simplify(self.validation.fill_rule, self.validation.options);
        self.uncheck_triangulate_shapes_into(&shapes, delaunay, triangulation);
    }
}

impl IntTriangulator {
    #[inline]
    pub fn uncheck_triangulate_contour<I: IndexType>(
        &mut self,
        contour: &IntContour,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = IntTriangulation {
            points: Vec::new(),
            indices: Vec::new(),
        };
        self.uncheck_triangulate_contour_into(contour, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_contour_into<I: IndexType>(
        &mut self,
        contour: &IntContour,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        self.builder.build_contour(&contour, None);
        if delaunay {
            self.builder.delaunay_refine();
        }
        self.builder.feed_triangulation(triangulation);
    }

    #[inline]
    pub fn uncheck_triangulate_shape<I: IndexType>(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = IntTriangulation {
            points: Vec::new(),
            indices: Vec::new(),
        };
        self.uncheck_triangulate_shape_into(shape, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shape_into<I: IndexType>(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        self.builder.build_shape(&shape, None);
        if delaunay {
            self.builder.delaunay_refine();
        }
        self.builder.feed_triangulation(triangulation);
    }

    #[inline]
    pub fn uncheck_triangulate_shapes<I: IndexType>(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = IntTriangulation {
            points: Vec::new(),
            indices: Vec::new(),
        };
        self.uncheck_triangulate_shapes_into(shapes, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shapes_into<I: IndexType>(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let mut buffer = IntTriangulation {
            points: Vec::new(),
            indices: Vec::new(),
        };
        triangulation.points.clear();
        triangulation.indices.clear();

        for shape in shapes.iter() {
            self.uncheck_triangulate_shape_into(shape, delaunay, &mut buffer);
            triangulation.join(&buffer);
        }
    }
}
