use alloc::vec::Vec;
use crate::advanced::bitset::IndexBitSet;

#[derive(Default)]
pub struct DelaunayBuffer {
    pub(crate) bitset: Option<IndexBitSet>,
    pub(crate) indices: Option<Vec<usize>>
}

impl DelaunayBuffer {
    #[inline]
    pub fn new() -> Self {
        Self { bitset: None, indices: None }
    }
}