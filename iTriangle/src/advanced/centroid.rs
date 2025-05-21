use alloc::vec;
use alloc::vec::Vec;
use crate::advanced::delaunay::IntDelaunay;
use crate::geom::triangle::IntTriangle;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::area::Area;
use i_overlay::i_shape::int::shape::IntContour;

impl IntDelaunay {

    /// Constructs a centroid-based polygonal net from the Delaunay triangulation.
    /// Each polygon surrounds a vertex using adjacent triangle centers and edge midpoints.
    ///
    /// This is similar to a centroidal Voronoi diagram.
    ///
    /// # Parameters
    /// - `min_area`: minimum polygon area to emit
    ///
    /// # Returns
    /// A list of `IntContour` objects forming closed or convex polygonal regions.
    pub fn centroid_net(&self, min_area: u64) -> Vec<IntContour> {
        let two_area = min_area << 1;
        let n = self.triangles.len();

        let mut visited_index = vec![false; self.points.len()];
        let mut result = Vec::with_capacity(self.points.len() / 4);

        for triangle_index in 0..n {
            for v in self.triangles[triangle_index].vertices.iter() {
                if visited_index[v.index] {
                    continue;
                }
                visited_index[v.index] = true;

                // go in counter-clockwise direction first

                let mut contour = IntContour::with_capacity(16);
                let mut t = &self.triangles[triangle_index];
                let (mut next_index, mut mid) = t.left_neighbor_and_mid_edge(v.index);
                contour.push(t.center());
                contour.push(mid);
                while next_index < self.triangles.len() && next_index != triangle_index {
                    t = &self.triangles[next_index];
                    (next_index, mid) = t.left_neighbor_and_mid_edge(v.index);
                    contour.push(t.center());
                    contour.push(mid);
                }

                if next_index == triangle_index {
                    // it's a closed contour
                    result.add_area_check(contour, two_area);
                    continue;
                }

                // collect other part in clockwise direction

                let mut start_contour = Vec::with_capacity(8);
                t = &self.triangles[triangle_index];
                let (mut next_index, mut mid) = t.right_neighbor_and_mid_edge(v.index);
                start_contour.push(mid);
                while next_index < self.triangles.len() {
                    t = &self.triangles[next_index];
                    (next_index, mid) = t.right_neighbor_and_mid_edge(v.index);
                    start_contour.push(t.center());
                    start_contour.push(mid);
                }

                start_contour.reverse(); // make it counter-clockwise
                start_contour.append(&mut contour);
                start_contour.push(v.point);

                result.add_area_check(start_contour, two_area);
            }
        }

        result
    }
}

trait SafeAdd {
    fn add_area_check(&mut self, contour: IntContour, two_area: u64);
}

impl SafeAdd for Vec<IntContour> {
    fn add_area_check(&mut self, contour: IntContour, two_area: u64) {
        if two_area == 0 || contour.area_two().unsigned_abs() > two_area {
            self.push(contour);
        }
    }
}

impl IntTriangle {
    #[inline]
    fn right_neighbor_and_mid_edge(&self, vertex_index: usize) -> (usize, IntPoint) {
        if self.vertices[0].index == vertex_index {
            let neighbor = self.neighbors[2];
            let mid = middle(self.vertices[0].point, self.vertices[1].point);
            (neighbor, mid)
        } else if self.vertices[1].index == vertex_index {
            let neighbor = self.neighbors[0];
            let mid = middle(self.vertices[1].point, self.vertices[2].point);
            (neighbor, mid)
        } else {
            let neighbor = self.neighbors[1];
            let mid = middle(self.vertices[2].point, self.vertices[0].point);
            (neighbor, mid)
        }
    }

    #[inline]
    fn left_neighbor_and_mid_edge(&self, vertex_index: usize) -> (usize, IntPoint) {
        if self.vertices[0].index == vertex_index {
            let neighbor = self.neighbors[1];
            let mid = middle(self.vertices[0].point, self.vertices[2].point);
            (neighbor, mid)
        } else if self.vertices[1].index == vertex_index {
            let neighbor = self.neighbors[2];
            let mid = middle(self.vertices[1].point, self.vertices[0].point);
            (neighbor, mid)
        } else {
            let neighbor = self.neighbors[0];
            let mid = middle(self.vertices[2].point, self.vertices[1].point);
            (neighbor, mid)
        }
    }

    #[inline]
    fn center(&self) -> IntPoint {
        let a = self.vertices[0].point;
        let b = self.vertices[1].point;
        let c = self.vertices[2].point;

        let x = a.x as i64 + b.x as i64 + c.x as i64;
        let y = a.y as i64 + b.y as i64 + c.y as i64;

        IntPoint::new((x / 3) as i32, (y / 3) as i32)
    }
}

#[inline]
fn middle(a: IntPoint, b: IntPoint) -> IntPoint {
    let x = a.x as i64 + b.x as i64;
    let y = a.y as i64 + b.y as i64;
    IntPoint::new((x / 2) as i32, (y / 2) as i32)
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;
    use crate::int::triangulatable::IntTriangulatable;

    #[test]
    fn test_0() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let centroids = contour.triangulate_with_steiner_points(&[IntPoint::new(5, 5)])
            .into_delaunay().centroid_net(0);
        assert_eq!(centroids.len(), 5);
    }
}