use i_overlay::i_float::int::point::IntPoint;

#[derive(Debug, Clone, Copy)]
pub struct IndexPoint {
    pub index: usize,
    pub point: IntPoint,
}

impl IndexPoint {
    #[inline]
    pub fn new(index: usize, point: IntPoint) -> Self {
        Self { index, point }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self {
            index: usize::MAX,
            point: IntPoint::ZERO,
        }
    }
}

impl Default for IndexPoint {
    #[inline]
    fn default() -> Self {
        IndexPoint::empty()
    }
}