use crate::advanced::delaunay::IntDelaunay;
use crate::geom::triangle::IntTriangle;
use alloc::vec;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::area::Area;
use i_overlay::i_shape::int::shape::IntContour;

impl<I: IntNumber> IntDelaunay<I> {
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
    pub fn centroid_net(&self, min_area: I::WideUInt) -> Vec<IntContour<I>> {
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

                let mut contour = IntContour::<I>::with_capacity(16);
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

trait SafeAdd<I: IntNumber> {
    fn add_area_check(&mut self, contour: IntContour<I>, two_area: I::WideUInt);
}

impl<I: IntNumber> SafeAdd<I> for Vec<IntContour<I>> {
    fn add_area_check(&mut self, contour: IntContour<I>, two_area: I::WideUInt) {
        if two_area == I::WideUInt::ZERO || contour.area_two().unsigned_abs() > two_area {
            self.push(contour);
        }
    }
}

impl<I: IntNumber> IntTriangle<I> {
    #[inline]
    fn right_neighbor_and_mid_edge(&self, vertex_index: usize) -> (usize, IntPoint<I>) {
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
    fn left_neighbor_and_mid_edge(&self, vertex_index: usize) -> (usize, IntPoint<I>) {
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
    fn center(&self) -> IntPoint<I> {
        let a = self.vertices[0].point;
        let b = self.vertices[1].point;
        let c = self.vertices[2].point;

        let x = a.x.to_wide() + b.x.to_wide() + c.x.to_wide();
        let y = a.y.to_wide() + b.y.to_wide() + c.y.to_wide();

        IntPoint::new(
            I::from_wide(x / I::Wide::from_usize(3)),
            I::from_wide(y / I::Wide::from_usize(3)),
        )
    }
}

#[inline]
fn middle<I: IntNumber>(a: IntPoint<I>, b: IntPoint<I>) -> IntPoint<I> {
    let x = a.x.to_wide() + b.x.to_wide();
    let y = a.y.to_wide() + b.y.to_wide();
    IntPoint::new(
        I::from_wide(x / I::Wide::TWO),
        I::from_wide(y / I::Wide::TWO),
    )
}

#[cfg(test)]
mod tests {
    use crate::int::triangulatable::IntTriangulatable;
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn test_0() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let centroids = contour
            .triangulate_with_steiner_points(&[IntPoint::new(5, 5)])
            .into_delaunay()
            .centroid_net(0u64);
        assert_eq!(centroids.len(), 5);
    }
}
