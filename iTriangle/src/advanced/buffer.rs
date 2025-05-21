use alloc::vec::Vec;
use crate::advanced::bitset::IndexBitSet;

pub struct DelaunayBuffer {
    bitset: Option<IndexBitSet>,
    indices: Option<Vec<usize>>
}

impl DelaunayBuffer {
    #[inline]
    pub fn new() -> Self {
        Self { bitset: None, indices: None }
    }

    #[inline]
    pub(crate) fn take_bitset(&mut self) -> IndexBitSet {
        if let Some(bitset) = self.bitset.take() {
            bitset
        } else {
            IndexBitSet::new()
        }
    }

    #[inline]
    pub(crate) fn take_indices(&mut self) -> Vec<usize> {
        if let Some(indices) = self.indices.take() {
            indices
        } else {
            Vec::new()
        }
    }

    #[inline]
    pub(crate) fn set(&mut self, bitset: IndexBitSet, indices: Vec<usize>) {
        self.bitset = Some(bitset);
        self.indices = Some(indices);
    }
}