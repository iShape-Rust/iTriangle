use i_float::fix_vec::FixVec;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DVType {
    Origin,
    ExtraPath,
    ExtraInner,
    ExtraTessellated
}

impl DVType {
    pub fn is_path(&self) -> bool {
        matches!(self, DVType::Origin | DVType::ExtraPath)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DVertex {
    pub (crate) index: usize,
    pub (crate) point: FixVec,
    pub (crate) dv_type: DVType
}

impl DVertex {

    pub const fn empty() -> Self {
        Self {
            index: usize::MAX,
            point: FixVec::ZERO,
            dv_type: DVType::Origin
        }
    }

    pub fn new(index: usize, point: FixVec, dv_type: DVType) -> Self {
        Self { index, point, dv_type }
    }
}