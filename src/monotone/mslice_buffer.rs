use crate::delaunay::triangle::DTriangle;
use crate::index::{Index, NIL_INDEX};

#[derive(Debug, Clone, Copy)]
pub(crate) struct MSlice {
    a: usize,
    b: usize
}

impl MSlice {
    pub fn new(a: usize, b: usize) -> Self {
        Self { a, b }
    }
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    pub id: usize,
    pub edge: usize,
    pub triangle: usize
}

impl Edge {
    pub const EMPTY: Edge = Edge {
        id: 0,
        edge: 0,
        triangle: 0,
    };
}

pub(crate) struct MSliceBuffer {
    vertex_count: usize,
    edges: Vec<Edge>,
    vertex_marks: Vec<bool>,
}

impl MSliceBuffer {
    pub(crate) fn new(vertex_count: usize, slices: &Vec<MSlice>) -> Self {
        let mut vertex_marks = vec![false; vertex_count];
        let mut edges = vec![Edge::EMPTY; slices.len()];

        for (i, slice) in slices.iter().enumerate() {
            vertex_marks[slice.a] = true;
            vertex_marks[slice.b] = true;
            let id = Self::id(vertex_count, slice.a, slice.b);
            edges[i] = Edge { id, edge: NIL_INDEX, triangle: NIL_INDEX };
        }

        edges.sort_by(|a, b| a.id.cmp(&b.id));

        Self {
            vertex_count,
            edges,
            vertex_marks
        }
    }

    pub fn add_connections(&mut self, triangles: &mut Vec<DTriangle>) {
        let n = triangles.len();

        for i in 0..n {
            let mut triangle = triangles[i];
            let mut j0 = 1;
            let mut j1 = 2;
            for j2 in 0..3 {
                let a = triangle.vertices[j1].index;
                let b = triangle.vertices[j2].index;

                let edge_index = self.find(a, b);
                if edge_index.is_not_nil() {
                     let mut edge = self.edges[edge_index];

                    if edge.triangle.is_nil() {
                        edge.triangle = i;
                        edge.edge = j0;
                        self.edges[edge_index] = edge;
                    } else {
                        triangle.neighbors[j0] = edge.triangle;
                        let mut neighbor = triangles[edge.triangle];
                        neighbor.neighbors[edge.edge] = i;
                        triangles[edge.triangle] = neighbor;
                        triangles[i] = triangle;
                    }
                }
                j0 = j1;
                j1 = j2;
            }
        }
    }

    fn find(&self, a: usize, b: usize) -> usize {
        if !(self.vertex_marks[a] && self.vertex_marks[b]) {
            return NIL_INDEX;
        }

        let id = Self::id(self.vertex_count, a, b);

        if let Ok(pos) = self.edges.binary_search_by(|edge| edge.id.cmp(&id)) {
            return pos;
        }

        NIL_INDEX
    }

    fn id(n: usize, a: usize, b: usize) -> usize {
        if a < b {
            a * n + b
        } else {
            b * n + a
        }
    }

}