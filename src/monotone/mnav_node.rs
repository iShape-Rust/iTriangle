use i_float::fix_vec::FixVec;
use crate::delaunay::vertex::DVertex;
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
        next: usize::MAX,
        index: usize::MAX,
        prev: usize::MAX,
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
    fn is_intersect_next_reverse(&self, p0: FixVec, p1: FixVec, start: usize) -> bool;
    fn is_intersect_prev_reverse(&self, p0: FixVec, p1: FixVec, start: usize) -> bool;
    fn is_intersect(&self, p0: FixVec, p1: FixVec, next: usize, prev: usize) -> bool;
}

impl MNavNodeArray for Vec<MNavNode> {

    fn new_next(&mut self, a: usize, b: usize) -> ABVert {
        let mut a_vert = self[a];
        let mut b_vert = self[b];

        let count = self.len();

        let new_a = MNavNode::new(count + 1, count, a_vert.prev, a_vert.vert.clone());
        self.push(new_a.clone());

        self[a_vert.prev].next = count;

        let new_b = MNavNode::new(b_vert.next, count + 1, count, b_vert.vert.clone());
        self.push(new_b.clone());

        self[b_vert.next].prev = count + 1;

        a_vert.prev = b;
        b_vert.next = a;

        self[a] = a_vert;
        self[b] = b_vert;

        ABVert::new(new_a, new_b)
    }

    fn is_intersect_next_reverse(&self, p0: FixVec, p1: FixVec, start: usize) -> bool {
        let mut n = self[start];
        let stop = p0.x;
        let v = p1 - p0;
        while n.vert.point.x <= stop {
            let s = v.unsafe_cross_product(n.vert.point - p0);
            if s >= 0 {
                return true;
            }
            n = self[n.prev];
        }

        false
    }

    fn is_intersect_prev_reverse(&self, p0: FixVec, p1: FixVec, start: usize) -> bool {
        let mut n = self[start];
        let stop = p0.x;
        let v = p1 - p0;
        while n.vert.point.x > stop {
            let s = v.unsafe_cross_product(n.vert.point - p0);
            if s >= 0 {
                return true;
            }
            n = self[n.next];
        }

        false
    }

    fn is_intersect(&self, p0: FixVec, p1: FixVec, next: usize, prev: usize) -> bool {
        let is_next = self.is_intersect_next_reverse(p0, p1, next);
        let is_prev = self.is_intersect_prev_reverse(p0, p1, prev);

        is_next || is_prev
    }
}