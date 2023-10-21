use i_float::fix_vec::FixVec;
use i_shape::triangle::Triangle;
use crate::flip_shape::FlipShape;
use crate::monotone::mnav_node::{MNavNode, MNavNodeArray};
use crate::monotone::mpoly::MPoly;
use crate::monotone::mslice_buffer::MSlice;
use crate::monotone::nlayout::{MNodeType, MSpecialNode};

enum MLayoutStatus {
    Empty,
    Success,
    Fail
}

struct MLayout {
    start_list: Vec<usize>,
    nav_nodes: Vec<MNavNode>,
    slice_list: Vec<MSlice>,
    status: MLayoutStatus
}

impl MLayout {

    fn fail() -> Self {
        MLayout {
            start_list: Vec::new(),
            nav_nodes: Vec::new(),
            slice_list: Vec::new(),
            status: MLayoutStatus::Fail
        }
    }

    fn empty() -> Self {
        MLayout{
            start_list: Vec::new(),
            nav_nodes: Vec::new(),
            slice_list: Vec::new(),
            status: MLayoutStatus::Empty
        }
    }

}

struct NavIndex {
    next: usize,
    prev: usize
}

enum MType {
    Direct,
    Next,
    Prev
}

struct MSolution {
    mtype: MType,
    a: usize,
    b: usize,
    node_index: usize
}

impl FlipShape {

    fn mlayout(&self) -> MLayout {
        let nlayout = self.nlayout();

        let mut specs = nlayout.spec_nodes;

        if specs.is_empty() {
            return MLayout::empty();
        }

        let first = specs[0];
        if first.node_type() != MNodeType::Start {
            return MLayout::empty();
        }


        let mut start_list = Vec::new();
        start_list.push(first.index);

        let mut navs = nlayout.nav_nodes;

        let mut slice_list = Vec::new();
        let mut mpolies = Vec::new();

        mpolies.push(MPoly::new(navs[first.index].index));

        let mut j = 1;

        while j < specs.len() && !mpolies.is_empty() {
            let spec = specs[j];

            let px = Self::fill(&mut mpolies, &navs, spec.sort);

            let nav = navs[spec.index];

            match spec.node_type {
                MNodeType::End => {
                    if !(px.next == px.prev && px.next != usize::MAX) {
                        return MLayout::fail();
                    }
                    let p_index = px.next;
                    mpolies.remove(p_index);
                }
                MNodeType::Start => {
                    start_list.push(spec.index);
                    mpolies.push(MPoly::new(nav.index));
                }
                MNodeType::Split => {
                    let mut p_index = usize::MAX;
                    for i in 0..mpolies.len() {
                        if Self::is_contain(mpolies[i], nav.vert.point, &navs) {
                            p_index = i;
                            break;
                        }
                    }

                    if p_index == usize::MAX {
                        return MLayout::fail();
                    }

                    let mpoly = mpolies[p_index];

                    let sv = nav;

                    if mpoly.next == mpoly.prev {
                        let start = mpoly.next;
                        let s = navs.new_next(start, sv.index);
                        slice_list.push(s.slice());

                        mpolies[p_index] = MPoly::next_prev(start, sv.index);
                        mpolies.push(MPoly::next_prev(s.b.index, s.a.index));

                        start_list.push(s.a.index);
                    } else {
                        let a = navs[mpoly.next].vert.point;
                        let b = navs[mpoly.prev].vert.point;

                        let sp = nav.vert.point;

                        let is_next = if a.x == b.x {
                            sp.sqr_distance(a) < sp.sqr_distance(b)
                        } else {
                            a.x > b.x
                        };

                        if is_next {
                            let nv = mpoly.next;

                            let s = navs.new_next(sv.index, nv);
                            slice_list.push(s.slice());

                            mpolies[p_index] = MPoly::new(s.b.index);
                            start_list.push(s.b.index);

                            mpolies.push(MPoly::next_prev(nv, mpoly.prev));
                        } else {
                            let nv = mpoly.prev;

                            let s = navs.new_next(nv, sv.index);
                            slice_list.push(s.slice());

                            // next
                            mpolies[p_index] = MPoly::next_prev(mpoly.next, sv.index);

                            // prev
                            mpolies.push(MPoly::next_prev(s.b.index, s.a.index));

                            start_list.push(s.a.index);
                        }
                    }
                }
                MNodeType::Merge => {
                    if px.next == usize::MAX || px.prev == usize::MAX {
                        return MLayout::fail();
                    }

                    let next_poly = mpolies[px.next];
                    let prev_poly = mpolies[px.prev];

                    let prev = navs[prev_poly.prev];
                    let next = navs[next_poly.next];

                    let ms = Self::find_node_to_merge(prev, next, nav, j + 1, &specs, &navs);

                    match ms.mtype {
                        MType::Direct => {
                            let r_node = specs.remove(ms.node_index);

                            let s = navs.new_next(ms.a, ms.b);
                            slice_list.push(s.slice());

                            if r_node.node_type == MNodeType::End {
                                if px.next > px.prev {
                                    mpolies.remove(px.next);
                                    mpolies.remove(px.prev);
                                } else {
                                        mpolies.remove(px.prev);
                                        mpolies.remove(px.next);
                                }
                            } else {
                                mpolies[px.next] = MPoly::next_prev(next_poly.next, ms.b);

                                mpolies[px.prev] = MPoly::next_prev(s.b.index, prev_poly.prev)
                            }
                        }
                        MType::Next => {
                            let s = navs.new_next(ms.b, ms.a);
                            slice_list.push(s.slice());
                            mpolies.remove(px.next);
                        }
                        MType::Prev => {
                            let s = navs.new_next(ms.a, ms.b);
                            slice_list.push(s.slice());
                            mpolies.remove(px.prev);
                        }
                    }
                }
            }
            j += 1
        }

        MLayout::empty()
    }

    fn fill(mpolies: &mut Vec<MPoly>, verts: &Vec<MNavNode>, stop: i64) -> NavIndex {

        let mut next_poly_ix = usize::MAX;
        let mut prev_poly_ix = usize::MAX;
        for i in 0..mpolies.len() {
            let mut mpoly = mpolies[i];

            let mut n0 = verts[mpoly.next];
            let mut n1 = verts[n0.next];

            while n1.vert.point.bit_pack() < stop  {
                n0 = n1;
                n1 = verts[n1.next];
            }

            if n1.vert.point.bit_pack() == stop {
                mpoly.next = n1.index;
                prev_poly_ix = i;
            } else {
                mpoly.next = n0.index;
            }

            let mut p0 = verts[mpoly.prev];
            let mut p1 = verts[p0.prev];

            while p1.vert.point.bit_pack() < stop {
                p0 = p1;
                p1 = verts[p1.prev];
            }

            if p1.vert.point.bit_pack() == stop {
                mpoly.prev = p1.index;
                next_poly_ix = i;
            } else {
                mpoly.prev = p0.index;
            }

            mpolies[i] = mpoly;
        }

        return NavIndex { next: next_poly_ix, prev: prev_poly_ix }
    }

    fn find_node_to_merge(prev: MNavNode, next: MNavNode, merge: MNavNode, start_node: usize, specs: &Vec<MSpecialNode>, navs: &Vec<MNavNode>) -> MSolution {
        let a0 = next.vert.point;
        let a1 = navs[next.next].vert.point;
        let b1 = navs[prev.prev].vert.point;
        let b0 = prev.vert.point;

        let m = merge.vert.point;

        // check inner nodes
        if start_node < specs.len() {

            // 3 triangles:
            // top: m, a0, a1
            // middle: m, a1, b1
            // bottom: m, b1, b0

            let min_x = a1.x.min(b1.x);

            let mut i = start_node;

            while i < specs.len() {
                let spec = specs[i];
                let nav = navs[spec.index];
                let p = nav.vert.point;
                if p.x > min_x {
                    break;
                }
                if spec.node_type == MNodeType::Split || spec.node_type == MNodeType::End {
                    let is_contain = Triangle::is_contain(p, m, a0, a1)
                        || Triangle::is_contain(p, m, a1, b1)
                        || Triangle::is_contain(p, m, b1, b0);

                    if is_contain {
                        return MSolution{ mtype: MType::Direct, a: merge.index, b: nav.index, node_index: i }
                    }
                }
                i += 1;
            }
        }

        let compare = if a1.x == b1.x {
            m.sqr_distance(a1) < m.sqr_distance(b1)
        } else { a1.x < b1.x };

        if compare {
            MSolution{ mtype: MType::Next, a: merge.index, b: next.next, node_index: usize::MAX }
        } else {
            MSolution{ mtype: MType::Prev, a: merge.index, b: prev.prev, node_index: usize::MAX }
        }
    }

    fn is_contain(mpoly: MPoly, point: FixVec, navs: &Vec<MNavNode>) -> bool {
        let a0 = navs[mpoly.next];
        let a1 = navs[a0.next];

        let b0 = navs[mpoly.prev];
        let b1 = navs[b0.prev];

        Self::is_contain_point(point, a0.vert.point, a1.vert.point, b0.vert.point, b1.vert.point)
    }

    fn is_contain_point(point: FixVec, a0: FixVec, a1: FixVec, b0: FixVec, b1: FixVec) -> bool {
        let sa = (a1 - a0).unsafe_cross_product(point - a0);
        let sb = (b1 - b0).unsafe_cross_product(point - b0);

        sa <= 0 && sb >= 0
    }

}