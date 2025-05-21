use alloc::vec::Vec;

#[derive(Copy, Clone)]
pub(super) struct PhantomHandler {
    pub(super) vertex: usize,
    pub(super) triangle: usize,
}

pub(super) struct PhantomEdgePool {
    buffer: Vec<PhantomHandler>,
    unused: Vec<usize>,
}

impl PhantomEdgePool {
    const EMPTY: PhantomHandler = PhantomHandler {
        vertex: usize::MAX,
        triangle: usize::MAX,
    };

    pub(super) fn new() -> Self {
        let capacity = 4;
        let mut store = Self {
            buffer: Vec::with_capacity(capacity),
            unused: Vec::with_capacity(capacity),
        };
        store.reserve(capacity);
        store
    }

    #[inline]
    fn reserve(&mut self, length: usize) {
        debug_assert!(length > 0);
        let n = self.buffer.len();
        self.buffer.reserve(length);
        self.buffer.resize(self.buffer.len() + length, Self::EMPTY);
        self.unused.reserve(length);
        self.unused.extend((n..n + length).rev());
    }

    #[inline]
    pub(super) fn get(&self, index: usize) -> Option<PhantomHandler> {
        let item = self.buffer[index];
        if item.triangle == usize::MAX {
            None
        } else {
            Some(item)
        }
    }

    #[inline]
    pub(super) fn register_phantom_link(&mut self, index: usize, handler: PhantomHandler) {
        debug_assert!(self.buffer[index].triangle == usize::MAX);
        self.buffer[index] = handler;
    }

    #[inline]
    pub(super) fn alloc_phantom_index(&mut self) -> usize {
        if self.unused.is_empty() {
            self.reserve(self.unused.capacity());
        }
        self.unused.pop().unwrap()
    }

    #[inline]
    pub(super) fn free_phantom_index(&mut self, index: usize) {
        self.buffer[index] = Self::EMPTY;
        self.unused.push(index)
    }
}