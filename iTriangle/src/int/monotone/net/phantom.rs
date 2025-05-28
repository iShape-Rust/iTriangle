use alloc::vec::Vec;

#[derive(Copy, Clone)]
pub(crate) struct PhantomHandler {
    pub(crate) vertex: usize,
    pub(crate) triangle: usize,
}

pub(crate) struct PhantomEdgePool {
    buffer: Vec<PhantomHandler>,
    unused: Vec<usize>,
}

impl PhantomEdgePool {
    const INIT_LEN: usize = 8;

    const EMPTY: PhantomHandler = PhantomHandler {
        vertex: usize::MAX,
        triangle: usize::MAX,
    };

    pub(crate) fn new() -> Self {
        let mut store = Self {
            buffer: Vec::with_capacity(Self::INIT_LEN),
            unused: Vec::with_capacity(Self::INIT_LEN),
        };
        store.reserve(Self::INIT_LEN);
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
    pub(crate) fn get(&self, index: usize) -> Option<PhantomHandler> {
        let item = self.buffer[index];
        if item.triangle == usize::MAX {
            None
        } else {
            Some(item)
        }
    }

    #[inline]
    pub(crate) fn register_phantom_link(&mut self, index: usize, handler: PhantomHandler) {
        debug_assert!(self.buffer[index].triangle == usize::MAX);
        self.buffer[index] = handler;
    }

    #[inline]
    pub(crate) fn alloc_phantom_index(&mut self) -> usize {
        if self.unused.is_empty() {
            self.reserve(self.unused.capacity());
        }
        self.unused.pop().unwrap()
    }

    #[inline]
    pub(crate) fn free_phantom_index(&mut self, index: usize) {
        self.buffer[index] = Self::EMPTY;
        self.unused.push(index)
    }
}