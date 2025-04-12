use crate::plain::section::{Content, EdgeType, Section, TriangleEdge};
use crate::plain::triangle::PlainTriangle;
use crate::plain::v_segment::VSegment;
use crate::plain::vertex::{ChainVertex, VertexType};
use i_overlay::i_float::triangle::Triangle;
use i_tree::set::sort::SetCollection;
use i_tree::set::tree::SetTree;
use std::collections::HashMap;
use std::mem::swap;
use i_overlay::i_float::int::point::IntPoint;

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
    pub(super) fn build(&mut self, vertices: &[ChainVertex]) {
        let mut sections: SetTree<VSegment, Section> = SetTree::new(8);

        for v in vertices.iter() {
            match v.vert_type() {
                VertexType::Start => sections.insert(Section::with_vertex(v)),
                VertexType::Merge => self.merge(v, &mut sections),
                VertexType::Other => {
                    let index = sections.first_index_less_by(|s| s.is_under_point_order(v.this));
                    let s = sections.value_by_index_mut(index);
                    match s.add(v, self) {
                        Action::Create(new) => sections.insert(new),
                        Action::Delete => sections.delete_by_index(index),
                        Action::Update => {}
                    }
                }
            }
        }
    }

    pub(super) fn triangle_indices(&self) -> Vec<usize> {
        let mut result = Vec::with_capacity(3 * self.triangles.len());
        for t in &self.triangles {
            let v = &t.vertices;
            result.extend_from_slice(&[v[0].index, v[1].index, v[2].index]);
        }
        result
    }
}

impl TriangleNetBuilder {

    #[inline]
    fn next_triangle_index(&self) -> usize {
        self.triangles.len()
    }

    #[inline]
    fn get_unique_phantom_edge_index(&mut self) -> usize {
        let index = self.edges_counter;
        self.edges_counter += 1;
        index
    }

    #[inline]
    fn add_triangle_and_join_by_edge(
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

    fn merge(&mut self, v: &ChainVertex, sections: &mut SetTree<VSegment, Section>) {
        let index = sections.first_index_less_by(|s| s.is_under_point_order(v.this));
        let s = sections.value_by_index(index);
        let (next_index, prev_index) = if s.prev == v.this {
            let prev_index = sections.index_after(index);
            (index, prev_index)
        } else {
            let next_index = sections.index_before(index);
            (next_index, index)
        };

        let next = sections.value_by_index_mut(next_index);
        next.add_from_start(v, self);

        let mut next_edges = if let Content::Edges(edges) = &next.content {
            edges.clone()
        } else {
            Vec::new()
        };

        let p_next = next.next;
        let sort = next.sort;

        let prev = sections.value_by_index_mut(prev_index);
        prev.add_from_end(v, self);

        match &mut prev.content {
            Content::Point(_) => {}
            Content::Edges(edges) => {
                edges.append(&mut next_edges)
            }
        }

        prev.next = p_next;
        prev.sort = sort;

        sections.delete_by_index(next_index);
    }
}

enum Action {
    Create(Section),
    Delete,
    Update,
}

impl Section {
    #[inline]
    fn with_vertex(v: &ChainVertex) -> Self {
        Self {
            prev: v.prev,
            next: v.next,
            sort: VSegment {
                a: v.this,
                b: v.next,
            },
            content: Content::Point(v.index_point()),
        }
    }

    #[inline]
    fn add(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) -> Action {
        let eq_prev = v.this == self.prev;
        let eq_next = v.this == self.next;

        match (eq_prev, eq_next) {
            (true, true) => {
                self.add_as_last(v, net_builder);
                Action::Delete
            }
            (true, false) => {
                self.add_to_top(v, net_builder);
                Action::Update
            }
            (false, true) => {
                self.add_to_bottom(v, net_builder);
                Action::Update
            }
            (false, false) => {
                let bottom_section = self.add_to_middle(v, net_builder);
                Action::Create(bottom_section)
            }
        }
    }

    #[inline]
    fn add_as_last(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        let edges = match &mut self.content {
            Content::Point(_) => {
                panic!("not implemented case")
            }
            Content::Edges(edges) => edges,
        };

        let vp = v.index_point();
        let mut prev_index = usize::MAX;
        for ei in edges.iter().take(edges.len() - 1) {
            let mut triangle = PlainTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = net_builder.next_triangle_index() + 1;
            triangle.neighbors[2] = prev_index;

            prev_index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);
        }

        let el = edges.last().unwrap();
        let mut triangle = PlainTriangle::abc(vp, el.a, el.b);
        triangle.neighbors[2] = prev_index;

        net_builder.add_triangle_and_join_by_edge(el, 0, triangle);
    }

    #[inline]
    fn add_to_top(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        self.prev = v.prev;
        self.add_from_start(v, net_builder);
    }

    #[inline]
    fn add_to_bottom(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        self.sort = VSegment {
            a: v.this,
            b: v.next,
        };
        self.next = v.next;
        self.add_from_end(v, net_builder);
    }

    #[inline]
    fn add_to_middle(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) -> Section {
        let edges = match &mut self.content {
            Content::Point(point) => {
                let phantom_index = net_builder.get_unique_phantom_edge_index();
                let vp = v.index_point();
                let top_edge = TriangleEdge {
                    a: *point,
                    b: vp,
                    kind: EdgeType::Phantom(phantom_index),
                };

                let bottom_edge = TriangleEdge {
                    a: vp,
                    b: *point,
                    kind: EdgeType::Phantom(phantom_index),
                };

                self.content = Content::Edges(vec![top_edge]);

                // bottom
                let bottom_section = Section {
                    prev: v.prev,
                    next: self.next,
                    sort: self.sort,
                    content: Content::Edges(vec![bottom_edge]),
                };

                self.next = v.next;
                self.sort = VSegment {
                    a: v.this,
                    b: v.next,
                };

                return bottom_section;
            }
            Content::Edges(edges) => edges,
        };

        let mut i = 0;
        while i < edges.len() {
            let ei = &edges[i];
            // skip first not valid triangles
            if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
                i += 1;
                continue;
            }
            break;
        }

        let vp = v.index_point();
        let e0 = &edges[i];

        let mut index =
            net_builder.add_triangle_and_join_by_edge(e0, 0, PlainTriangle::abc(vp, e0.a, e0.b));

        let top_edge = TriangleEdge {
            a: e0.a,
            b: vp,
            kind: EdgeType::Regular(index),
        };

        let mut top_edges = edges.split_off(i);
        swap(&mut top_edges, edges);
        top_edges.push(top_edge);

        let top_section = Section {
            prev: self.prev,
            next: v.next,
            sort: VSegment {
                a: v.this,
                b: v.next,
            },
            content: Content::Edges(top_edges),
        };

        let mut next_index = index + 2;
        i = 1;
        while i < edges.len() {
            let ei = &edges[i];
            if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
                break;
            }
            let mut triangle = PlainTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = next_index;
            triangle.neighbors[2] = index;
            index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);
            next_index = index + 2;

            i += 1;
        }
        net_builder.triangles[index].neighbors[1] = usize::MAX;

        let bottom_edge = TriangleEdge {
            a: vp,
            b: edges[i - 1].b,
            kind: EdgeType::Regular(index),
        };

        *edges = edges.split_off(i);
        edges.insert(0, bottom_edge);

        self.prev = v.prev;

        top_section
    }

    fn add_from_start(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        let vp = v.index_point();

        let edges = match &mut self.content {
            Content::Point(point) => {
                let edges = vec![TriangleEdge {
                    a: vp,
                    b: *point,
                    kind: EdgeType::Regular(usize::MAX),
                }];
                self.content = Content::Edges(edges);
                return;
            }
            Content::Edges(edges) => edges,
        };

        let e0 = edges.first().unwrap();

        if Triangle::is_cw_or_line_point(v.this, e0.a.point, e0.b.point) {
            edges.insert(
                0,
                TriangleEdge {
                    a: vp,
                    b: e0.a,
                    kind: EdgeType::Regular(usize::MAX),
                },
            );
            return;
        }

        let mut index =
            net_builder.add_triangle_and_join_by_edge(e0, 0, PlainTriangle::abc(vp, e0.a, e0.b));

        let mut n = 1;
        let mut eb = e0.b;
        for ei in edges.iter().skip(1) {
            if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
                break;
            }
            eb = ei.b;
            n += 1;
            let mut triangle = PlainTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = index;
            let prev_index = index;
            index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);

            net_builder.triangles[prev_index].neighbors[1] = index;
        }

        if edges.len() == n {
            edges.clear();
        } else {
            *edges = edges.split_off(n);
        }

        edges.insert(
            0,
            TriangleEdge {
                a: vp,
                b: eb,
                kind: EdgeType::Regular(index),
            },
        );
    }

    fn add_from_end(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        let vp = v.index_point();
        let edges = match &mut self.content {
            Content::Point(point) => {
                let edges = vec![TriangleEdge {
                    a: *point,
                    b: vp,
                    kind: EdgeType::Regular(usize::MAX),
                }];
                self.content = Content::Edges(edges);
                return;
            }
            Content::Edges(edges) => edges,
        };

        let el = edges.last().unwrap();

        if Triangle::is_cw_or_line_point(v.this, el.a.point, el.b.point) {
            edges.push(TriangleEdge {
                a: el.b,
                b: vp,
                kind: EdgeType::Regular(usize::MAX),
            });
            return;
        }

        let mut index =
            net_builder.add_triangle_and_join_by_edge(el, 0, PlainTriangle::abc(vp, el.a, el.b));
        let mut ea = el.a;
        let mut n = 1;
        for ei in edges.iter().rev().skip(1) {
            if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
                break;
            }
            ea = ei.a;
            n += 1;
            let mut triangle = PlainTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = index;
            let prev_index = index;
            index = net_builder.add_triangle_and_join_by_edge(el, 0, triangle);

            net_builder.triangles[prev_index].neighbors[2] = index;
        }
        edges.truncate(edges.len() - n);

        edges.push(TriangleEdge {
            a: ea,
            b: vp,
            kind: EdgeType::Regular(index),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::plain::builder::TriangleNetBuilder;
    use crate::plain::vertex::ShapeToVertices;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::shape::IntShape;

    fn shape_to_builder(shape: IntShape) -> TriangleNetBuilder {
        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2);

        let chain_vertices = shape.to_chain_vertices(&[]);
        let mut net = TriangleNetBuilder::with_triangles_count(triangles_count);
        net.build(&chain_vertices);
        net
    }

    #[test]
    fn test_0() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]];

        let net = shape_to_builder(shape);

        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[2], 1);
        assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_1() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(5, 0),
            IntPoint::new(0, 5),
            IntPoint::new(-5, 0),
        ]];

        let net = shape_to_builder(shape);

        assert_eq!(net.triangles.len(), 2);

        // assert_eq!(net.triangles[0].neighbors[2], 1);
        // assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_2() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_3() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 5),
            IntPoint::new(0, 10),
            IntPoint::new(5, 5),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[2], 0);
    }

    #[test]
    fn test_4() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[0], 1);
        assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_5() {
        let shape = vec![vec![
            IntPoint::new(-15, -15),
            IntPoint::new(15, -15),
            IntPoint::new(25, 0),
            IntPoint::new(15, 15),
            IntPoint::new(-15, 15),
            IntPoint::new(-25, 0),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 4);
        //
        // assert_eq!(net.triangles[0].neighbors, [usize::MAX, 1, 4]);
        // assert_eq!(net.triangles[1].neighbors, [0, 2, usize::MAX]);
        // assert_eq!(net.triangles[2].neighbors, [usize::MAX, usize::MAX, 1]);
        // assert_eq!(net.triangles[3].neighbors, [usize::MAX, 4, usize::MAX]);
        // assert_eq!(net.triangles[4].neighbors, [0, usize::MAX, 3]);
    }

    #[test]
    fn test_6() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(-10, -15),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
            IntPoint::new(-10, 15),
            IntPoint::new(0, 5),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 5);

        assert_eq!(net.triangles[0].neighbors, [usize::MAX, 1, 4]);
        assert_eq!(net.triangles[1].neighbors, [0, 2, usize::MAX]);
        assert_eq!(net.triangles[2].neighbors, [usize::MAX, usize::MAX, 1]);
        assert_eq!(net.triangles[3].neighbors, [usize::MAX, 4, usize::MAX]);
        assert_eq!(net.triangles[4].neighbors, [0, usize::MAX, 3]);
    }

    #[test]
    fn test_7() {
        let shape = vec![vec![
            IntPoint::new(15, -15),
            IntPoint::new(0, 15),
            IntPoint::new(0, 0),
            IntPoint::new(-15, 0),
            IntPoint::new(-15, -15),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 3);
    }

    #[test]
    fn test_8() {
        let shape = vec![vec![
            IntPoint::new(-5, -10),
            IntPoint::new(-10, -15),
            IntPoint::new(5, -20),
            IntPoint::new(0, 0),
            IntPoint::new(5, 20),
            IntPoint::new(-10, 15),
            IntPoint::new(-5, 10),
            IntPoint::new(-15, 0),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 3);
    }

    #[test]
    fn test_9() {
        let shape = vec![vec![
            IntPoint::new(-5, -10),
            IntPoint::new(-10, -15),
            IntPoint::new(-2, -20),
            IntPoint::new(5, -20),
            IntPoint::new(0, 0),
            IntPoint::new(5, 20),
            IntPoint::new(-2, 20),
            IntPoint::new(-10, 15),
            IntPoint::new(-5, 10),
            IntPoint::new(-15, 0),
        ]];

        let net = shape_to_builder(shape);
        assert_eq!(net.triangles.len(), 3);
    }
}
