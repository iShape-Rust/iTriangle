use crate::advanced::bitset::IndexBitSet;
use alloc::vec::Vec;

#[derive(Default)]
pub struct DelaunayBuffer {
    pub(crate) bitset: Option<IndexBitSet>,
    pub(crate) indices: Option<Vec<usize>>,
}

impl DelaunayBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            bitset: None,
            indices: None,
        }
    }
}
