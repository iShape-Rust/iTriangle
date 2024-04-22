use i_float::point::IntPoint;

#[derive(Debug, Clone, Copy)]
pub struct DVertex {
    pub(crate) index: usize,
    pub(crate) point: IntPoint
}

impl DVertex {
    pub const fn empty() -> Self {
        Self {
            index: usize::MAX,
            point: IntPoint::ZERO
        }
    }

    pub fn new(index: usize, point: IntPoint) -> Self {
        Self { index, point }
    }
}