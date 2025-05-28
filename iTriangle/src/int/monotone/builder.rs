// use alloc::vec;
// use alloc::vec::Vec;
// use crate::int::triangulation::{IndexType, IndicesBuilder, IntTriangulation};
// use crate::int::meta::TrianglesCount;
// use crate::int::triangulation::RawIntTriangulation;
// use crate::geom::point::IndexPoint;
// use crate::geom::triangle::IntTriangle;
// use crate::int::monotone::chain::builder::ChainBuilder;
// use crate::int::monotone::chain::vertex::ChainVertex;
// use crate::int::monotone::chain::vertex::VertexType;
// use crate::int::monotone::net::phantom::PhantomEdgePool;
// use crate::int::monotone::net::phantom::PhantomHandler;
// use crate::int::monotone::net::section::{Content, EdgeType, Section, TriangleEdge};
// use crate::int::monotone::v_segment::VSegment;
// use i_overlay::i_float::int::point::IntPoint;
// use i_overlay::i_float::triangle::Triangle;
// use i_overlay::i_shape::int::shape::{IntContour, IntShape};
// use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
// use i_tree::set::list::SetList;
// use i_tree::set::sort::SetCollection;
// use i_tree::set::tree::SetTree;
// use core::cmp::Ordering;
// use core::mem::swap;
// use i_overlay::i_shape::util::reserve::Reserve;
// use crate::advanced::buffer::DelaunayBuffer;
// use crate::advanced::delaunay::DelaunayRefine;
// use crate::int::meta::MeshMetaProvider;
//
// pub(crate) struct TrianglesBuilder {
//     chain_builder: ChainBuilder,
//     vertices: Option<Vec<ChainVertex>>,
//     triangles: Vec<IntTriangle>,
//     phantom_store: PhantomEdgePool,
//     buffer: Option<DelaunayBuffer>,
// }
//
// impl TrianglesBuilder {
//
//     #[inline]
//     pub(crate) fn shape_triangulation(shape: &IntShape, points: Option<&[IntPoint]>) -> RawIntTriangulation {
//         if shape.len() <= 1 {
//             return if let Some(contour) = shape.first() {
//                 Self::contour_triangulation(contour, points)
//             } else {
//                 RawIntTriangulation::empty()
//             };
//         }
//
//         let points_count = points.map(|points| points.len()).unwrap_or(0);
//         let meta = shape.meta(points_count);
//         let mut builder = Self::with_capacity(meta.triangles_count, meta.vertices_count);
//
//         builder.chain_builder.shape_to_vertices(shape, points);
//         builder.build();
//         builder.into_raw_triangulation()
//     }
//
//     #[inline]
//     pub(crate) fn contour_triangulation(contour: &IntContour, points: Option<&[IntPoint]>) -> RawIntTriangulation {
//         let points_count = points.map(|points| points.len()).unwrap_or(0);
//         let meta = contour.meta(points_count);
//         let mut builder = Self::with_capacity(meta.triangles_count, meta.vertices_count);
//
//         builder.chain_builder.contour_to_vertices(contour, points);
//         builder.build();
//         builder.into_raw_triangulation()
//     }
//
//     #[inline]
//     pub(crate) fn build_flat(&mut self, flat: &FlatContoursBuffer) {
//         self.chain_builder.flat_to_vertices(flat);
//         self.reserve_and_clear_triangles(flat.triangles_count(0));
//         self.build();
//     }
//
//     #[inline]
//     pub(crate) fn build_shape(&mut self, shape: &IntShape, points: Option<&[IntPoint]>) {
//         if shape.len() <= 1 {
//             if let Some(contour) = shape.first() {
//                 self.build_contour(contour, points);
//             };
//             return;
//         }
//
//         self.chain_builder.shape_to_vertices(shape, points);
//         let points_count = points.map(|points| points.len()).unwrap_or(0);
//         self.reserve_and_clear_triangles(shape.triangles_count(points_count));
//
//         self.build();
//     }
//
//     #[inline]
//     pub(crate) fn build_contour(&mut self, contour: &IntContour, points: Option<&[IntPoint]>) {
//         self.chain_builder.contour_to_vertices(contour, points);
//         let points_count = points.map(|points| points.len()).unwrap_or(0);
//         self.reserve_and_clear_triangles(contour.triangles_count(points_count));
//         self.build();
//     }
//
//     #[inline]
//     pub(crate) fn with_capacity(triangles_capacity: usize, vertices_capacity: usize) -> Self {
//         Self {
//             triangles: Vec::with_capacity(triangles_capacity),
//             chain_builder: ChainBuilder::with_capacity(vertices_capacity),
//             phantom_store: PhantomEdgePool::new(),
//             buffer: None,
//         }
//     }
//
//     #[inline]
//     pub(crate) fn build(&mut self) {
//         let n = self.chain_builder.vertices.len();
//         let capacity = if n < 128 { 4 } else { n.ilog2() as usize };
//         if capacity <= 12 {
//             self.build_with_store(SetList::new(capacity))
//         } else {
//             self.build_with_store(SetTree::new(capacity))
//         }
//     }
//
//     #[inline]
//     pub(crate) fn into_raw_triangulation(self) -> RawIntTriangulation {
//         RawIntTriangulation {
//             triangles: self.triangles,
//             points: self.chain_builder.to_points(),
//         }
//     }
//
//     #[inline]
//     pub(crate) fn feed_triangulation<I: IndexType>(&self, triangulation: &mut IntTriangulation<I>) {
//         self.chain_builder.feed_points(&mut triangulation.points);
//         self.triangles.feed_indices(triangulation.points.len(), &mut triangulation.indices)
//     }
//
//     #[inline]
//     pub(crate) fn delaunay_refine(&mut self) {
//         let mut buffer = self.buffer.take().unwrap_or_default();
//         self.triangles.build_with_buffer(&mut buffer)
//     }
// }
//
// impl TrianglesBuilder {
//
//     #[inline]
//     fn build_with_store<S: SetCollection<VSegment, Section>>(&mut self, mut store: S) {
//         let vertices_ptr = self.chain_builder.vertices.as_ptr();
//         let len = self.chain_builder.vertices.len();
//
//         for i in 0..len {
//             // SAFETY: `vertices` is not modified.
//             let v = unsafe { &*vertices_ptr.add(i) };
//
//             match v.get_type() {
//                 VertexType::Start => self.start(v, &mut store),
//                 VertexType::End => self.end(v, &mut store),
//                 VertexType::Merge => self.merge(v, &mut store),
//                 VertexType::Split => self.split(v, &mut store),
//                 VertexType::Join => self.join(v, &mut store),
//                 VertexType::Steiner => self.steiner(v, &mut store),
//             }
//         }
//     }
//
//     #[inline]
//     fn next_triangle_index(&self) -> usize {
//         self.triangles.len()
//     }
//
//     #[inline]
//     fn get_unique_phantom_edge_index(&mut self) -> usize {
//         self.phantom_store.alloc_phantom_index()
//     }
//
//     #[inline]
//     fn insert_triangle_with_neighbor_link(
//         &mut self,
//         edge: &TriangleEdge,
//         vertex: usize,
//         mut new_triangle: IntTriangle,
//     ) -> usize {
//         let new_index = self.next_triangle_index();
//         match edge.kind {
//             EdgeType::Regular(triangle_index) => {
//                 if self.triangles.len() <= triangle_index {
//                     self.triangles.push(new_triangle);
//                     return new_index;
//                 }
//                 new_triangle.set_neighbor(vertex, triangle_index);
//                 let other = &mut self.triangles[triangle_index];
//                 let vi = other.other_vertex(edge.a.index, edge.b.index);
//                 other.set_neighbor(vi, new_index);
//             }
//             EdgeType::Phantom(edge_index) => {
//                 if let Some(handler) = self.phantom_store.get(edge_index) {
//                     // if exist update neighbor
//                     self.triangles[handler.triangle].set_neighbor(handler.vertex, new_index);
//                     new_triangle.set_neighbor(vertex, handler.triangle);
//                     self.phantom_store.free_phantom_index(edge_index);
//                 } else {
//                     // create a phantom edge
//                     self.phantom_store.register_phantom_link(
//                         edge_index,
//                         PhantomHandler {
//                             vertex,
//                             triangle: new_index,
//                         },
//                     );
//                 }
//             }
//         }
//         self.triangles.push(new_triangle);
//
//         new_index
//     }
//
//     #[inline]
//     fn join<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let index = tree.find_section(v);
//         let section = tree.value_by_index_mut(index);
//         if section.sort.b == v.this {
//             section.add_to_bottom(v, self);
//         } else {
//             section.add_to_top(v, self);
//         }
//     }
//
//     #[inline]
//     fn start<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let section = Section {
//             sort: VSegment {
//                 a: v.this,
//                 b: v.next,
//             },
//             content: Content::Point(v.index_point()),
//         };
//         tree.insert(section);
//     }
//
//     #[inline]
//     fn end<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let index = tree.find_section(v);
//         let section = tree.value_by_index_mut(index);
//         section.add_as_last(v, self);
//         tree.delete_by_index(index);
//     }
//
//     fn split<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let index = tree.find_section(v);
//         let section = tree.value_by_index_mut(index);
//         let new_section = section.add_to_middle(v, self);
//         tree.insert(new_section);
//     }
//
//     fn merge<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let prev_index = tree.find_section(v);
//         let next_index = tree.index_before(prev_index);
//         let next = tree.value_by_index_mut(next_index);
//         next.add_from_start(v, self);
//
//         let mut next_edges = if let Content::Edges(edges) = &next.content {
//             edges.clone()
//         } else {
//             Vec::new()
//         };
//
//         let sort = next.sort;
//
//         let prev = tree.value_by_index_mut(prev_index);
//         prev.add_from_end(v, self);
//
//         match &mut prev.content {
//             Content::Point(_) => {}
//             Content::Edges(edges) => edges.append(&mut next_edges),
//         }
//
//         prev.sort = sort;
//
//         tree.delete_by_index(next_index);
//     }
//
//     fn steiner<S: SetCollection<VSegment, Section>>(&mut self, v: &ChainVertex, tree: &mut S) {
//         let index = tree.find_section(v);
//         let section = tree.value_by_index_mut(index);
//         section.add_steiner(v.index_point(), self);
//     }
// }
//
// impl Section {
//     #[inline]
//     fn add_as_last(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) {
//         let edges = match &mut self.content {
//             Content::Edges(edges) => edges,
//             Content::Point(_) => unreachable!("Section with less then 3 points not possible"),
//         };
//
//         let vp = v.index_point();
//         let mut prev_index = usize::MAX;
//
//         // Iterate all but last edge
//         for ei in edges.iter().take(edges.len().saturating_sub(1)) {
//             let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
//             triangle.set_neighbor(1, net_builder.next_triangle_index() + 1);
//             triangle.set_neighbor(2, prev_index);
//
//             prev_index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
//         }
//
//         // Final triangle links only to previous
//         if let Some(last_edge) = edges.last() {
//             let mut triangle = IntTriangle::abc(vp, last_edge.a, last_edge.b);
//             triangle.set_neighbor(2, prev_index);
//             net_builder.insert_triangle_with_neighbor_link(last_edge, 0, triangle);
//         }
//     }
//
//     #[inline]
//     fn add_to_top(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) {
//         self.add_from_start(v, net_builder);
//     }
//
//     #[inline]
//     fn add_to_bottom(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) {
//         self.sort = VSegment {
//             a: v.this,
//             b: v.next,
//         };
//         self.add_from_end(v, net_builder);
//     }
//
//     #[inline]
//     fn add_to_middle(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) -> Section {
//         let edges = match &mut self.content {
//             Content::Point(point) => {
//                 let phantom_index = net_builder.get_unique_phantom_edge_index();
//                 let vp = v.index_point();
//                 let top_edge = TriangleEdge::phantom(*point, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, *point, phantom_index);
//
//                 self.content = Content::Edges(vec![top_edge]);
//
//                 // bottom
//                 let bottom_section = Section {
//                     sort: self.sort,
//                     content: Content::Edges(vec![bottom_edge]),
//                 };
//
//                 self.sort = VSegment {
//                     a: v.this,
//                     b: v.next,
//                 };
//
//                 return bottom_section;
//             }
//             Content::Edges(edges) => edges,
//         };
//
//         let mut i = 0;
//         while i < edges.len() {
//             let ei = &edges[i];
//             // skip first not valid triangles
//             if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
//                 i += 1;
//                 continue;
//             }
//             break;
//         }
//
//         let vp = v.index_point();
//         if i >= edges.len() {
//             let last = edges[edges.len() - 1].b;
//             let mut index = edges.len();
//             let mut min_dist = vp.point.x - last.point.x;
//             for (ei, e) in edges.iter().enumerate() {
//                 let dist = vp.point.x - e.a.point.x;
//                 if dist < min_dist {
//                     min_dist = dist;
//                     index = ei;
//                 }
//             }
//
//             let phantom_index = net_builder.get_unique_phantom_edge_index();
//
//             return if index == edges.len() {
//                 let eb = edges[i - 1].b;
//                 let top_edge = TriangleEdge::phantom(eb, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, eb, phantom_index);
//                 edges.push(top_edge);
//
//                 let bottom_section = Section {
//                     sort: self.sort,
//                     content: Content::Edges(vec![bottom_edge]),
//                 };
//
//                 self.sort = VSegment {
//                     a: v.this,
//                     b: v.next,
//                 };
//
//                 bottom_section
//             } else {
//                 let ea = edges[index].a;
//                 let mut bottom_edges = edges.split_off(index);
//
//                 let top_edge = TriangleEdge::phantom(ea, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, ea, phantom_index);
//
//                 edges.push(top_edge);
//                 bottom_edges.insert(0, bottom_edge);
//
//                 // bottom section
//                 let bottom_section = Section {
//                     sort: self.sort,
//                     content: Content::Edges(bottom_edges),
//                 };
//
//                 self.sort = VSegment {
//                     a: v.this,
//                     b: v.next,
//                 };
//
//                 bottom_section
//             };
//         }
//         let e0 = &edges[i];
//
//         let mut t0 = IntTriangle::abc(vp, e0.a, e0.b);
//         t0.set_neighbor(1, net_builder.triangles.len() + 1);
//         let mut index = net_builder.insert_triangle_with_neighbor_link(e0, 0, t0);
//
//         let top_edge = TriangleEdge::regular(e0.a, vp, index);
//
//         let mut top_edges = edges.split_off(i);
//         swap(&mut top_edges, edges);
//         top_edges.push(top_edge);
//
//         let top_section = Section {
//             sort: VSegment {
//                 a: v.this,
//                 b: v.next,
//             },
//             content: Content::Edges(top_edges),
//         };
//
//         let mut next_index = index + 2;
//         i = 1;
//         while i < edges.len() {
//             let ei = &edges[i];
//             if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
//                 break;
//             }
//             let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
//             triangle.set_neighbor(1, next_index);
//             triangle.set_neighbor(2, index);
//             index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
//             next_index = index + 2;
//
//             i += 1;
//         }
//         net_builder.triangles[index].remove_neighbor(1);
//
//         let bottom_edge = TriangleEdge::regular(vp, edges[i - 1].b, index);
//
//         *edges = edges.split_off(i);
//         edges.insert(0, bottom_edge);
//
//         top_section
//     }
//
//     fn add_from_start(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) {
//         let vp = v.index_point();
//
//         let edges = match &mut self.content {
//             Content::Point(point) => {
//                 let edges = vec![TriangleEdge::border(vp, *point)];
//                 self.content = Content::Edges(edges);
//                 return;
//             }
//             Content::Edges(edges) => edges,
//         };
//
//         debug_assert!(!edges.is_empty());
//
//         let e0 = unsafe { edges.get_unchecked(0) };
//
//         if Triangle::is_cw_or_line_point(v.this, e0.a.point, e0.b.point) {
//             edges.insert(0, TriangleEdge::border(vp, e0.a));
//             return;
//         }
//
//         let mut index =
//             net_builder.insert_triangle_with_neighbor_link(e0, 0, IntTriangle::abc(vp, e0.a, e0.b));
//
//         let mut n = 1;
//         let mut eb = e0.b;
//         for ei in edges.iter().skip(1) {
//             if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
//                 break;
//             }
//             eb = ei.b;
//             n += 1;
//             let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
//             triangle.set_neighbor(2, index);
//             let prev_index = index;
//             index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
//
//             net_builder.triangles[prev_index].set_neighbor(1, index);
//         }
//
//         if edges.len() == n {
//             edges.clear();
//         } else {
//             *edges = edges.split_off(n);
//         }
//
//         edges.insert(
//             0,
//             TriangleEdge {
//                 a: vp,
//                 b: eb,
//                 kind: EdgeType::Regular(index),
//             },
//         );
//     }
//
//     fn add_from_end(&mut self, v: &ChainVertex, net_builder: &mut TrianglesBuilder) {
//         let vp = v.index_point();
//         let edges = match &mut self.content {
//             Content::Point(point) => {
//                 self.content = Content::Edges(vec![TriangleEdge::border(*point, vp)]);
//                 return;
//             }
//             Content::Edges(edges) => edges,
//         };
//
//         let el = edges.last().unwrap();
//
//         if Triangle::is_cw_or_line_point(v.this, el.a.point, el.b.point) {
//             edges.push(TriangleEdge::border(el.b, vp));
//             return;
//         }
//
//         let mut index =
//             net_builder.insert_triangle_with_neighbor_link(el, 0, IntTriangle::abc(vp, el.a, el.b));
//         let mut ea = el.a;
//         let mut n = 1;
//         for ei in edges.iter().rev().skip(1) {
//             if Triangle::is_cw_or_line_point(v.this, ei.a.point, ei.b.point) {
//                 break;
//             }
//             ea = ei.a;
//             n += 1;
//             let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
//             triangle.set_neighbor(1, index);
//             let prev_index = index;
//             index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
//
//             net_builder.triangles[prev_index].set_neighbor(2, index);
//         }
//         edges.truncate(edges.len() - n);
//
//         edges.push(TriangleEdge::regular(ea, vp, index));
//     }
//
//     #[inline]
//     fn add_steiner(&mut self, vp: IndexPoint, net_builder: &mut TrianglesBuilder) {
//         let edges = match &mut self.content {
//             Content::Point(point) => {
//                 let phantom_index = net_builder.get_unique_phantom_edge_index();
//                 let top_edge = TriangleEdge::phantom(*point, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, *point, phantom_index);
//
//                 self.content = Content::Edges(vec![top_edge, bottom_edge]);
//
//                 return;
//             }
//             Content::Edges(edges) => edges,
//         };
//
//         let mut i = 0;
//         while i < edges.len() {
//             let ei = &edges[i];
//             // skip first not valid triangles
//             if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
//                 i += 1;
//                 continue;
//             }
//             break;
//         }
//
//         if i >= edges.len() {
//             let last = edges[edges.len() - 1].b;
//             let mut index = edges.len();
//             let mut min_dist = vp.point.x - last.point.x;
//             for (ei, e) in edges.iter().enumerate() {
//                 let dist = vp.point.x - e.a.point.x;
//                 if dist < min_dist {
//                     min_dist = dist;
//                     index = ei;
//                 }
//             }
//
//             let phantom_index = net_builder.get_unique_phantom_edge_index();
//             if index == edges.len() {
//                 let top_edge = TriangleEdge::phantom(last, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, last, phantom_index);
//
//                 edges.push(top_edge);
//                 edges.push(bottom_edge);
//             } else {
//                 let ea = edges[index].a;
//                 let top_edge = TriangleEdge::phantom(ea, vp, phantom_index);
//                 let bottom_edge = TriangleEdge::phantom(vp, ea, phantom_index);
//
//                 edges.insert(index, top_edge);
//                 edges.insert(index + 1, bottom_edge);
//             }
//             return;
//         }
//         let e0 = &edges[i];
//
//         let mut t0 = IntTriangle::abc(vp, e0.a, e0.b);
//         t0.set_neighbor(1, net_builder.triangles.len() + 1);
//         let mut index = net_builder.insert_triangle_with_neighbor_link(e0, 0, t0);
//
//         let top_edge = TriangleEdge::regular(e0.a, vp, index);
//
//         let mut new_edges = edges.split_off(i);
//         swap(&mut new_edges, edges);
//         new_edges.push(top_edge);
//
//         let mut next_index = index + 2;
//         i = 1;
//         while i < edges.len() {
//             let ei = &edges[i];
//             if Triangle::is_cw_or_line_point(vp.point, ei.a.point, ei.b.point) {
//                 break;
//             }
//             let mut triangle = IntTriangle::abc(vp, ei.a, ei.b);
//             triangle.set_neighbor(1, next_index);
//             triangle.set_neighbor(2, index);
//             index = net_builder.insert_triangle_with_neighbor_link(ei, 0, triangle);
//             next_index = index + 2;
//
//             i += 1;
//         }
//         net_builder.triangles[index].remove_neighbor(1);
//
//         let bottom_edge = TriangleEdge::regular(vp, edges[i - 1].b, index);
//
//         let mut tail = edges.split_off(i);
//
//         new_edges.push(bottom_edge);
//         new_edges.append(&mut tail);
//
//         self.content = Content::Edges(new_edges);
//     }
// }
//
// trait FindSection {
//     fn find_section(&self, v: &ChainVertex) -> u32;
// }
//
// impl<C> FindSection for C
// where
//     C: SetCollection<VSegment, Section>,
// {
//     #[inline]
//     fn find_section(&self, v: &ChainVertex) -> u32 {
//         self.first_index_less_by(|s| {
//             let point_search = s.is_under_point_order(v.this);
//             match point_search {
//                 Ordering::Equal => {
//                     if v.prev == s.a {
//                         Ordering::Equal
//                     } else {
//                         Triangle::clock_order_point(s.a, v.next, s.b)
//                     }
//                 }
//                 _ => point_search,
//             }
//         })
//     }
// }
//
// #[cfg(test)]
// impl RawIntTriangulation {
//     pub fn validate(&self) {
//         for (i, t) in self.triangles.iter().enumerate() {
//             let a = t.vertices[0].point;
//             let b = t.vertices[1].point;
//             let c = t.vertices[2].point;
//             let area = Triangle::area_two_point(a, b, c);
//             assert!(area <= 0);
//
//             let n0 = t.neighbors[0];
//             let n1 = t.neighbors[1];
//             let n2 = t.neighbors[2];
//
//             if n0 != usize::MAX {
//                 assert!(self.triangles[n0].neighbors.contains(&i));
//             }
//             if n1 != usize::MAX {
//                 assert!(self.triangles[n1].neighbors.contains(&i));
//             }
//             if n2 != usize::MAX {
//                 assert!(self.triangles[n2].neighbors.contains(&i));
//             }
//         }
//     }
//
//     pub fn area(&self) -> i64 {
//         let mut s = 0;
//         for t in self.triangles.iter() {
//             let a = t.vertices[0].point;
//             let b = t.vertices[1].point;
//             let c = t.vertices[2].point;
//
//             s += Triangle::area_two_point(a, b, c);
//         }
//
//         s
//     }
// }
//
