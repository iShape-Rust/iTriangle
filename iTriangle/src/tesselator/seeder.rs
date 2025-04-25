use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub(super) struct Seeder {
    radius: u64,
    sqr_radius: u64,
    shapes: IntShapes
}

impl Seeder {

    #[inline]
    pub(super) fn new(radius: i32) -> Self {
        let r = radius.unsigned_abs().max(1) as u64;
        Self {
            radius: r,
            sqr_radius: r.pow(2),
            shapes: Vec::new()
        }
    }

    #[inline]
    pub(super) fn add_contour(&mut self, contour: &IntContour) {
        self.add_contour(contour)
    }

    #[inline]
    pub(super) fn add_shape(&mut self, shape: &IntShape) {
        self.add_shape(shape)
    }

    #[inline]
    pub(super) fn add_shapes(&mut self, shapes: &IntShapes) {
        self.add_shapes(shapes)
    }

    // pub(super) fn seed(radius: i32) -> Vec<Vec<IntPoint>> {
    //
    // }

}

