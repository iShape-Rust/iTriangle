use std::vec;
use i_float::fix_vec::FixVec;
use i_shape::triangle::Triangle;
use crate::delaunay::delaunay::Delaunay;
use crate::delaunay::triangle::DTriangle;
use crate::delaunay::vertex::DVertex;
use crate::flip_shape::FlipShape;
use crate::monotone::mlayout::MLayoutStatus;
use crate::monotone::mnav_node::MNavNode;
use crate::monotone::mslice_buffer::MSliceBuffer;

#[derive(Debug, Clone, Copy)]
struct Edge {
    a: usize,               // vertex index
    b: usize,               // vertex index
    neighbor: usize         // prev triangle index
}

struct TriangleStack {
    edges: Vec<Edge>,
    triangles: Vec<DTriangle>,
    counter: usize
}

impl TriangleStack {

    fn with_count(count: usize) -> Self {
        let edges = Vec::with_capacity(8);
        let triangles = vec![DTriangle::new(); count];

        Self { edges, triangles, counter: 0 }
    }

    fn take_triangles(mut self) -> Vec<DTriangle> {
        if self.counter != self.triangles.len() {
            self.triangles.truncate(self.counter);
        }

        self.triangles
    }

    fn reset(&mut self) {
        self.edges.clear()
    }

    fn add(&mut self, a: DVertex, b: DVertex, c: DVertex) {
        if a.index == b.index || a.index == c.index || b.index == c.index {
            // ignore triangle with tween vertices
            return;
        }

        let mut triangle = DTriangle::abc(self.counter, a, b, c);

        if let Some(ac) = self.pop(a.index, c.index) {
            let mut neighbor = self.triangles[ac.neighbor];
            neighbor.neighbors[0] = triangle.index;
            triangle.neighbors[1] = neighbor.index;
            self.triangles[neighbor.index] = neighbor;
        }

        if let Some(ab) = self.pop(a.index, b.index) {
            let mut neighbor = self.triangles[ab.neighbor];
            neighbor.neighbors[0] = triangle.index;
            triangle.neighbors[2] = neighbor.index;
            self.triangles[neighbor.index] = neighbor;
        }

        self.edges.push(Edge { a: b.index, b: c.index, neighbor: triangle.index }); // bc is always slice

        self.triangles[triangle.index] = triangle;

        self.counter += 1;
    }

    fn pop(&mut self, a: usize, b: usize) -> Option<Edge> {
        if self.edges.is_empty() {
            return None;
        }
        let last = self.edges.len() - 1;
        let mut i = 0;
        while i <= last {
            let e = self.edges[i];
            if (e.a == a || e.a == b) && (e.b == a || e.b == b) {
                if i != last {
                    self.edges[i] = self.edges[last]
                }
                self.edges.pop();

                return Some(e);
            }
            i += 1;
        }
        None
    }
}


impl FlipShape {

    pub(crate) fn delaunay(&self) -> Option<Delaunay> {
        let layout = self.mlayout();

        if layout.status() != MLayoutStatus::Success {
            return None;
        }

        let holes_count = self.paths().len() - 1;
        let verts_count: usize = self.paths().iter().map(|path| path.len()).sum();
        let total_count = verts_count + holes_count * 2;

        let mut triangle_stack = TriangleStack::with_count(total_count);

        let mut links = layout.nav_nodes;
        for index in layout.start_list.iter() {
            Self::triangulate(*index, &mut links, &mut triangle_stack);
            triangle_stack.reset();
        }

        let mut points = vec!(FixVec::ZERO; links.len());
        for node in links.iter() {
            points[node.index] = node.vert.point;
        }

        let mut triangles = triangle_stack.take_triangles();

        let mut slice_buffer = MSliceBuffer::new(links.len(), &layout.slice_list);
        slice_buffer.add_connections(&mut triangles);

        let mut delaunay = Delaunay::new(points, triangles);

        delaunay.build();

        Some(delaunay)
    }

    fn triangulate(index: usize, links: &mut Vec<MNavNode>, triangle_stack: &mut TriangleStack) {
        let mut c = links[index];

        let mut a0 = links[c.next];
        let mut b0 = links[c.prev];

        while a0.index != b0.index {
            let a1 = links[a0.next];
            let b1 = links[b0.prev];

            let mut a_bit0 = a0.vert.point.bit_pack();
            let mut a_bit1 = a1.vert.point.bit_pack();
            if a_bit1 < a_bit0 {
                a_bit1 = a_bit0;
            }

            let mut b_bit0 = b0.vert.point.bit_pack();
            let mut b_bit1 = b1.vert.point.bit_pack();
            if b_bit1 < b_bit0 {
                b_bit1 = b_bit0;
            }

            if a_bit0 <= b_bit1 && b_bit0 <= a_bit1 {
                triangle_stack.add(c.vert, a0.vert, b0.vert);

                a0.prev = b0.index;
                b0.next = a0.index;
                links[a0.index] = a0;
                links[b0.index] = b0;

                if b_bit0 < a_bit0 {
                    c = b0;
                    b0 = b1;
                } else {
                    c = a0;
                    a0 = a1;
                }
            } else {
                if a_bit1 < b_bit1 {
                    let mut cx = c;
                    let mut ax0 = a0;
                    let mut ax1 = a1;
                    let mut ax1_bit = i64::MIN;
                    while ax1_bit < b_bit0 {
                        let is_cw_or_line = Triangle::is_cw_or_line(cx.vert.point, ax0.vert.point, ax1.vert.point);

                        if is_cw_or_line {
                            triangle_stack.add(ax0.vert, ax1.vert, cx.vert);

                            ax1.prev = cx.index;
                            cx.next = ax1.index;
                            links[cx.index] = cx;
                            links[ax1.index] = ax1;

                            if cx.index != c.index {
                                // move back
                                ax0 = cx;
                                cx = links[cx.prev];
                            } else {
                                // move forward
                                ax0 = ax1;
                                ax1 = links[ax1.next];
                            }
                        } else {
                            cx = ax0;
                            ax0 = ax1;
                            ax1 = links[ax1.next];
                        }
                        ax1_bit = ax1.vert.point.bit_pack();
                    }
                } else {
                    let mut cx = c;
                    let mut bx0 = b0;
                    let mut bx1 = b1;
                    let mut bx1_bit = i64::MIN;
                    while bx1_bit < a_bit0 {
                        let is_cw_or_line = Triangle::is_cw_or_line(cx.vert.point, bx1.vert.point, bx0.vert.point);
                        if is_cw_or_line {
                            triangle_stack.add(bx0.vert, cx.vert, bx1.vert);

                            bx1.next = cx.index;
                            cx.prev = bx1.index;
                            links[cx.index] = cx;
                            links[bx1.index] = bx1;

                            if cx.index != c.index {
                                // move back
                                bx0 = cx;
                                cx = links[cx.next];
                            } else {
                                // move forward
                                bx0 = bx1;
                                bx1 = links[bx0.prev];
                            }
                        } else {
                            cx = bx0;
                            bx0 = bx1;
                            bx1 = links[bx1.prev];
                        }
                        bx1_bit = bx1.vert.point.bit_pack();
                    }
                }

                c = links[c.index];
                a0 = links[c.next];
                b0 = links[c.prev];

                a_bit0 = a0.vert.point.bit_pack();
                b_bit0 = b0.vert.point.bit_pack();

                triangle_stack.add(c.vert, a0.vert, b0.vert);

                a0.prev = b0.index;
                b0.next = a0.index;
                links[a0.index] = a0;
                links[b0.index] = b0;

                if b_bit0 < a_bit0 {
                    c = b0;
                    b0 = links[b0.prev];
                } else {
                    c = a0;
                    a0 = links[a0.next];
                }
            } //while
        }
    }
}