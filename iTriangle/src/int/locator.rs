use alloc::vec;
use alloc::vec::Vec;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::IntPoint;

use crate::{
    int::triangulation::{IndexType, IntTriangulation},
    location::{PointLocationInTriangulation, TriangleIndex},
};

pub trait IntPointInTriangulationLocator {
    fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation>;
}

impl<I: IndexType> IntTriangulation<I> {
    pub fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation> {
        let mut result = vec![PointLocationInTriangulation::Outside; points.len()];

        for (index, triangle) in self.indices.chunks_exact(3).enumerate() {
            let vertex0 = self.points[triangle[0].into_usize()];
            let vertex1 = self.points[triangle[1].into_usize()];
            let vertex2 = self.points[triangle[2].into_usize()];
            let triangle_index = TriangleIndex::new(index);

            for (point_index, &point) in points.iter().enumerate() {
                if point == vertex0 || point == vertex1 || point == vertex2 {
                    match &mut result[point_index] {
                        PointLocationInTriangulation::Outside => {
                            result[point_index] =
                                PointLocationInTriangulation::OnVertex(vec![triangle_index]);
                        }
                        PointLocationInTriangulation::OnVertex(hits) => {
                            hits.push(triangle_index);
                        }
                        // Shouldn't happen.
                        _ => {}
                    }

                    continue;
                }

                if !Triangle::is_contain_point(point, vertex0, vertex1, vertex2) {
                    continue;
                }

                if Triangle::is_contain_point_exclude_borders(point, vertex0, vertex1, vertex2) {
                    match &result[point_index] {
                        PointLocationInTriangulation::Outside => {
                            result[point_index] =
                                PointLocationInTriangulation::InsideTriangle(triangle_index);
                        }
                        // Shouldn't happen.
                        _ => {}
                    }

                    continue;
                }

                match &result[point_index] {
                    PointLocationInTriangulation::Outside => {
                        result[point_index] =
                            PointLocationInTriangulation::OnExteriorEdge(triangle_index);
                    }
                    PointLocationInTriangulation::OnExteriorEdge(i) => {
                        result[point_index] =
                            PointLocationInTriangulation::OnInteriorEdge(*i, triangle_index);
                    }
                    // Shouldn't happen.
                    _ => {}
                }
            }
        }

        result
    }
}

impl<I: IndexType> IntPointInTriangulationLocator for IntTriangulation<I> {
    #[inline]
    fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation> {
        IntTriangulation::locate_points(self, points)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use i_overlay::i_shape::int::IntPoint;

    use crate::{
        int::triangulation::IntTriangulation,
        location::{PointLocationInTriangulation, TriangleIndex},
    };

    fn square_triangulation() -> IntTriangulation<u16> {
        IntTriangulation {
            points: vec![
                IntPoint::new(0, 0),
                IntPoint::new(4, 0),
                IntPoint::new(4, 4),
                IntPoint::new(0, 4),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    #[test]
    fn test_locate_points() {
        let triangulation = square_triangulation();
        let points_to_locate = vec![
            IntPoint::new(3, 1),
            IntPoint::new(1, 3),
            IntPoint::new(2, 2),
            IntPoint::new(2, 0),
            IntPoint::new(0, 0),
            IntPoint::new(5, 1),
        ];

        let locations = triangulation.locate_points(&points_to_locate);

        assert!(matches!(
            locations[0],
            PointLocationInTriangulation::InsideTriangle(t) if t == TriangleIndex::new(0)
        ));
        assert!(matches!(
            locations[1],
            PointLocationInTriangulation::InsideTriangle(t) if t == TriangleIndex::new(1)
        ));
        assert!(matches!(
            locations[2],
            PointLocationInTriangulation::OnInteriorEdge(a, b)
                if a == TriangleIndex::new(0) && b == TriangleIndex::new(1)
        ));
        assert!(matches!(
            locations[3],
            PointLocationInTriangulation::OnExteriorEdge(t)
                if t == TriangleIndex::new(0)
        ));
        assert!(matches!(
            locations[4].clone(),
            PointLocationInTriangulation::OnVertex(triangles)
                if triangles.as_slice() == [TriangleIndex::new(0), TriangleIndex::new(1)]
        ));
        assert!(matches!(
            locations[5],
            PointLocationInTriangulation::Outside
        ));
    }
}
