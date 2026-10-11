use crate::advanced::delaunay::IntDelaunay;
use crate::geom::triangle::IntTriangle;
use crate::index::Index;
use alloc::vec;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::IntContour;
use i_overlay::i_shape::int::simple::Simplify;

#[derive(Debug, Clone, Copy)]
struct Node<I: IntNumber> {
    next: usize,
    index: usize,
    prev: usize,
    point: IntPoint<I>,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    triangle_index: usize,
    neighbor: usize,
    a: usize,
    b: usize,
}

struct ConvexPolygonBuilder<I: IntNumber> {
    nodes: Vec<Node<I>>,
    edges: Vec<Edge>,
}

impl<I: IntNumber> ConvexPolygonBuilder<I> {
    fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(16),
            edges: Vec::with_capacity(16),
        }
    }

    fn to_contour(&self) -> IntContour<I> {
        let count = self.nodes.len();
        let mut contour = Vec::with_capacity(count);

        let mut node = self.nodes[count - 1];
        for _ in 0..count {
            contour.push(node.point);
            node = self.nodes[node.next];
        }

        contour.simplify_contour();

        contour
    }

    fn start(&mut self, triangle_index: usize, triangle: &IntTriangle<I>) {
        self.nodes.clear();
        self.edges.clear();

        let bc = triangle.neighbors[0];
        let ca = triangle.neighbors[1];
        let ab = triangle.neighbors[2];

        let is_ca_inner = ca.is_not_nil();
        let is_ab_inner = ab.is_not_nil();
        let is_bc_inner = bc.is_not_nil();

        self.nodes.push(Node {
            next: 1,
            index: 0,
            prev: 2,
            point: triangle.vertices[0].point,
        });
        self.nodes.push(Node {
            next: 2,
            index: 1,
            prev: 0,
            point: triangle.vertices[1].point,
        });
        self.nodes.push(Node {
            next: 0,
            index: 2,
            prev: 1,
            point: triangle.vertices[2].point,
        });

        if is_ab_inner {
            self.edges.push(Edge {
                triangle_index,
                neighbor: ab,
                a: 0,
                b: 1,
            })
        }

        if is_bc_inner {
            self.edges.push(Edge {
                triangle_index,
                neighbor: bc,
                a: 1,
                b: 2,
            })
        }

        if is_ca_inner {
            self.edges.push(Edge {
                triangle_index,
                neighbor: ca,
                a: 2,
                b: 0,
            })
        }
    }

    fn add(&mut self, edge: Edge, triangle: &IntTriangle<I>) -> bool {
        let v_index = triangle.opposite(edge.triangle_index);
        let v = triangle.vertices[v_index];

        // a0 -> a1 -> p

        let mut node_a1 = self.nodes[edge.a];
        let va0 = self.nodes[node_a1.prev].point;
        let va1 = node_a1.point;

        let aa = va1 - va0;
        let ap = v.point - va1;

        let apa = aa.cross_product(ap);
        if apa < I::Wide::ZERO {
            return false;
        }

        // b0 <- b1 <- p

        let mut node_b1 = self.nodes[edge.b];
        let vb0 = self.nodes[node_b1.next].point;
        let vb1 = node_b1.point;

        let bb = vb0 - vb1;
        let bp = vb1 - v.point;

        let bpb = bp.cross_product(bb);
        if bpb < I::Wide::ZERO {
            return false;
        }

        let prev_neighbor = triangle.neighbors[(v_index + 2) % 3];
        let next_neighbor = triangle.neighbors[(v_index + 1) % 3];

        let new_index = self.nodes.len();

        let new_node = Node {
            next: node_b1.index,
            index: new_index,
            prev: node_a1.index,
            point: v.point,
        };

        node_a1.next = new_index;
        node_b1.prev = new_index;

        self.nodes.push(new_node);
        self.nodes[node_a1.index] = node_a1;
        self.nodes[node_b1.index] = node_b1;

        if next_neighbor.is_not_nil() {
            let edge = Edge {
                triangle_index: edge.neighbor,
                neighbor: next_neighbor,
                a: edge.a,
                b: new_index,
            };
            self.edges.push(edge);
        }

        if prev_neighbor.is_not_nil() {
            let edge = Edge {
                triangle_index: edge.neighbor,
                neighbor: prev_neighbor,
                a: new_index,
                b: edge.b,
            };
            self.edges.push(edge)
        }

        true
    }
}

impl<I: IntNumber> IntDelaunay<I> {
    /// Groups adjacent triangles into convex polygons in counter-clockwise order.
    ///
    /// This method traverses the Delaunay triangulation and greedily merges
    /// triangles into larger convex regions, ensuring the result is always convex.
    ///
    /// # Returns
    /// A `Vec<IntContour>` where each path is a **counter-clockwise** convex polygon.
    /// No two polygons overlap, and all are composed of original Delaunay triangles.
    ///
    /// # Guarantees
    /// - Each polygon is strictly convex.
    /// - Vertices are ordered counter-clockwise.
    /// - The union of all polygons equals the area of the Delaunay mesh.
    ///
    /// # Example
    /// ```rust
    /// use i_overlay::i_float::int::point::IntPoint;
    /// use i_triangle::int::triangulatable::IntTriangulatable;
    /// let path = vec![IntPoint::new(0, 0), IntPoint::new(2, 0), IntPoint::new(1, 2)];
    /// let triangulation = path.triangulate().into_delaunay();
    /// let polygons = triangulation.to_convex_polygons();
    /// assert!(!polygons.is_empty());
    /// ```
    pub fn to_convex_polygons(&self) -> Vec<IntContour<I>> {
        self.convex_polygons(usize::MAX)
    }

    /// Returns counter-clockwise convex polygons with at most `max_vertices.max(3)` vertices.
    ///
    /// First builds the same convex regions as [`Self::to_convex_polygons`], then
    /// splits oversized, simplified contours into a fan using existing vertices.
    ///
    /// Values of `max_vertices` below 3 are treated as 3.
    pub fn to_convex_polygons_with_max_vertices(&self, max_vertices: usize) -> Vec<IntContour<I>> {
        self.convex_polygons(max_vertices.max(3))
    }

    fn convex_polygons(&self, max_vertices: usize) -> Vec<IntContour<I>> {
        let mut result = Vec::new();
        let n = self.triangles.len();

        let mut visited = vec![false; n];

        let mut builder = ConvexPolygonBuilder::new();

        for i in 0..n {
            if visited[i] {
                continue;
            }

            let first = &self.triangles[i];
            builder.start(i, first);
            visited[i] = true;

            while let Some(edge) = builder.edges.pop() {
                if visited[edge.neighbor] {
                    continue;
                }
                let triangle = &self.triangles[edge.neighbor];
                if builder.add(edge, triangle) {
                    visited[edge.neighbor] = true;
                }
            }

            append_fan_parts(builder.to_contour(), max_vertices, &mut result);
        }

        result
    }
}

fn append_fan_parts<I: IntNumber>(
    contour: IntContour<I>,
    max_vertices: usize,
    result: &mut Vec<IntContour<I>>,
) {
    if contour.len() <= max_vertices {
        result.push(contour);
        return;
    }

    let triangle_count = contour.len() - 2;
    let part_count = triangle_count.div_ceil(max_vertices - 2);
    let triangles_per_part = triangle_count / part_count;
    let extra = triangle_count % part_count;
    let mut start = 1;

    for i in 0..part_count {
        let end = start + triangles_per_part + usize::from(i < extra);
        let mut part = Vec::with_capacity(end - start + 2);
        part.push(contour[0]);
        part.extend_from_slice(&contour[start..=end]);
        result.push(part);
        start = end;
    }
}

#[cfg(test)]
mod tests {
    use crate::int::triangulatable::IntTriangulatable;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;

    fn path(slice: &[[i32; 2]]) -> IntPath<i32> {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    #[test]
    fn test_0() {
        let path = path(&[[0, 0], [-2, 0], [-2, -2], [2, -2], [2, 2], [0, 2]]);
        let polygons = path.triangulate().into_delaunay().to_convex_polygons();

        assert_eq!(polygons.len(), 2);

        assert!(polygons[0].area_two() > 0);
        assert!(polygons[1].area_two() > 0);
    }

    #[test]
    fn test_1() {
        let path = path(&[[-1, -1], [1, -1], [1, 1], [0, 2], [-1, 1]]);
        let polygons = path.triangulate().into_delaunay().to_convex_polygons();

        assert_eq!(polygons.len(), 1);

        assert!(polygons[0].area_two() > 0);
    }

    #[test]
    fn test_2() {
        let path = path(&[
            [-3, 1],
            [-3, -1],
            [-1, -1],
            [-1, -3],
            [1, -3],
            [1, -1],
            [3, -1],
            [3, 1],
            [1, 1],
            [1, 3],
            [-1, 3],
            [-1, 1],
        ]);
        let polygons = path.triangulate().into_delaunay().to_convex_polygons();

        assert_eq!(polygons.len(), 3);

        assert!(polygons[0].area_two() > 0);
        assert!(polygons[1].area_two() > 0);
        assert!(polygons[2].area_two() > 0);
    }
}
