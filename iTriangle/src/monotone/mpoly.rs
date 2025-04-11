#[derive(Debug, Clone, Copy)]
pub (super) struct MPoly {
    pub next: usize,
    pub prev: usize
}

impl MPoly {

    #[inline]
    pub (super) fn new(start: usize) -> Self {
        Self { next: start, prev: start }
    }

    #[inline]
    pub (super) fn next_prev(next: usize, prev: usize) -> Self {
        Self { next, prev }
    }
}