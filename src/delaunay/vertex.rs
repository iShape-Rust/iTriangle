use i_float::fix_vec::FixVec;

#[derive(Debug, Clone, Copy)]
pub struct DVertex {
    pub(crate) index: usize,
    pub(crate) point: FixVec
}

impl DVertex {
    pub const fn empty() -> Self {
        Self {
            index: usize::MAX,
            point: FixVec::ZERO
        }
    }

    pub fn new(index: usize, point: FixVec) -> Self {
        Self { index, point }
    }
}