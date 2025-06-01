use crate::int::earcut::earcut_64::Earcut64;
use crate::int::monotone::triangulator::MonotoneTriangulator;
use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use crate::int::validation::Validation;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::Overlay;
use i_overlay::core::solver::Solver;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub struct IntTriangulator<I> {
    pub overlay: Overlay,
    pub fill_rule: FillRule,
    pub earcut: bool,
    pub delaunay: bool,
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
            earcut: true,
            delaunay: false,
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
    pub fn triangulate_contour(&mut self, contour: &IntContour) -> IntTriangulation<I> {
        match self.overlay.simplify_contour(contour, self.fill_rule) {
            None => self.uncheck_triangulate_contour(contour),
            Some(shapes) => self.uncheck_triangulate_shapes(&shapes),
        }
    }

    #[inline]
    pub fn triangulate_contour_into(
        &mut self,
        contour: IntContour,
        triangulation: &mut IntTriangulation<I>,
    ) {
        match self.overlay.simplify_contour(&contour, self.fill_rule) {
            None => self.uncheck_triangulate_contour_into(&contour, triangulation),
            Some(shapes) => self.uncheck_triangulate_shapes_into(&shapes, triangulation),
        }
    }

    #[inline]
    pub fn triangulate_shape(&mut self, shape: &IntShape) -> IntTriangulation<I> {
        match self.overlay.simplify_shape(shape, self.fill_rule) {
            None => self.uncheck_triangulate_shape(shape),
            Some(shapes) => self.uncheck_triangulate_shapes(&shapes),
        }
    }

    #[inline]
    pub fn triangulate_shape_into(
        &mut self,
        shape: &IntShape,
        triangulation: &mut IntTriangulation<I>,
    ) {
        match self.overlay.simplify_shape(shape, self.fill_rule) {
            None => self.uncheck_triangulate_shape_into(shape, triangulation),
            Some(shapes) => self.uncheck_triangulate_shapes_into(&shapes, triangulation),
        }
    }

    #[inline]
    pub fn triangulate_shapes(&mut self, shapes: &IntShapes) -> IntTriangulation<I> {
        let simple = self.overlay.simplify_shapes(shapes, self.fill_rule);
        self.uncheck_triangulate_shapes(&simple)
    }

    #[inline]
    pub fn triangulate_shapes_into(
        &mut self,
        shapes: &IntShapes,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let simple = self.overlay.simplify_shapes(shapes, self.fill_rule);
        self.uncheck_triangulate_shapes_into(&simple, triangulation);
    }

    #[inline]
    pub fn triangulate_flat(&mut self, flat: &mut FlatContoursBuffer) -> IntTriangulation<I> {
        self.overlay.simplify_flat_buffer(flat, self.fill_rule);
        self.uncheck_triangulate_flat(flat)
    }

    #[inline]
    pub fn triangulate_flat_into(
        &mut self,
        flat: &mut FlatContoursBuffer,
        triangulation: &mut IntTriangulation<I>,
    ) {
        self.overlay.simplify_flat_buffer(flat, self.fill_rule);
        self.uncheck_triangulate_flat_into(flat, triangulation);
    }
}

impl<I: IndexType> IntTriangulator<I> {
    #[inline]
    pub fn uncheck_triangulate_contour(&mut self, contour: &IntContour) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_contour_into(contour, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_contour_into(
        &mut self,
        contour: &IntContour,
        triangulation: &mut IntTriangulation<I>,
    ) {
        if self.delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            if self.earcut && contour.is_earcut_compatible() {
                contour.earcut_net_triangulate_into(&mut raw);
            } else {
                self.triangulator
                    .contour_into_net_triangulation(contour, None, &mut raw);
            }
            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else if self.earcut && contour.is_earcut_compatible() {
            contour.earcut_flat_triangulate_into(triangulation);
        } else {
            self.triangulator
                .contour_into_flat_triangulation(contour, triangulation);
        }
    }

    #[inline]
    pub fn uncheck_triangulate_shape(&mut self, shape: &IntShape) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_shape_into(shape, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shape_into(
        &mut self,
        shape: &IntShape,
        triangulation: &mut IntTriangulation<I>,
    ) {
        if shape.len() == 1 {
            self.uncheck_triangulate_contour_into(&shape[0], triangulation);
            return;
        }

        if self.delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            self.triangulator
                .shape_into_net_triangulation(shape, None, &mut raw);
            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else {
            self.triangulator
                .shape_into_flat_triangulation(shape, triangulation);
        }
    }

    #[inline]
    pub fn uncheck_triangulate_shapes(&mut self, shapes: &IntShapes) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_shapes_into(shapes, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_shapes_into(
        &mut self,
        shapes: &IntShapes,
        triangulation: &mut IntTriangulation<I>,
    ) {
        if shapes.len() == 1 {
            self.uncheck_triangulate_shape_into(&shapes[0], triangulation);
            return;
        }
        
        triangulation.points.clear();
        triangulation.indices.clear();

        let mut buffer = self.shapes_buffer.take().unwrap_or_default();
        for shape in shapes.iter() {
            self.uncheck_triangulate_shape_into(shape, &mut buffer);
            triangulation.join(&buffer);
        }
        self.shapes_buffer = Some(buffer)
    }

    #[inline]
    pub fn uncheck_triangulate_flat(
        &mut self,
        flat_buffer: &FlatContoursBuffer,
    ) -> IntTriangulation<I> {
        let mut triangulation = Default::default();
        self.uncheck_triangulate_flat_into(flat_buffer, &mut triangulation);
        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_flat_into(
        &mut self,
        flat: &FlatContoursBuffer,
        triangulation: &mut IntTriangulation<I>,
    ) {
        if flat.is_empty() {
            triangulation.reserve_and_clear(0);
            return;
        }

        if self.delaunay {
            let mut raw = self.raw_buffer.take().unwrap_or_default();
            if self.earcut && flat.is_earcut_compatible() {
                flat.as_first_contour()
                    .earcut_net_triangulate_into(&mut raw);
            } else {
                self.triangulator
                    .flat_into_net_triangulation(flat, &mut raw);
            }

            triangulation.fill_with_raw(&raw);
            self.raw_buffer = Some(raw);
        } else if self.earcut && flat.is_earcut_compatible() {
            flat.as_first_contour()
                .earcut_flat_triangulate_into(triangulation);
        } else {
            self.triangulator
                .flat_into_flat_triangulation(flat, triangulation);
        }
    }
}

trait Earcut64Compatible {
    fn is_earcut_compatible(&self) -> bool;
}

impl Earcut64Compatible for FlatContoursBuffer {
    #[inline(always)]
    fn is_earcut_compatible(&self) -> bool {
        self.is_single_contour() && self.points.len() <= 64
    }
}

impl Earcut64Compatible for [IntPoint] {
    #[inline(always)]
    fn is_earcut_compatible(&self) -> bool {
        self.len() <= 64
    }
}
