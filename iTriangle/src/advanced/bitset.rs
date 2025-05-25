use alloc::vec;
use alloc::vec::Vec;

pub struct IndexBitSet {
    chunks: Vec<u64>,
}

impl IndexBitSet {

    #[inline]
    pub fn with_size(count: usize) -> Self {
        let len = count >> 6;
        Self {
            chunks: vec![0; len],
        }
    }

    #[inline]
    pub fn clear_and_resize(&mut self, count: usize) {
        self.chunks.clear();
        let new_len = count >> 6;
        self.chunks.resize(new_len, 0);
    }

    #[inline]
    pub fn insert(&mut self, index: usize) {
        let chunk_index = index >> 6;
        if chunk_index >= self.chunks.len() {
            self.chunks.resize(chunk_index + 1, 0);
        }
        let bit_index = 63 & index;
        self.chunks[chunk_index] |= 1 << bit_index;
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        let chunk_index = index >> 6;
        if chunk_index < self.chunks.len() {
            let bit_index = 63 & index;
            self.chunks[chunk_index] &= !(1 << bit_index);
        }
    }

    #[inline]
    pub fn read_and_clean(&mut self, indices: &mut Vec<usize>) {
        let count = self.chunks.iter().map(|ch| ch.count_ones() as usize).sum::<usize>();
        indices.clear();
        if count == 0 {
            return;
        }

        let additional = count.saturating_sub(indices.capacity());
        if additional > 0 {
            indices.reserve(additional);
        }

        for (chunk_index, chunk) in self.chunks.iter_mut().enumerate() {
            if *chunk == 0 {
                continue;
            }
            let mut bits = *chunk;
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                indices.push((chunk_index << 6) + bit);
                bits &= !(1 << bit);
            }
            *chunk = 0;
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.iter().find(|&&ch|ch != 0).is_none()
    }

}

impl Default for IndexBitSet {
    fn default() -> Self {
        Self {
            chunks: vec![0; 8],
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::vec;
    use super::*;
    use std::collections::HashSet;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    #[test]
    fn test_empty() {
        let mut set = IndexBitSet::default();
        let mut out = Vec::new();
        set.read_and_clean(&mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_single_value() {
        let mut set = IndexBitSet::default();
        set.insert(42);
        let mut out = Vec::new();
        set.read_and_clean(&mut out);
        assert_eq!(out, vec![42]);
    }

    #[test]
    fn test_duplicates() {
        let mut set = IndexBitSet::default();
        set.insert(10);
        set.insert(10);
        set.insert(10);
        let mut out = Vec::new();
        set.read_and_clean(&mut out);
        assert_eq!(out, vec![10]);
    }

    #[test]
    fn test_ordered_values() {
        let mut set = IndexBitSet::default();
        for i in 0..100 {
            set.insert(i);
        }
        let mut out = Vec::new();
        set.read_and_clean(&mut out);
        let expected: Vec<_> = (0..100).collect();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_random_comparison_with_hashset() {
        let mut rng = StdRng::seed_from_u64(12345);
        let mut set = IndexBitSet::default();
        let mut hash = HashSet::new();

        for _ in 0..10_000 {
            let v = rng.random_range(0..10_000);
            set.insert(v);
            hash.insert(v);
        }

        let mut out = Vec::new();
        set.read_and_clean(&mut out);
        let set_from_index_set: HashSet<_> = out.into_iter().collect();

        assert_eq!(set_from_index_set, hash);
    }

    #[test]
    fn test_reuse_and_clean() {
        let mut set = IndexBitSet::default();
        let mut out = Vec::new();

        set.insert(1);
        set.insert(2);
        set.read_and_clean(&mut out);
        assert_eq!(out.len(), 2);

        // Second call should be empty now
        set.read_and_clean(&mut out);
        assert!(out.is_empty());

        // Reuse
        set.insert(5);
        set.insert(10);
        set.read_and_clean(&mut out);
        let out_set: HashSet<_> = out.iter().cloned().collect();
        assert_eq!(out_set, HashSet::from([5, 10]));
    }

    #[test]
    fn test_remove() {
        let mut set = IndexBitSet::default();
        set.insert(15);
        set.insert(100);
        set.insert(200);

        set.remove(100);
        let mut out = Vec::new();
        set.read_and_clean(&mut out);

        let expected = vec![15, 200];
        assert_eq!(out, expected);
    }
}
