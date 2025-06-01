pub(crate) struct Test {
    pub(crate) count: usize,
    pub(crate) repeat: usize
}

impl Test {
    pub(crate) fn new(count: usize, repeat: usize) -> Self {
        Self {
            count,
            repeat,
        }
    }
}