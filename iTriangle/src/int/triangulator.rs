use crate::int::monotone::builder::TrianglesBuilder;
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::validation::Validation;
use alloc::vec::Vec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::Overlay;
use i_overlay::core::solver::Solver;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub struct IntTriangulator<I> {
    overlay: Overlay,
    fill_rule: FillRule,
    builder: TrianglesBuilder,
    triangulation_buffer: Option<IntTriangulation<I>>,
}

impl<I: IndexType> IntTriangulator<I> {
    #[inline]
    pub fn new(max_points_count: usize, validation: Validation, solver: Solver) -> Self {
        Self {
            overlay: Overlay::new_custom(max_points_count, validation.options, solver),
            fill_rule: validation.fill_rule,
            builder: TrianglesBuilder::with_capacity(3 * max_points_count, max_points_count),
            triangulation_buffer: None,
        }
    }
}

impl<I: IndexType> Default for IntTriangulator<I> {
    #[inline]
    fn default() -> Self {
        Self::new(64, Default::default(), Default::default())
    }
}

impl<I: IndexType> IntTriangulator<I> {
    #[inline]
    pub fn triangulate_contour(
        &mut self,
        contour: &IntContour,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        match self.overlay.simplify_contour(&contour, self.fill_rule) {
            None => self.uncheck_triangulate_contour(contour, delaunay),
            Some(shapes) => self.uncheck_triangulate_shapes(&shapes, delaunay),
        }
    }

    #[inline]
    pub fn triangulate_contour_into(
        &mut self,
        contour: IntContour,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        match self.overlay.simplify_contour(&contour, self.fill_rule) {
            None => self.uncheck_triangulate_contour_into(&contour, delaunay, triangulation),
            Some(shapes) => self.uncheck_triangulate_shapes_into(&shapes, delaunay, triangulation),
        }
    }

    #[inline]
    pub fn triangulate_shape(&mut self, shape: &IntShape, delaunay: bool) -> IntTriangulation<I> {
        match self.overlay.simplify_shape(shape, self.fill_rule) {
            None => self.uncheck_triangulate_shape(shape, delaunay),
            Some(shapes) => self.uncheck_triangulate_shapes(&shapes, delaunay),
        }
    }

    #[inline]
    pub fn triangulate_shape_into(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        match self.overlay.simplify_shape(shape, self.fill_rule) {
            None => self.uncheck_triangulate_shape_into(shape, delaunay, triangulation),
            Some(shapes) => self.uncheck_triangulate_shapes_into(&shapes, delaunay, triangulation),
        }
    }

    #[inline]
    pub fn triangulate_shapes(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let simple = self.overlay.simplify_shapes(shapes, self.fill_rule);
        self.uncheck_triangulate_shapes(&simple, delaunay)
    }

    #[inline]
    pub fn triangulate_shapes_into(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let simple = self.overlay.simplify_shapes(shapes, self.fill_rule);
        self.uncheck_triangulate_shapes_into(&simple, delaunay, triangulation);
    }

    #[inline]
    pub fn triangulate_flat(
        &mut self,
        flat: &mut FlatContoursBuffer,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        self.overlay.simplify_flat_buffer(flat, self.fill_rule);
        self.uncheck_triangulate_flat(flat, delaunay)
    }

    #[inline]
    pub fn triangulate_flat_into(
        &mut self,
        flat: &mut FlatContoursBuffer,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        self.overlay.simplify_flat_buffer(flat, self.fill_rule);
        self.uncheck_triangulate_flat_into(flat, delaunay, triangulation);
    }
}

impl<I: IndexType> IntTriangulator<I> {
    #[inline]
    pub fn uncheck_triangulate_contour(
        &mut self,
        contour: &IntContour,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_contour_into(contour, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_contour_into(
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
    pub fn uncheck_triangulate_shape(
        &mut self,
        shape: &IntShape,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_shape_into(shape, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shape_into(
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
    pub fn uncheck_triangulate_shapes(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_shapes_into(shapes, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shapes_into(
        &mut self,
        shapes: &IntShapes,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        triangulation.points.clear();
        triangulation.indices.clear();

        let mut buffer = self.triangulation_buffer();
        for shape in shapes.iter() {
            self.uncheck_triangulate_shape_into(shape, delaunay, &mut buffer);
            triangulation.join(&buffer);
        }
        self.triangulation_buffer = Some(buffer)
    }

    #[inline]
    pub fn uncheck_triangulate_flat(
        &mut self,
        flat_buffer: &FlatContoursBuffer,
        delaunay: bool,
    ) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_flat_into(flat_buffer, delaunay, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_flat_into(
        &mut self,
        flat_buffer: &FlatContoursBuffer,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        self.builder.build_flat(flat_buffer);
        if delaunay {
            self.builder.delaunay_refine();
        }
        self.builder.feed_triangulation(triangulation);
    }

    #[inline]
    fn triangulation_buffer(&mut self) -> IntTriangulation<I> {
        if let Some(buffer) = self.triangulation_buffer.take() {
            buffer
        } else {
            IntTriangulation::with_capacity(64)
        }
    }
}
