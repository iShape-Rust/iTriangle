use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use crate::int::validation::Validation;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::Overlay;
use i_overlay::core::solver::Solver;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::monotone::triangulator::MonotoneTriangulator;

pub struct IntTriangulator<I> {
    overlay: Overlay,
    fill_rule: FillRule,
    triangulator: MonotoneTriangulator,
    shapes_buffer: Option<IntTriangulation<I>>,
    raw_buffer: Option<RawIntTriangulation>,
}

impl<I: IndexType> IntTriangulator<I> {
    #[inline]
    pub fn new(max_points_count: usize, validation: Validation, solver: Solver) -> Self {
        Self {
            overlay: Overlay::new_custom(max_points_count, validation.options, solver),
            fill_rule: validation.fill_rule,
            triangulator: MonotoneTriangulator::default(),
            raw_buffer: None,
            shapes_buffer: None,
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
        match self.overlay.simplify_contour(contour, self.fill_rule) {
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
        if delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            self.triangulator.contour_into_net_triangulation(contour, None, &mut raw);

            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else {
            self.triangulator.contour_into_flat_triangulation(contour, None, triangulation);
        }
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
        if delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            self.triangulator.shape_into_net_triangulation(shape, None, &mut raw);

            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else {
            self.triangulator.shape_into_flat_triangulation(shape, None, triangulation);
        }
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

        let mut buffer = self.shapes_buffer.take().unwrap_or_default();
        for shape in shapes.iter() {
            self.uncheck_triangulate_shape_into(shape, delaunay, &mut buffer);
            triangulation.join(&buffer);
        }
        self.shapes_buffer = Some(buffer)
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
        flat: &FlatContoursBuffer,
        delaunay: bool,
        triangulation: &mut IntTriangulation<I>,
    ) {
        if flat.is_empty() {
            triangulation.reserve_and_clear(0);
           return;
        }

        if delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            self.triangulator.flat_into_net_triangulation(flat, &mut raw);

            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else {
            self.triangulator.flat_into_flat_triangulation(flat, triangulation);
        }
    }
}
