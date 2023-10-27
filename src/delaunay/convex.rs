use i_float::fix_vec::FixVec;
use i_shape::fix_path::FixPath;
use crate::delaunay::delaunay::Delaunay;
use crate::delaunay::triangle::DTriangle;
use crate::index::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvexSide {
    Inner,
    Outer,
}

impl ConvexSide {
    fn new(is_inner: bool) -> Self {
        if is_inner {
            Self::Inner
        } else {
            Self::Outer
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConvexPath {
    pub path: FixPath,
    pub side: Vec<ConvexSide>,
}

impl PartialEq for ConvexPath {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.side == other.side
    }
}

#[derive(Debug, Clone, Copy)]
struct Node {
    next: usize,
    index: usize,
    prev: usize,
    next_side: ConvexSide,
    point: FixVec,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    triangle_index: usize,
    neighbor: usize,
    a: usize,
    b: usize,
}

struct ConvexPolygonBuilder {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl ConvexPolygonBuilder {
    fn new() -> Self {
        Self { nodes: Vec::with_capacity(16), edges: Vec::with_capacity(16) }
    }

    fn to_path(&self) -> ConvexPath {
        let count = self.nodes.len();
        let mut path = Vec::with_capacity(count);
        let mut side = Vec::with_capacity(count);

        let mut node = self.nodes[count - 1];
        for _ in 0..count {
            path.push(node.point);
            side.push(node.next_side);
            node = self.nodes[node.next];
        }

        ConvexPath { path, side }
    }

    fn start(&mut self, triangle: DTriangle) {
        self.nodes.clear();
        self.edges.clear();

        let bc = triangle.na();
        let ca = triangle.nb();
        let ab = triangle.nc();

        let is_ca_inner = ca.is_not_nil();
        let is_ab_inner = ab.is_not_nil();
        let is_bc_inner = bc.is_not_nil();

        self.nodes.push(Node { next: 1, index: 0, prev: 2, point: triangle.va().point, next_side: ConvexSide::new(is_ab_inner) });
        self.nodes.push(Node { next: 2, index: 1, prev: 0, point: triangle.vb().point, next_side: ConvexSide::new(is_bc_inner) });
        self.nodes.push(Node { next: 0, index: 2, prev: 1, point: triangle.vc().point, next_side: ConvexSide::new(is_ca_inner) });

        if is_ab_inner {
            self.edges.push(Edge { triangle_index: triangle.index, neighbor: ab, a: 0, b: 1 })
        }

        if is_bc_inner {
            self.edges.push(Edge { triangle_index: triangle.index, neighbor: bc, a: 1, b: 2 })
        }

        if is_ca_inner {
            self.edges.push(Edge { triangle_index: triangle.index, neighbor: ca, a: 2, b: 0 })
        }
    }

    fn add(&mut self, edge: Edge, triangle: DTriangle) -> bool {
        let v_index = triangle.opposite(edge.triangle_index);
        let v = triangle.vertices[v_index];

        // a0 -> a1 -> p

        let mut node_a1 = self.nodes[edge.a];
        let va0 = self.nodes[node_a1.prev].point;
        let va1 = node_a1.point;

        let aa = va1 - va0;
        let ap = v.point - va1;

        let apa = aa.unsafe_cross_product(ap);
        if apa > 0 {
            return false;
        }

        // b0 <- b1 <- p

        let mut node_b1 = self.nodes[edge.b];
        let vb0 = self.nodes[node_b1.next].point;
        let vb1 = node_b1.point;

        let bb = vb0 - vb1;
        let bp = vb1 - v.point;

        let bpb = bp.unsafe_cross_product(bb);
        if bpb > 0 {
            return false;
        }

        let prev_neighbor = triangle.neighbors[(v_index + 2) % 3];
        let next_neighbor = triangle.neighbors[(v_index + 1) % 3];

        let new_index = self.nodes.len();
        let new_side = ConvexSide::new(prev_neighbor.is_not_nil());

        let new_node = Node { next: node_b1.index, index: new_index, prev: node_a1.index, point: v.point, next_side: new_side };

        node_a1.next = new_index;
        node_a1.next_side = ConvexSide::new(next_neighbor.is_not_nil());
        node_b1.prev = new_index;

        self.nodes.push(new_node);
        self.nodes[node_a1.index] = node_a1;
        self.nodes[node_b1.index] = node_b1;

        if next_neighbor.is_not_nil() {
            let edge = Edge { triangle_index: triangle.index, neighbor: next_neighbor, a: edge.a, b: new_index };
            self.edges.push(edge);
        }

        if prev_neighbor.is_not_nil() {
            let edge = Edge { triangle_index: triangle.index, neighbor: prev_neighbor, a: new_index, b: edge.b };
            self.edges.push(edge)
        }

        return true;
    }
}

impl Delaunay {
    pub fn to_convex_polygons(&self) -> Vec<ConvexPath> {
        let mut result = Vec::new();
        let n = self.triangles.len();

        let mut visited = vec![false; n];

        let mut builder = ConvexPolygonBuilder::new();

        for i in 0..n {
            if visited[i] {
                continue;
            }

            let first = self.triangles[i];
            builder.start(first);
            visited[i] = true;

            while !builder.edges.is_empty() {
                let edge = builder.edges.pop().unwrap(); // edges is not empty
                if visited[edge.neighbor] {
                    continue;
                }
                let triangle = self.triangles[edge.neighbor];
                if builder.add(edge, triangle) {
                    visited[edge.neighbor] = true;
                }
            }

            result.push(builder.to_path())
        }

        result
    }
}