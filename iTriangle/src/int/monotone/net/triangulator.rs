use crate::geom::point::IndexPoint;
use crate::geom::triangle::IntTriangle;
use crate::int::monotone::chain::vertex::{ChainVertex, VertexType};
use crate::int::monotone::net::phantom::{PhantomEdgePool, PhantomHandler};
use crate::int::monotone::net::section::{Content, EdgeType, Section, TriangleEdge};
use crate::int::monotone::v_segment::VSegment;
use crate::int::triangulation::RawIntTriangulation;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem::swap;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::util::reserve::Reserve;
use i_tree::set::list::SetList;
use i_tree::set::sort::SetCollection;
use i_tree::set::tree::SetTree;
use crate::int::monotone::chain::builder::ChainVertexExport;

struct NetBuilder<'a> {
    triangulation: &'a mut RawIntTriangulation,
    phantom_store: PhantomEdgePool,
}

pub(crate) trait NetTriangulation {
    fn net_triangulate_into(&self, triangles_count: usize, triangulation: &mut RawIntTriangulation);
}

impl NetTriangulation for [ChainVertex] {
    fn net_triangulate_into(
        &self,
        triangles_count: usize,
        triangulation: &mut RawIntTriangulation,
    ) {
        triangulation.triangles.reserve_capacity(triangles_count);
        triangulation.triangles.clear();
        let mut builder = NetBuilder::new(triangulation);

        let n = self.len();
        let capacity = if n < 128 { 4 } else { n.ilog2() as usize };
        if capacity <= 12 {
            builder.triangulate(self, SetList::new(capacity));
        } else {
            builder.triangulate(self, SetTree::new(capacity));
        }

        self.feed_points(&mut triangulation.points);
    }
}

impl<'a> NetBuilder<'a> {
    fn new(triangulation: &'a mut RawIntTriangulation) -> Self {
        Self {
            triangulation,
            phantom_store: PhantomEdgePool::new(),
        }
    }
}

impl NetBuilder<'_> {
    #[inline]
    fn triangulate<S: SetCollection<VSegment, Section>>(
        &mut self,
        vertices: &[ChainVertex],
        mut store: S,
    ) {
        for v in vertices.iter() {
            match v.get_type() {
                VertexType::Start => self.start(v, &mut store),
                VertexType::End => self.end(v, &mut store),
                VertexType::Merge => self.merge(v, &mut store),
                VertexType::Split => self.split(v, &mut store),
                VertexType::Join => self.join(v, &mut store),
                VertexType::Steiner => self.steiner(v, &mut store),
            }
        }
    }

    #[inline]
    fn next_triangle_index(&self) -> usize {
        self.triangulation.triangles.len()
    }

    #[inline]
    fn get_unique_phantom_edge_index(&mut self) -> usize {
        self.phantom_store.alloc_phantom_index()
    }

    #[inline]
    fn insert_triangle_with_neighbor_link(
        &mut self,
        edge: &TriangleEdge,
        vertex: usize,
        mut new_triangle: IntTriangle,
    ) -> usize {
        let new_index = self.next_triangle_index();
        match edge.kind {
            EdgeType::Regular(triangle_index) => {
                if self.triangulation.triangles.len() <= triangle_index {
                    self.triangulation.triangles.push(new_triangle);
                    return new_index;
                }
                new_triangle.set_neighbor(vertex, triangle_index);
                let other = &mut self.triangulation.triangles[triangle_index];
                let vi = other.other_vertex(edge.a.index, edge.b.index);
                other.set_neighbor(vi, new_index);
            }
            EdgeType::Phantom(edge_index) => {
                if let Some(handler) = self.phantom_store.get(edge_index) {
                    // if exist update neighbor
                    self.triangulation.triangles[handler.triangle]
                        .set_neighbor(handler.vertex, new_index);
                    new_triangle.set_neighbor(vertex, handler.triangle);
                    self.phantom_store.free_phantom_index(edge_index);
                } else {
                    // create a phantom edge
                    self.phantom_store.register_phantom_link(
                        edge_index,
                        PhantomHandler {
                            vertex,
                            triangle: new_index,
                        },
                    );
                }
            }
        }
        self.triangulation.triangles.push(new_triangle);

        new_index
    }

    #[inline]
    fn join<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        if section.sort.b == v.this {
            section.add_to_bottom(v, self);
        } else {
            section.add_to_top(v, self);
        }
    }

    #[inline]
    fn start<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let section = Section {
            sort: VSegment {
                a: v.this,
                b: v.next,
            },
            content: Content::Point(v.index_point()),
        };
        tree.insert(section);
    }

    #[inline]
    fn end<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        section.add_as_last(v, self);
        tree.delete_by_index(index);
    }

    fn split<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        let new_section = section.add_to_middle(v, self);
        tree.insert(new_section);
    }

    fn merge<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
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

    fn steiner<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        section.add_steiner(v.index_point(), self);
    }
}

impl Section {
    #[inline]
    fn add_as_last(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) {
        let edges = match &mut self.content {
            Content::Edges(edges) => edges,
            Content::Point(_) => unreachable!("Section with less then 3 points not possible"),
        };

        let vp = v.index_point();
        let mut prev_index = usize::MAX;

        // Iterate all but last edge
        for ei in edges.iter().take(edges.len().saturating_sub(1)) {
            let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
            triangle.set_neighbor(1, net_builder.next_triangle_index() + 1);
            triangle.set_neighbor(2, prev_index);

            prev_index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
        }

        // Final triangle links only to previous
        if let Some(last_edge) = edges.last() {
            let mut triangle = IntTriangle::abc(vp, last_edge.a, last_edge.b);
            triangle.set_neighbor(2, prev_index);
            net_builder.insert_triangle_with_neighbor_link(last_edge, 0, triangle);
        }
    }

    #[inline]
    fn add_to_top(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) {
        self.add_from_start(v, net_builder);
    }

    #[inline]
    fn add_to_bottom(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) {
        self.sort = VSegment {
            a: v.this,
            b: v.next,
        };
        self.add_from_end(v, net_builder);
    }

    #[inline]
    fn add_to_middle(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) -> Section {
        let edges = match &mut self.content {
            Content::Point(point) => {
                let phantom_index = net_builder.get_unique_phantom_edge_index();
                let vp = v.index_point();
                let top_edge = TriangleEdge::phantom(*point, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, *point, phantom_index);

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
            let last = edges[edges.len() - 1].b;
            let mut index = edges.len();
            let mut min_dist = vp.point.x - last.point.x;
            for (ei, e) in edges.iter().enumerate() {
                let dist = vp.point.x - e.a.point.x;
                if dist < min_dist {
                    min_dist = dist;
                    index = ei;
                }
            }

            let phantom_index = net_builder.get_unique_phantom_edge_index();

            return if index == edges.len() {
                let eb = edges[i - 1].b;
                let top_edge = TriangleEdge::phantom(eb, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, eb, phantom_index);
                edges.push(top_edge);

                let bottom_section = Section {
                    sort: self.sort,
                    content: Content::Edges(vec![bottom_edge]),
                };

                self.sort = VSegment {
                    a: v.this,
                    b: v.next,
                };

                bottom_section
            } else {
                let ea = edges[index].a;
                let mut bottom_edges = edges.split_off(index);

                let top_edge = TriangleEdge::phantom(ea, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, ea, phantom_index);

                edges.push(top_edge);
                bottom_edges.insert(0, bottom_edge);

                // bottom section
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

        let mut t0 = IntTriangle::abc(vp, e0.a, e0.b);
        t0.set_neighbor(1, net_builder.triangulation.triangles.len() + 1);
        let mut index = net_builder.insert_triangle_with_neighbor_link(e0, 0, t0);

        let top_edge = TriangleEdge::regular(e0.a, vp, index);

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
            let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
            triangle.set_neighbor(1, next_index);
            triangle.set_neighbor(2, index);
            index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
            next_index = index + 2;

            i += 1;
        }
        net_builder.triangulation.triangles[index].remove_neighbor(1);

        let bottom_edge = TriangleEdge::regular(vp, edges[i - 1].b, index);

        *edges = edges.split_off(i);
        edges.insert(0, bottom_edge);

        top_section
    }

    fn add_from_start(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) {
        let vp = v.index_point();

        let edges = match &mut self.content {
            Content::Point(point) => {
                let edges = vec![TriangleEdge::border(vp, *point)];
                self.content = Content::Edges(edges);
                return;
            }
            Content::Edges(edges) => edges,
        };

        debug_assert!(!edges.is_empty());

        let e0 = unsafe { edges.get_unchecked(0) };

        if Triangle::is_cw_or_line_point(v.this, e0.a.point, e0.b.point) {
            edges.insert(0, TriangleEdge::border(vp, e0.a));
            return;
        }

        let mut index =
            net_builder.insert_triangle_with_neighbor_link(e0, 0, IntTriangle::abc(vp, e0.a, e0.b));

        let mut n = 1;
        let mut eb = e0.b;
        for ei in edges.iter().skip(1) {
            if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
                break;
            }
            eb = ei.b;
            n += 1;
            let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
            triangle.set_neighbor(2, index);
            let prev_index = index;
            index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);

            net_builder.triangulation.triangles[prev_index].set_neighbor(1, index);
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

    fn add_from_end(&mut self, v: &ChainVertex, net_builder: &mut NetBuilder) {
        let vp = v.index_point();
        let edges = match &mut self.content {
            Content::Point(point) => {
                self.content = Content::Edges(vec![TriangleEdge::border(*point, vp)]);
                return;
            }
            Content::Edges(edges) => edges,
        };

        let el = edges.last().unwrap();

        if Triangle::is_cw_or_line_point(v.this, el.a.point, el.b.point) {
            edges.push(TriangleEdge::border(el.b, vp));
            return;
        }

        let mut index =
            net_builder.insert_triangle_with_neighbor_link(el, 0, IntTriangle::abc(vp, el.a, el.b));
        let mut ea = el.a;
        let mut n = 1;
        for ei in edges.iter().rev().skip(1) {
            if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
                break;
            }
            ea = ei.a;
            n += 1;
            let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
            triangle.set_neighbor(1, index);
            let prev_index = index;
            index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);

            net_builder.triangulation.triangles[prev_index].set_neighbor(2, index);
        }
        edges.truncate(edges.len() - n);

        edges.push(TriangleEdge::regular(ea, vp, index));
    }

    #[inline]
    fn add_steiner(&mut self, vp: IndexPoint, net_builder: &mut NetBuilder) {
        let edges = match &mut self.content {
            Content::Point(point) => {
                let phantom_index = net_builder.get_unique_phantom_edge_index();
                let top_edge = TriangleEdge::phantom(*point, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, *point, phantom_index);

                self.content = Content::Edges(vec![top_edge, bottom_edge]);

                return;
            }
            Content::Edges(edges) => edges,
        };

        let mut i = 0;
        while i < edges.len() {
            let ei = &edges[i];
            // skip first not valid triangles
            if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
                i += 1;
                continue;
            }
            break;
        }

        if i >= edges.len() {
            let last = edges[edges.len() - 1].b;
            let mut index = edges.len();
            let mut min_dist = vp.point.x - last.point.x;
            for (ei, e) in edges.iter().enumerate() {
                let dist = vp.point.x - e.a.point.x;
                if dist < min_dist {
                    min_dist = dist;
                    index = ei;
                }
            }

            let phantom_index = net_builder.get_unique_phantom_edge_index();
            if index == edges.len() {
                let top_edge = TriangleEdge::phantom(last, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, last, phantom_index);

                edges.push(top_edge);
                edges.push(bottom_edge);
            } else {
                let ea = edges[index].a;
                let top_edge = TriangleEdge::phantom(ea, vp, phantom_index);
                let bottom_edge = TriangleEdge::phantom(vp, ea, phantom_index);

                edges.insert(index, top_edge);
                edges.insert(index + 1, bottom_edge);
            }
            return;
        }
        let e0 = &edges[i];

        let mut t0 = IntTriangle::abc(vp, e0.a, e0.b);
        t0.set_neighbor(1, net_builder.triangulation.triangles.len() + 1);
        let mut index = net_builder.insert_triangle_with_neighbor_link(e0, 0, t0);

        let top_edge = TriangleEdge::regular(e0.a, vp, index);

        let mut new_edges = edges.split_off(i);
        swap(&mut new_edges, edges);
        new_edges.push(top_edge);

        let mut next_index = index + 2;
        i = 1;
        while i < edges.len() {
            let ei = &edges[i];
            if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
                break;
            }
            let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
            triangle.set_neighbor(1, next_index);
            triangle.set_neighbor(2, index);
            index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
            next_index = index + 2;

            i += 1;
        }
        net_builder.triangulation.triangles[index].remove_neighbor(1);

        let bottom_edge = TriangleEdge::regular(vp, edges[i - 1].b, index);

        let mut tail = edges.split_off(i);

        new_edges.push(bottom_edge);
        new_edges.append(&mut tail);

        self.content = Content::Edges(new_edges);
    }
}

trait FindSection {
    fn find_section(&self, v: &ChainVertex) -> u32;
}

impl<C> FindSection for C
where
    C: SetCollection<VSegment, Section>,
{
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
    extern crate std;

    use alloc::vec;
    use alloc::vec::Vec;
    use std::collections::HashSet;
    use crate::int::binder::SteinerInference;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::core::simplify::Simplify;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;
    use rand::Rng;
    use crate::int::monotone::triangulator::MonotoneTriangulator;
    use crate::int::triangulation::RawIntTriangulation;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 5);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 3);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 6);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 8);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 16);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 24);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 16);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 6);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_14() {
        let shape = vec![path(&[[-2, -3], [-4, -4], [5, -1], [1, -1], [2, 3]])];
        let s = &shape.simplify(FillRule::EvenOdd, IntOverlayOptions::default())[0];

        let shape_area = s.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 3);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_15() {
        let shape = vec![path(&[[0, 2], [2, 0], [5, 0], [4, 6]])];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_16() {
        let shape = vec![path(&[[0, 4], [-4, -3], [-2, -2], [1, -2], [0, -1]])];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 3);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 6);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 5);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 5);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
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

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, None, &mut raw);

        assert_eq!(raw.triangles.len(), 6);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_22() {
        let shape = vec![path(&[[-10, 0], [10, -10], [10, 10]])];
        let points = vec![IntPoint::new(0, 0)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 3);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_23() {
        let shape = vec![path(&[[-10, 0], [0, -10], [10, 0], [0, 10]])];
        let points = vec![IntPoint::new(0, 0)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_24() {
        let shape = vec![path(&[
            [-10, 10],
            [0, 5],
            [0, 0],
            [0, -5],
            [-10, -10],
            [10, -10],
            [10, 10],
        ])];
        let points = vec![IntPoint::new(5, 0)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 7);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_25() {
        let shape = vec![path(&[[-10, 0], [0, -10], [10, 0], [0, 10]])];
        let points = vec![
            IntPoint::new(-2, 0),
            IntPoint::new(-1, 0),
            IntPoint::new(1, -2),
        ];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 8);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_26() {
        let shape = vec![path(&[[4, 4], [-5, 3], [3, -3], [2, 3]])];
        let points = vec![IntPoint::new(1, 3)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_27() {
        let shape = vec![path(&[[3, -1], [0, 0], [1, -1], [3, -5]])];
        let points = vec![IntPoint::new(2, -2)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_28() {
        let shape = vec![path(&[[3, -1], [0, 0], [1, -1], [3, -5]])];
        let points = vec![IntPoint::new(2, -2)];
        let shape_area = shape.area_two();

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_29() {
        let shape = vec![path(&[[1, 0], [-4, -2], [3, 0], [5, 1], [4, 1], [-4, -1]])];
        let points = vec![IntPoint::new(0, 3), IntPoint::new(4, 3)];
        let shape_area = shape.area_two();
        let group = vec![shape.clone()].group_by_shapes(&points);

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&group[0]), &mut raw);

        assert_eq!(raw.triangles.len(), 4);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_30() {
        let shape = vec![path(&[[-1, 2], [-5, -2], [2, -2], [3, 4]])];
        let points = vec![IntPoint::new(1, 5)];
        let shape_area = shape.area_two();
        let group = vec![shape.clone()].group_by_shapes(&points);

        let mut raw = RawIntTriangulation::default();
        MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&group[0]), &mut raw);

        assert_eq!(raw.triangles.len(), 2);
        raw.validate();

        assert_eq!(raw.area_two(), shape_area);
    }

    #[test]
    fn test_random_0() {
        for _ in 0..100_000 {
            let path = random(8, 5);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_1() {
        for _ in 0..100_000 {
            let path = random(10, 6);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_2() {
        for _ in 0..100_000 {
            let path = random(10, 12);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_3() {
        for _ in 0..50_000 {
            let path = random(20, 20);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_4() {
        for _ in 0..10_000 {
            let path = random(30, 50);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
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
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, None, &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_6() {
        let shape = vec![path(&[[-10, 0], [0, -10], [10, 0], [0, 10]])];
        let shape_area = shape.area_two();
        for _ in 0..100_000 {
            let points = random_points(5, 10);

            let mut raw = RawIntTriangulation::default();
            MonotoneTriangulator::default().shape_into_net_triangulation(&shape, Some(&points), &mut raw);

            raw.validate();
            assert_eq!(raw.area_two(), shape_area);
        }
    }

    #[test]
    fn test_random_7() {
        let shapes = vec![vec![path(&[[-5, 0], [0, -5], [5, 0], [0, 5]])]];
        let shape_area = shapes.area_two();
        for _ in 0..100_000 {
            let points = random_points(8, 2);
            let group = shapes.group_by_shapes(&points);

            let mut raw = RawIntTriangulation::default();
            MonotoneTriangulator::default().shape_into_net_triangulation(&shapes[0], Some(&group[0]), &mut raw);

            raw.validate();
            assert_eq!(raw.area_two(), shape_area);
        }
    }

    #[test]
    fn test_random_8() {
        for _ in 0..100_000 {
            let points = random_points(15, 1);
            let shape = random(10, 4);

            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shapes = vec![first.clone()];
                let shape_area = shapes.area_two();

                let group = shapes.group_by_shapes(&points);

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&shapes[0], Some(&group[0]), &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_9() {
        for _ in 0..100_000 {
            let points = random_points(10, 2);
            let shape = random(10, 4);

            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shapes = vec![first.clone()];
                let shape_area = shapes.area_two();

                let group = shapes.group_by_shapes(&points);

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&shapes[0], Some(&group[0]), &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_10() {
        for _ in 0..50_000 {
            let points = random_points(10, 8);
            let shape = random(10, 8);

            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shapes = vec![first.clone()];
                let shape_area = shapes.area_two();

                let group = shapes.group_by_shapes(&points);

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&shapes[0], Some(&group[0]), &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_11() {
        for _ in 0..10_000 {
            let main = random(50, 20);
            let mut shape = vec![main];
            for _ in 0..10 {
                shape.push(random(30, 5));
            }
            let points = random_points(20, 8);
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                let group = vec![first.clone()].group_by_shapes(&points);

                let mut raw = RawIntTriangulation::default();
                MonotoneTriangulator::default().shape_into_net_triangulation(&first, Some(&group[0]), &mut raw);

                raw.validate();
                assert_eq!(raw.area_two(), shape_area);
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

    fn random_points(radius: i32, n: usize) -> Vec<IntPoint> {
        let a = radius / 2;
        let mut points = HashSet::new();
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a);
            let y = rng.random_range(-a..=a);
            points.insert(IntPoint { x, y });
        }

        points.iter().map(|p| p).copied().collect()
    }
}