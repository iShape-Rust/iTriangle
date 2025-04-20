use crate::geom::triangle::ABCTriangle;
use crate::raw::section::{Content, EdgeType, Section, TriangleEdge};
use crate::raw::v_segment::VSegment;
use crate::raw::vertex::{ChainVertex, VertexType};
use i_overlay::i_float::triangle::Triangle;
use i_tree::set::sort::SetCollection;
use i_tree::set::tree::SetTree;
use std::cmp::Ordering;
use std::mem::swap;

#[derive(Copy, Clone)]
struct PhantomHandler {
    vertex: usize,
    triangle: usize,
}

struct PhantomStore {
    buffer: Vec<PhantomHandler>,
    unused: Vec<usize>,
}

impl PhantomStore {
    const EMPTY: PhantomHandler = PhantomHandler {
        vertex: usize::MAX,
        triangle: usize::MAX,
    };

    fn new(capacity: usize) -> Self {
        let capacity = capacity.max(8);
        let mut store = Self {
            buffer: Vec::with_capacity(capacity),
            unused: Vec::with_capacity(capacity),
        };
        store.reserve(capacity);
        store
    }

    #[inline]
    fn reserve(&mut self, length: usize) {
        debug_assert!(length > 0);
        let n = self.buffer.len();
        let l = length;
        self.buffer.reserve(length);
        self.buffer.resize(self.buffer.len() + length, Self::EMPTY);
        self.unused.reserve(length);
        self.unused.extend((n..n + l).rev());
    }

    #[inline]
    fn get(&self, index: usize) -> Option<PhantomHandler> {
        let item = self.buffer[index];
        if item.triangle == usize::MAX {
            None
        } else {
            Some(item)
        }
    }

    #[inline]
    fn set(&mut self, index: usize, handler: PhantomHandler) {
        debug_assert!(self.buffer[index].triangle == usize::MAX);
        self.buffer[index] = handler;
    }

    #[inline]
    fn get_free_index(&mut self) -> usize {
        if self.unused.is_empty() {
            self.reserve(self.unused.capacity());
        }
        self.unused.pop().unwrap()
    }

    #[inline]
    fn put_back(&mut self, index: usize) {
        self.buffer[index] = Self::EMPTY;
        self.unused.push(index)
    }
}

pub(super) struct TriangleNetBuilder {
    pub(super) triangles: Vec<ABCTriangle>,
    phantom_store: PhantomStore,
}

impl TriangleNetBuilder {
    #[inline]
    pub(super) fn with_triangles_count(triangles_count: usize) -> Self {
        Self {
            triangles: Vec::with_capacity(triangles_count),
            phantom_store: PhantomStore::new(16),
        }
    }

    #[inline]
    pub(super) fn build(&mut self, vertices: &[ChainVertex]) {
        let mut tree: SetTree<VSegment, Section> = SetTree::new(8);

        for v in vertices.iter() {
            match v.get_type() {
                VertexType::Start => self.start(v, &mut tree),
                VertexType::End => self.end(v, &mut tree),
                VertexType::Merge => self.merge(v, &mut tree),
                VertexType::Split => self.split(v, &mut tree),
                VertexType::Join => self.join(v, &mut tree),
                VertexType::Implant => self.implant(v, &mut tree),
            }
        }
    }
}

impl TriangleNetBuilder {
    #[inline]
    fn next_triangle_index(&self) -> usize {
        self.triangles.len()
    }

    #[inline]
    fn get_unique_phantom_edge_index(&mut self) -> usize {
        self.phantom_store.get_free_index()
    }

    #[inline]
    fn add_triangle_and_join_by_edge(
        &mut self,
        edge: &TriangleEdge,
        vertex: usize,
        mut new_triangle: ABCTriangle,
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
                if let Some(handler) = self.phantom_store.get(edge_index) {
                    // if exist update neighbor
                    self.triangles[handler.triangle].neighbors[handler.vertex] = new_index;
                    new_triangle.neighbors[vertex] = handler.triangle;
                    self.phantom_store.put_back(edge_index);
                } else {
                    // create a phantom edge
                    self.phantom_store.set(
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

    fn join(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        if section.sort.b == v.this {
            section.add_to_bottom(v, self);
        } else {
            section.add_to_top(v, self);
        }
    }

    fn start(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        let section = Section {
            sort: VSegment {
                a: v.this,
                b: v.next,
            },
            content: Content::Point(v.index_point()),
        };
        tree.insert(section);
    }

    fn end(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        section.add_as_last(v, self);
        tree.delete_by_index(index);
    }

    fn split(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        let new_section = section.add_to_middle(v, self);
        tree.insert(new_section);
    }

    fn merge(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        let prev_index = tree.find_section(v);
        let next_index = tree.index_before(prev_index);
        let next = tree.value_by_index_mut(next_index);
        next.add_from_start(v, self);

        let mut next_edges = if let Content::Edges(edges) = &next.content {
            edges.clone()
        } else {
            Vec::new()
        };

        let sort = next.sort;

        let prev = tree.value_by_index_mut(prev_index);
        prev.add_from_end(v, self);

        match &mut prev.content {
            Content::Point(_) => {}
            Content::Edges(edges) => edges.append(&mut next_edges),
        }

        prev.sort = sort;

        tree.delete_by_index(next_index);
    }

    fn implant(&mut self, v: &ChainVertex, tree: &mut SetTree<VSegment, Section>) {
        self.split(v, tree)
    }
}

impl Section {
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
            let mut triangle = ABCTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = net_builder.next_triangle_index() + 1;
            triangle.neighbors[2] = prev_index;

            prev_index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);
        }

        let el = edges.last().unwrap();
        let mut triangle = ABCTriangle::abc(vp, el.a, el.b);
        triangle.neighbors[2] = prev_index;

        net_builder.add_triangle_and_join_by_edge(el, 0, triangle);
    }

    #[inline]
    fn add_to_top(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        self.add_from_start(v, net_builder);
    }

    #[inline]
    fn add_to_bottom(&mut self, v: &ChainVertex, net_builder: &mut TriangleNetBuilder) {
        self.sort = VSegment {
            a: v.this,
            b: v.next,
        };
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
                    sort: self.sort,
                    content: Content::Edges(vec![bottom_edge]),
                };

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
        if i >= edges.len() {
            let ax = vp.point.x - edges[0].a.point.x;
            let bx = vp.point.x - edges[edges.len() - 1].b.point.x;

            let phantom_index = net_builder.get_unique_phantom_edge_index();
            return if ax < bx {
                let ea = edges[0].a;
                let top_edges = vec![TriangleEdge {
                    a: ea,
                    b: vp,
                    kind: EdgeType::Phantom(phantom_index),
                }];

                let bottom_edge = TriangleEdge {
                    a: vp,
                    b: ea,
                    kind: EdgeType::Phantom(phantom_index),
                };

                edges.insert(0, bottom_edge);

                // top section
                Section {
                    sort: VSegment {
                        a: v.this,
                        b: v.next,
                    },
                    content: Content::Edges(top_edges),
                }
            } else {
                let eb = edges[i - 1].b;
                let top_edge = TriangleEdge {
                    a: eb,
                    b: vp,
                    kind: EdgeType::Phantom(phantom_index),
                };
                edges.push(top_edge);

                let bottom_edges = vec![TriangleEdge {
                    a: vp,
                    b: eb,
                    kind: EdgeType::Phantom(phantom_index),
                }];

                let bottom_section = Section {
                    sort: self.sort,
                    content: Content::Edges(bottom_edges),
                };

                self.sort = VSegment {
                    a: v.this,
                    b: v.next,
                };

                bottom_section
            };
        }
        let e0 = &edges[i];

        let mut t0 = ABCTriangle::abc(vp, e0.a, e0.b);
        t0.neighbors[1] = net_builder.triangles.len() + 1;
        let mut index = net_builder.add_triangle_and_join_by_edge(e0, 0, t0);

        let top_edge = TriangleEdge {
            a: e0.a,
            b: vp,
            kind: EdgeType::Regular(index),
        };

        let mut top_edges = edges.split_off(i);
        swap(&mut top_edges, edges);
        top_edges.push(top_edge);

        let top_section = Section {
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
            let mut triangle = ABCTriangle::abc(vp, ei.a, ei.b);
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
            net_builder.add_triangle_and_join_by_edge(e0, 0, ABCTriangle::abc(vp, e0.a, e0.b));

        let mut n = 1;
        let mut eb = e0.b;
        for ei in edges.iter().skip(1) {
            if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
                break;
            }
            eb = ei.b;
            n += 1;
            let mut triangle = ABCTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[2] = index;
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
            net_builder.add_triangle_and_join_by_edge(el, 0, ABCTriangle::abc(vp, el.a, el.b));
        let mut ea = el.a;
        let mut n = 1;
        for ei in edges.iter().rev().skip(1) {
            if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
                break;
            }
            ea = ei.a;
            n += 1;
            let mut triangle = ABCTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = index;
            let prev_index = index;
            index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);

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
impl TriangleNetBuilder {
    pub fn validate(&self) {
        for (i, t) in self.triangles.iter().enumerate() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;
            let area = Triangle::area_two_point(a, b, c);
            assert!(area <= 0);

            let n0 = t.neighbors[0];
            let n1 = t.neighbors[1];
            let n2 = t.neighbors[2];

            if n0 != usize::MAX {
                assert!(self.triangles[n0].neighbors.contains(&i));
            }
            if n1 != usize::MAX {
                assert!(self.triangles[n1].neighbors.contains(&i));
            }
            if n2 != usize::MAX {
                assert!(self.triangles[n2].neighbors.contains(&i));
            }
        }
    }

    pub fn area(&self) -> i64 {
        let mut s = 0;
        for t in self.triangles.iter() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;

            s += Triangle::area_two_point(a, b, c);
        }

        s
    }
}

trait FindSection {
    fn find_section(&self, v: &ChainVertex) -> u32;
}

impl FindSection for SetTree<VSegment, Section> {
    #[inline]
    fn find_section(&self, v: &ChainVertex) -> u32 {
        self.first_index_less_by(|s| {
            let point_search = s.is_under_point_order(v.this);
            match point_search {
                Ordering::Equal => {
                    if v.prev == s.a {
                        Ordering::Equal
                    } else {
                        Triangle::clock_order_point(s.a, v.next, s.b)
                    }
                }
                _ => point_search,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::builder::TriangleNetBuilder;
    use crate::raw::vertex::ToChainVertices;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::ContourDirection;
    use i_overlay::core::simplify::Simplify;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;
    use i_overlay::i_shape::int::shape::IntShape;
    use rand::Rng;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    fn shape_to_builder(shape: &IntShape) -> TriangleNetBuilder {
        let triangles_count = shape.iter().fold(0, |s, path| s + path.len() - 2);

        let mut net = TriangleNetBuilder::with_triangles_count(triangles_count);
        net.build(&shape.to_chain_vertices());
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);

        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_1() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(5, 0),
            IntPoint::new(0, 5),
            IntPoint::new(-5, 0),
        ]];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);

        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_2() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
        ]];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_3() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 5),
            IntPoint::new(0, 10),
            IntPoint::new(5, 5),
        ]];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_4() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
        ]];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 4);
        net.validate();

        assert_eq!(net.area(), shape_area);
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 5);
        net.validate();

        assert_eq!(net.area(), shape_area);
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 3);
        net.validate();

        assert_eq!(net.area(), shape_area);
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 6);
        net.validate();

        assert_eq!(net.area(), shape_area);
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
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 8);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_10() {
        let shape = vec![
            path(&[[-15, -15], [15, -15], [15, 15], [-15, 15]]),
            path(&[[-10, -5], [-10, 5], [0, 0]]),
            path(&[[5, -10], [-5, -10], [0, 0]]),
            path(&[[10, 5], [10, -5], [0, 0]]),
            path(&[[-5, 10], [5, 10], [0, 0]]),
        ];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 16);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_11() {
        let shape = vec![
            path(&[[-5, -5], [20, -5], [20, 20], [-5, 20]]),
            path(&[[0, 0], [0, 5], [5, 5], [5, 0]]),
            path(&[[0, 10], [0, 15], [5, 15], [5, 10]]),
            path(&[[10, 0], [10, 5], [15, 5], [15, 0]]),
            path(&[[10, 10], [10, 15], [15, 15], [15, 10]]),
            path(&[[5, 5], [5, 10], [10, 10], [10, 5]]),
        ];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 24);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_12() {
        let shape = vec![
            path(&[
                [-30, -30],
                [0, -15],
                [30, -30],
                [15, 0],
                [30, 30],
                [0, 15],
                [-30, 30],
                [-15, 0],
            ]),
            path(&[
                [-20, 20],
                [0, 10],
                [20, 20],
                [10, 0],
                [20, -20],
                [0, -10],
                [-20, -20],
                [-10, 0],
            ]),
        ];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 16);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_13() {
        let shape = vec![path(&[
            [-15, 15],
            [10, 15],
            [18, -15],
            [15, -15],
            [30, -30],
            [15, 0],
            [30, 30],
            [-15, 30],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 6);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_14() {
        let shape = vec![path(&[[-2, -3], [-4, -4], [5, -1], [1, -1], [2, 3]])];
        let s = &shape.simplify(
            FillRule::NonZero,
            ContourDirection::CounterClockwise,
            false,
            0,
        )[0];

        let shape_area = s.area_two();

        let net = shape_to_builder(&s);
        assert_eq!(net.triangles.len(), 3);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_15() {
        let shape = vec![path(&[[0, 2], [2, 0], [5, 0], [4, 6]])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 2);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_16() {
        let shape = vec![path(&[[0, 4], [-4, -3], [-2, -2], [1, -2], [0, -1]])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 3);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_17() {
        let shape = vec![path(&[
            [-1, -2],
            [-2, -2],
            [1, -4],
            [1, -1],
            [3, -1],
            [1, -2],
            [5, -2],
            [0, 5],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 6);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_18() {
        let shape = vec![path(&[
            [3, 3],
            [-4, 3],
            [1, -2],
            [-2, 2],
            [0, 1],
            [1, -2],
            [1, -4],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 5);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_19() {
        let shape = vec![path(&[
            [-2, 0],
            [-3, 2],
            [0, -10],
            [2, 1],
            [-1, 2],
            [-1, 5],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 4);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_20() {
        let shape = vec![path(&[
            [5, 5],
            [-5, 1],
            [2, 0],
            [-2, 2],
            [1, 3],
            [2, 0],
            [2, -5],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 5);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_21() {
        let shape = vec![path(&[
            [-2, 0],
            [-5, 1],
            [5, -5],
            [3, -1],
            [-1, 0],
            [2, 0],
            [3, -1],
            [4, 4],
        ])];
        let shape_area = shape.area_two();

        let net = shape_to_builder(&shape);
        assert_eq!(net.triangles.len(), 6);
        net.validate();

        assert_eq!(net.area(), shape_area);
    }

    #[test]
    fn test_random_0() {
        for _ in 0..100_000 {
            let path = random(8, 5);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_1() {
        for _ in 0..100_000 {
            let path = random(10, 6);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_2() {
        for _ in 0..100_000 {
            let path = random(10, 12);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_3() {
        for _ in 0..50_000 {
            let path = random(20, 20);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_4() {
        for _ in 0..10_000 {
            let path = random(30, 50);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_5() {
        for _ in 0..2_000 {
            let main = random(50, 20);
            let mut shape = vec![main];
            for _ in 0..10 {
                shape.push(random(30, 5));
            }

            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    ContourDirection::CounterClockwise,
                    false,
                    0,
                )
                .first()
            {
                let shape_area = first.area_two();

                let net = shape_to_builder(first);
                net.validate();
                assert_eq!(net.area(), shape_area);
            };
        }
    }

    fn random(radius: i32, n: usize) -> IntPath {
        let a = radius / 2;
        let mut points = Vec::with_capacity(n);
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a);
            let y = rng.random_range(-a..=a);
            points.push(IntPoint { x, y })
        }

        points
    }
}
