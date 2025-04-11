use crate::plain::builder::TriangleNetBuilder;
use crate::plain::net::TriangleNet;
use crate::plain::section::{Content, EdgeType, Section, TriangleEdge, VSegment};
use crate::plain::triangle::PlainTriangle;
use crate::plain::vertex::{PathVertex, ShapeToVertices, VertexType};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_shape::int::shape::IntShape;
use i_tree::set::sort::SetCollection;
use i_tree::set::tree::SetTree;

pub struct Triangulator {
    validate_rule: Option<FillRule>,
    min_area: usize,
}

impl Default for Triangulator {
    fn default() -> Self {
        Self {
            validate_rule: Some(FillRule::NonZero),
            min_area: 0,
        }
    }
}

impl Triangulator {
    pub fn triangulate_with_inner_points(
        &self,
        shape: &IntShape,
        inner_points: &[IntPoint],
    ) -> TriangleNet {
        let triangles_count =
            shape.iter().fold(0, |s, path| s + path.len() - 2) + 2 * inner_points.len();

        let mut net_builder = TriangleNetBuilder::with_triangles_count(triangles_count);

        let vertices = shape.to_vertices(inner_points);

        let mut sections: SetTree<VSegment, Section> = SetTree::new(8);

        for v in vertices.iter() {
            match v.vert_type() {
                VertexType::Start => sections.insert(Section::with_vertex(v)),
                VertexType::Merge => {
                    let index = sections.first_index_less_by(|s| s.is_under_point_order(v.this));
                    let s = sections.value_by_index(index);
                    let (next_index, prev_index) = if s.prev == v.this {
                        let prev_index = sections.index_after(index);
                        (index, prev_index)
                    } else {
                        let next_index = sections.index_before(index);
                        (next_index, index)
                    };

                    let next_s = sections.value_by_index(next_index) as *const Section;
                    let prev_s = sections.value_by_index_mut(prev_index);

                    prev_s.merge(v, next_s);
                    sections.delete_by_index(next_index);
                }
                VertexType::Other => {
                    let index = sections.first_index_less_by(|s| s.is_under_point_order(v.this));
                    let s = sections.value_by_index_mut(index);
                    match s.add(v, &mut net_builder) {
                        AddResult::Split(new) => sections.insert(new),
                        AddResult::Delete => sections.delete_by_index(index),
                        AddResult::Update => {}
                    }
                }
            }
        }

        net_builder.build()
    }

    #[inline]
    pub fn triangulate(&self, shape: &IntShape) -> TriangleNet {
        self.triangulate_with_inner_points(shape, &[])
    }
}

enum AddResult {
    Split(Section),
    Delete,
    Update,
}

impl Section {
    #[inline]
    fn with_vertex(v: &PathVertex) -> Self {
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
    fn add(&mut self, v: &PathVertex, net_builder: &mut TriangleNetBuilder) -> AddResult {
        let eq_prev = v.this == self.prev;
        let eq_next = v.this == self.next;

        match (eq_prev, eq_next) {
            (true, true) => {
                self.add_as_last(v, net_builder);
                AddResult::Delete
            }
            (true, false) => {
                self.add_to_next(v, net_builder);
                AddResult::Update
            }
            (false, true) => {
                self.add_to_bottom(v, net_builder);
                AddResult::Update
            }
            (false, false) => {
                let bottom_section = self.add_to_middle(v, net_builder);
                AddResult::Split(bottom_section)
            }
        }
    }

    #[inline]
    fn add_as_last(&mut self, v: &PathVertex, net_builder: &mut TriangleNetBuilder) {
        let edges = match &mut self.content {
            Content::Point(point) => {
                panic!("not implemented case")
            }
            Content::Edges(edges) => edges,
        };

        let vp = v.index_point();
        let mut prev_index = usize::MAX;
        for ei in edges.iter().take(edges.len() - 1) {
            let mut triangle = PlainTriangle::abc(vp, ei.a, ei.b);
            triangle.neighbors[1] = prev_index;
            triangle.neighbors[2] = net_builder.next_triangle_index() + 1;

            prev_index = net_builder.add_triangle_and_join_by_edge(ei, 0, triangle);
        }

        let el = edges.last().unwrap();
        let mut triangle = PlainTriangle::abc(vp, el.a, el.b);
        triangle.neighbors[1] = prev_index;

        net_builder.add_triangle_and_join_by_edge(el, 0, triangle);
    }

    #[inline]
    fn add_to_next(&mut self, v: &PathVertex, net_builder: &mut TriangleNetBuilder) {
        let edges = match &mut self.content {
            Content::Point(point) => {
                let edges = vec![TriangleEdge {
                    a: v.index_point(),
                    b: *point,
                    kind: EdgeType::Regular(usize::MAX),
                }];
                self.prev = v.prev;
                self.content = Content::Edges(edges);
                return;
            }
            Content::Edges(edges) => edges,
        };

        let e0 = edges.first().unwrap();

        let vp = v.index_point();
        if Triangle::is_clockwise_point(vp.point, e0.a.point, e0.b.point) {
            edges.push(TriangleEdge {
                a: e0.b,
                b: vp,
                kind: EdgeType::Regular(usize::MAX),
            });
            return;
        }

        let mut index = net_builder
            .add_triangle_and_join_by_edge(e0, 0, PlainTriangle::abc(vp, e0.a, e0.b));

        let mut n = 1;
        let mut eb = e0.b;
        for ei in edges.iter().skip(1) {
            if Triangle::is_clockwise_point(vp.point, ei.a.point, ei.b.point) {
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
                a: v.index_point(),
                b: eb,
                kind: EdgeType::Regular(index),
            },
        );


        self.prev = v.prev;
    }

    #[inline]
    fn add_to_bottom(&mut self, v: &PathVertex, net_builder: &mut TriangleNetBuilder) {
        let edges = match &mut self.content {
            Content::Point(point) => {
                let edges = vec![TriangleEdge {
                    a: *point,
                    b: v.index_point(),
                    kind: EdgeType::Regular(usize::MAX),
                }];
                self.content = Content::Edges(edges);
                self.sort = VSegment {
                    a: v.this,
                    b: v.prev,
                };
                self.next = v.next;
                return;
            }
            Content::Edges(edges) => edges,
        };

        let vp = v.index_point();
        let el = edges.last().unwrap();

        if Triangle::is_clockwise_point(vp.point, el.a.point, el.b.point) {
            edges.push(TriangleEdge {
                a: el.b,
                b: vp,
                kind: EdgeType::Regular(usize::MAX),
            });
            return;
        }

        let mut index = net_builder
            .add_triangle_and_join_by_edge(el, 0, PlainTriangle::abc(vp, el.a, el.b));
        let mut ea = el.a;
        let mut n = 1;
        for ei in edges.iter().rev().skip(1) {
            if Triangle::is_clockwise_point(vp.point, ei.a.point, ei.b.point) {
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
            b: v.index_point(),
            kind: EdgeType::Regular(index),
        });
        self.next = v.next;
    }

    #[inline]
    fn add_to_middle(&mut self, v: &PathVertex, net_builder: &mut TriangleNetBuilder) -> Section {
        /*
        if self.is_empty_edges() {
            self.edges[0].b = v.index_point();
            self.sort = VSegment {
                a: v.this,
                b: v.next,
            };
            return self.clone();
        }
*/
        self.clone()
    }

    #[inline]
    fn merge(&mut self, v: &PathVertex, next_ref: *const Section) {
        /*
        // self is prev
        let next = unsafe { &*next_ref };
        let p = v.index_point();

        let ea = self.edges.last().unwrap().b;
        if self.is_empty_edges() {
            self.edges.clear();
        }

        self.edges.push(TriangleEdge {
            a: ea,
            b: p,
            triangle: usize::MAX,
        });

        let eb = next.edges.first().unwrap().a;

        self.edges.push(TriangleEdge {
            a: p,
            b: eb,
            triangle: usize::MAX,
        });

        if !self.is_empty_edges() {
            self.edges.extend(next.edges.iter());
        }

        self.next = next.next;
        self.sort = next.sort;

         */
    }
}

#[cfg(test)]
mod tests {
    use crate::plain::triangulator::Triangulator;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn test_0() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]];

        let net = Triangulator::default().triangulate(&shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[2], 1);
        assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_1() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
        ]];

        let net = Triangulator::default().triangulate(&shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[0], 0);
    }

    #[test]
    fn test_2() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 5),
            IntPoint::new(0, 10),
            IntPoint::new(5, 5),
        ]];

        let net = Triangulator::default().triangulate(&shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[2], 0);
    }

    #[test]
    fn test_3() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(20, -5),
            IntPoint::new(15, 0),
            IntPoint::new(20, 5),
        ]];

        let net = Triangulator::default().triangulate(&shape);
        assert_eq!(net.triangles.len(), 2);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[2], 0);
    }

    #[test]
    fn test_4() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(-10, -15),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
            IntPoint::new(-10, 15),
            IntPoint::new(0, 5),
        ]];

        let net = Triangulator::default().triangulate(&shape);
        assert_eq!(net.triangles.len(), 5);

        assert_eq!(net.triangles[0].neighbors[1], 1);
        assert_eq!(net.triangles[1].neighbors[2], 0);
    }
}
