use i_float::bit_pack::{BitPack, BitPackVec};
use i_float::fix_vec::FixVec;
use i_float::triangle::Triangle;
use i_shape::fix_shape::FixShape;
use crate::index::{Index, NIL_INDEX};
use crate::monotone::mnav_node::{MNavNode, MNavNodeArray};
use crate::monotone::mpoly::MPoly;
use crate::monotone::mslice_buffer::MSlice;
use crate::monotone::node_layout::{MNodeType, MSpecialNode, ShapeNodeLayout};

#[derive(Clone, Debug, Copy, PartialEq)]
pub(crate) enum MonotoneLayoutStatus {
    Empty,
    Success,
    Fail
}

pub(crate) struct MonotoneLayout {
    pub(crate) start_list: Vec<usize>,
    pub(crate) nav_nodes: Vec<MNavNode>,
    pub(crate) slice_list: Vec<MSlice>,
    status: MonotoneLayoutStatus
}

impl MonotoneLayout {

    pub(crate) fn status(&self) -> MonotoneLayoutStatus {
        self.status
    }

    fn fail() -> Self {
        MonotoneLayout {
            start_list: Vec::new(),
            nav_nodes: Vec::new(),
            slice_list: Vec::new(),
            status: MonotoneLayoutStatus::Fail
        }
    }

    fn empty() -> Self {
        MonotoneLayout {
            start_list: Vec::new(),
            nav_nodes: Vec::new(),
            slice_list: Vec::new(),
            status: MonotoneLayoutStatus::Empty
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

pub trait ShapeLayout {
    fn monotone_layout(&self) -> MonotoneLayout;
}

impl ShapeLayout for FixShape {

    fn monotone_layout(&self) -> MonotoneLayout {
        let node_layout = self.node_layout();

        let mut specs = node_layout.spec_nodes;

        if specs.is_empty() {
            return MonotoneLayout::empty();
        }

        let first = specs[0];
        if first.node_type() != MNodeType::Start {
            return MonotoneLayout::empty();
        }

        let mut start_list = Vec::new();
        start_list.push(first.index);

        let mut navs = node_layout.nav_nodes;

        let mut slice_list = Vec::new();
        let mut mpolies = Vec::new();

        mpolies.push(MPoly::new(navs[first.index].index));

        let mut j = 1;

        while j < specs.len() && !mpolies.is_empty() {
            let spec = specs[j];

            let px = fill(&mut mpolies, &navs, spec.sort, spec.index);

            let nav = navs[spec.index];

            match spec.node_type {
                MNodeType::End => {
                    if !(px.next == px.prev && px.next.is_not_nil()) {
                        return MonotoneLayout::fail();
                    }
                    let p_index = px.next;
                    mpolies.remove(p_index);
                }
                MNodeType::Start => {
                    start_list.push(spec.index);
                    mpolies.push(MPoly::new(nav.index));
                }
                MNodeType::Split => {
                    let mut p_index = NIL_INDEX;
                    for i in 0..mpolies.len() {
                        if is_contain(mpolies[i], nav.vert.point, &navs) {
                            p_index = i;
                            break;
                        }
                    }

                    if p_index.is_nil() {
                        return MonotoneLayout::fail();
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
                    if px.next.is_nil() || px.prev.is_nil() {
                        return MonotoneLayout::fail();
                    }

                    let next_poly = mpolies[px.next];
                    let prev_poly = mpolies[px.prev];

                    let prev = navs[prev_poly.prev];
                    let next = navs[next_poly.next];

                    let ms = find_node_to_merge(prev, next, nav, j + 1, &specs, &navs);

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

        if j != specs.len() {
            MonotoneLayout::fail()
        } else {
            MonotoneLayout {
                start_list,
                nav_nodes: navs,
                slice_list,
                status: MonotoneLayoutStatus::Success,
            }
        }
    }
}

fn fill(mpolies: &mut Vec<MPoly>, verts: &Vec<MNavNode>, stop: BitPack, stop_index: usize) -> NavIndex {

    let mut next_poly_ix = NIL_INDEX;
    let mut prev_poly_ix = NIL_INDEX;
    for i in 0..mpolies.len() {
        let mut mpoly = mpolies[i];

        let mut n0 = verts[mpoly.next];
        let mut n1 = verts[n0.next];

        while n1.vert.point.bit_pack() < stop  {
            n0 = n1;
            n1 = verts[n1.next];
        }

        if n1.vert.index == stop_index {
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

        if p1.vert.index == stop_index {
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
    let va1 = navs[next.next].vert;
    let vb1 = navs[prev.prev].vert;
    let b0 = prev.vert.point;

    let m = merge.vert.point;

    // check inner nodes
    if start_node < specs.len() {

        // 3 triangles:
        // top: m, a0, a1
        // middle: m, a1, b1
        // bottom: m, b1, b0

        let min_x = va1.point.x.min(vb1.point.x);

        let mut i = start_node;

        while i < specs.len() {
            let spec = specs[i];
            let nav = navs[spec.index];
            let v = nav.vert;
            if v.point.x > min_x {
                break;
            }
            if spec.node_type == MNodeType::Split || spec.node_type == MNodeType::End {
                // if it end it can be unreachable (same point for different vertices!)
                let is_unreachable = v.point == va1.point && v.index != va1.index || v.point == vb1.point && v.index != vb1.index;
                if !is_unreachable {
                    let is_contain = Triangle::is_contain(v.point, m, a0, va1.point)
                        || Triangle::is_contain(v.point, m, va1.point, vb1.point)
                        || Triangle::is_contain(v.point, m, vb1.point, b0);

                    if is_contain {
                        return MSolution { mtype: MType::Direct, a: merge.index, b: nav.index, node_index: i }
                    }
                }
            }
            i += 1;
        }
    }

    let compare = if va1.point.x == vb1.point.x {
        m.sqr_distance(va1.point) < m.sqr_distance(vb1.point)
    } else { va1.point.x < vb1.point.x };

    if compare {
        MSolution{ mtype: MType::Next, a: merge.index, b: next.next, node_index: NIL_INDEX }
    } else {
        MSolution{ mtype: MType::Prev, a: merge.index, b: prev.prev, node_index: NIL_INDEX }
    }
}

fn is_contain(mpoly: MPoly, point: FixVec, navs: &Vec<MNavNode>) -> bool {
    let a0 = navs[mpoly.next];
    let a1 = navs[a0.next];

    let b0 = navs[mpoly.prev];
    let b1 = navs[b0.prev];

    is_contain_point(point, a0.vert.point, a1.vert.point, b0.vert.point, b1.vert.point)
}

fn is_contain_point(point: FixVec, a0: FixVec, a1: FixVec, b0: FixVec, b1: FixVec) -> bool {
    let sa = (a1 - a0).cross_product(point - a0);
    let sb = (b1 - b0).cross_product(point - b0);

    sa <= 0 && sb >= 0
}