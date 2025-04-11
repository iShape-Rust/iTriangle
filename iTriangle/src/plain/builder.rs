use crate::plain::net::TriangleNet;
use crate::plain::section::{EdgeType, TriangleEdge};
use crate::plain::triangle::PlainTriangle;
use std::collections::HashMap;

struct PhantomHandler {
    vertex: usize,
    triangle: usize,
}

pub(super) struct TriangleNetBuilder {
    pub(super) triangles: Vec<PlainTriangle>,
    edges_phantom_store: HashMap<usize, PhantomHandler>,
    edges_counter: usize,
}

impl TriangleNetBuilder {
    #[inline]
    pub(super) fn with_triangles_count(triangles_count: usize) -> Self {
        Self {
            triangles: Vec::with_capacity(triangles_count),
            edges_phantom_store: HashMap::with_capacity(16),
            edges_counter: 0,
        }
    }

    #[inline]
    pub(super) fn next_triangle_index(&self) -> usize {
        self.triangles.len()
    }

    #[inline]
    pub(super) fn next_phantom_edge_index(&self) -> usize {
        self.edges_counter
    }

    #[inline]
    pub(super) fn pop_or_insert(
        &mut self,
        side: usize,
        vertex: usize,
        triangle: usize,
    ) -> Option<usize> {
        if let Some(handler) = self.edges_phantom_store.get(&side) {
            let other_triangle = handler.triangle;
            self.triangles[other_triangle].neighbors[handler.vertex] = triangle;
            self.edges_phantom_store.remove(&side);
            Some(other_triangle)
        } else {
            self.edges_phantom_store
                .insert(side, PhantomHandler { vertex, triangle });
            None
        }
    }

    #[inline]
    pub(super) fn add(&mut self, triangle: PlainTriangle) {
        self.triangles.push(triangle);
    }

    #[inline]
    pub(super) fn build(self) -> TriangleNet {
        TriangleNet {
            triangles: self.triangles,
        }
    }

    #[inline]
    pub(super) fn add_triangle_and_join_by_edge(
        &mut self,
        edge: &TriangleEdge,
        vertex: usize,
        mut new_triangle: PlainTriangle,
    ) -> usize {
        let new_index = self.next_triangle_index();
        match edge.kind {
            EdgeType::Regular(triangle_index) => {
                if self.triangles.len() <= triangle_index {
                    self.triangles.push(new_triangle);
                    return new_index;
                }
                new_triangle.neighbors[vertex] = triangle_index;
                let other = &mut self.triangles[triangle_index];
                let vi = other.other_vertex(edge.a.index, edge.b.index);
                other.neighbors[vi] = new_index;
            }
            EdgeType::Phantom(edge_index) => {
                if let Some(handler) = self.edges_phantom_store.get(&edge_index) {
                    // if exist update neighbor
                    self.triangles[handler.triangle].neighbors[handler.vertex] = new_index;
                    new_triangle.neighbors[vertex] = handler.triangle;
                    self.edges_phantom_store.remove(&edge_index);
                } else {
                    // create a phantom edge
                    self.edges_phantom_store.insert(
                        edge_index,
                        PhantomHandler {
                            vertex,
                            triangle: new_index,
                        },
                    );
                }
            }
        }
        self.triangles.push(new_triangle);

        new_index
    }
}
