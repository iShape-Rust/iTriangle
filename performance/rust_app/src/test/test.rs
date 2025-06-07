pub(crate) struct TestData {
    pub(crate) count: usize,
    pub(crate) repeat: usize
}

impl TestData {
    pub(crate) fn new(count: usize, repeat: usize) -> Self {
        Self {
            count,
            repeat,
        }
    }
}