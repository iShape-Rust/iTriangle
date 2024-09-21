use crate::delaunay::vertex::DVertex;
use crate::index::NIL_INDEX;
use crate::monotone::mslice_buffer::MSlice; // To perform vector subtraction

#[derive(Clone, Debug, Copy)]
pub struct MNavNode {
    pub(crate) next: usize,
    pub(crate) index: usize,
    pub(crate) prev: usize,
    pub(crate) vert: DVertex,
}

impl MNavNode {
    pub const EMPTY: MNavNode = MNavNode {
        next: NIL_INDEX,
        index: NIL_INDEX,
        prev: NIL_INDEX,
        vert: DVertex::empty()
    };

    pub fn new(next: usize, index: usize, prev: usize, vert: DVertex) -> Self {
        Self { next, index, prev, vert }
    }
}

#[derive(Clone, Debug, Copy)]
pub(super) struct ABVert {
    pub(super) a: MNavNode,
    pub(super) b: MNavNode
}

impl ABVert {
    pub(super) fn new(a: MNavNode, b: MNavNode) -> Self {
        Self { a, b }
    }

    pub(super) fn slice(&self) -> MSlice {
        MSlice::new(self.a.vert.index, self.b.vert.index)
    }
}

pub(super) trait MNavNodeArray {
    fn new_next(&mut self, a: usize, b: usize) -> ABVert;
}

impl MNavNodeArray for Vec<MNavNode> {

    fn new_next(&mut self, a: usize, b: usize) -> ABVert {
        let mut a_vert = self[a];
        let mut b_vert = self[b];

        let count = self.len();

        let new_a = MNavNode::new(count + 1, count, a_vert.prev, a_vert.vert);
        self.push(new_a);

        self[a_vert.prev].next = count;

        let new_b = MNavNode::new(b_vert.next, count + 1, count, b_vert.vert);
        self.push(new_b);

        self[b_vert.next].prev = count + 1;

        a_vert.prev = b;
        b_vert.next = a;

        self[a] = a_vert;
        self[b] = b_vert;

        ABVert::new(new_a, new_b)
    }
}