use crate::geom::point::IndexPoint;
use crate::geom::triangle::IntTriangle;
use crate::int::earcut::earcut_64::{Bit, EarcutStore};
use crate::int::triangulation::RawIntTriangulation;
use alloc::vec::Vec;
use i_overlay::i_float::int::point::IntPoint;

struct TriangleHandler {
    triangle: u8,
    vertex: u8,
}

struct EdgeItem {
    edge: Edge,
    handler: TriangleHandler
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Edge {
    a: u8,
    b: u8,
}

impl Edge {
    #[inline]
    fn new(a: usize, b: usize) -> Self {
        debug_assert!(a < b);
        Self {
            a: a as u8,
            b: b as u8,
        }
    }
}

struct EdgePool {
    edges: Vec<EdgeItem>,
}

impl EdgePool {
    #[inline]
    fn insert(&mut self, edge: Edge, handler: TriangleHandler) -> Option<TriangleHandler> {
        // if it exists remove and return value
        // if not exists save
        if let Some(index) = self.edges.iter().position(|it| it.edge == edge) {
            Some(self.edges.swap_remove(index).handler)
        } else {
            let item = EdgeItem {
                edge,
                handler,
            };
            self.edges.push(item);
            None
        }
    }
}

pub(super) struct NetEarcutStore<'a> {
    triangulation: &'a mut RawIntTriangulation,
    pool: EdgePool,
    last: usize,
}

impl<'a> NetEarcutStore<'a> {
    #[inline]
    pub(super) fn new(count: usize, triangulation: &'a mut RawIntTriangulation) -> Self {
        Self {
            last: count - 1,
            triangulation,
            pool: EdgePool {
                edges: Vec::with_capacity(8),
            },
        }
    }
}

impl EarcutStore for NetEarcutStore<'_> {
    #[inline]
    fn collect_triangles(&mut self, contour: &[IntPoint], start: usize, bits: u64, count: u32) {
        let ai = start;
        let a = IndexPoint::new(ai, contour[ai]);

        let bi = bits.next_wrapped_index(ai);
        let mut b = IndexPoint::new(bi, contour[bi]);
        let mut ci = bits.next_wrapped_index(bi);
        for _ in 0..count {

            let c = IndexPoint::new(ci, contour[ci]);

            let triangle_index = self.triangulation.triangles.len();
            let mut triangle = IntTriangle::abc(a, b, c);

            triangle.neighbors[0] = self.get_or_put(b.index, c.index, triangle_index, 0);
            triangle.neighbors[1] = self.get_or_put(a.index, c.index, triangle_index, 1);
            triangle.neighbors[2] = self.get_or_put(a.index, b.index, triangle_index, 2);

            self.triangulation.triangles.push(triangle);

            b = c;
            ci = bits.next_wrapped_index(ci);
        }
    }
}

impl NetEarcutStore<'_> {
    #[inline]
    fn get_or_put(&mut self, i0: usize, i1: usize, t: usize, v: usize) -> usize {
        // is edge inner or outer
        let (a, b) = if i0 < i1 { (i0, i1) } else { (i1, i0) };
        if b - a == 1 || a == 0 && b == self.last {
            return usize::MAX;
        }
        // a < b

        let handler = TriangleHandler {
            triangle: t as u8,
            vertex: v as u8,
        };

        // if a neighbor exist we should also update it
        if let Some(other) = self.pool.insert(Edge::new(a, b), handler) {
            let triangle = other.triangle as usize;
            self.triangulation.triangles[triangle].neighbors[other.vertex as usize] = t;
            triangle
        } else {
            usize::MAX
        }
    }
}