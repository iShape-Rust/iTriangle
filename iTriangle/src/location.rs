use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TriangleIndex(usize);

impl TriangleIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }

    pub fn to_vertex_indices(&self) -> [usize; 3] {
        let offset = 3 * self.0;
        [offset, offset + 1, offset + 2]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointLocationInTriangulation {
    Outside,
    InsideTriangle(TriangleIndex),
    OnExteriorEdge(TriangleIndex),
    OnInteriorEdge(TriangleIndex, TriangleIndex),
    OnVertex(Vec<TriangleIndex>),
}
