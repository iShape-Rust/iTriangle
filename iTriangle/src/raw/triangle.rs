use crate::raw::vertex::IndexPoint;

#[derive(Debug, Clone, Copy)]
pub struct PlainTriangle {
    pub vertices: [IndexPoint; 3],
    pub neighbors: [usize; 3],
}

impl PlainTriangle {

    #[inline]
    pub(super) fn abc(a: IndexPoint, b: IndexPoint, c: IndexPoint) -> Self {
        Self {
            vertices: [a, b, c],
            neighbors: [usize::MAX; 3],
        }
    }

    #[inline]
    pub(super) fn other_vertex(&self, a: usize, b: usize) -> usize {
        if self.vertices[0].index != a && self.vertices[0].index != b {
            0
        } else if self.vertices[1].index != a && self.vertices[1].index != b {
            1
        } else {
            2
        }
    }
}