use alloc::vec;
use alloc::vec::Vec;
use i_overlay::i_shape::int::IntPoint;

use crate::{
    int::triangulation::{IndexType, IntTriangulation},
    location::{PointLocationInTriangulation, TriangleIndex},
};

pub trait IntPointInTriangulationLocator {
    fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation>;
}

impl<I> IntPointInTriangulationLocator for I
where
    I: Iterator<Item = [IntPoint; 3]> + Clone,
{
    fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation> {
        locate_points_in_triangles(self.clone(), points)
    }
}

fn locate_points_in_triangles(
    triangles: impl Iterator<Item = [IntPoint; 3]>,
    points: &[IntPoint],
) -> Vec<PointLocationInTriangulation> {
    let mut result = vec![PointLocationInTriangulation::Outside; points.len()];

    for (index, triangle) in triangles.enumerate() {
        let triangle_index = TriangleIndex::new(index);

        for (point_index, &point) in points.iter().enumerate() {
            match triangle.locate_point(point) {
                PointLocationInTriangle::Outside => {}
                PointLocationInTriangle::Inside => match &result[point_index] {
                    PointLocationInTriangulation::Outside => {
                        result[point_index] =
                            PointLocationInTriangulation::InsideTriangle(triangle_index);
                    }
                    // Shouldn't happen.
                    _ => {
                        panic!("Expected outside triangle");
                    }
                },
                PointLocationInTriangle::OnEdge => match &result[point_index] {
                    PointLocationInTriangulation::Outside => {
                        result[point_index] =
                            PointLocationInTriangulation::OnExteriorEdge(triangle_index);
                    }
                    PointLocationInTriangulation::OnExteriorEdge(i) => {
                        result[point_index] =
                            PointLocationInTriangulation::OnInteriorEdge(*i, triangle_index);
                    }
                    // Shouldn't happen.
                    _ => {
                        panic!("More than 2 triangles for one edge");
                    }
                },
                PointLocationInTriangle::OnVertex => match &mut result[point_index] {
                    PointLocationInTriangulation::Outside => {
                        result[point_index] =
                            PointLocationInTriangulation::OnVertex(vec![triangle_index]);
                    }
                    PointLocationInTriangulation::OnVertex(hits) => {
                        hits.push(triangle_index);
                    }
                    // Shouldn't happen.
                    _ => {
                        panic!("Point must be only on Vertex");
                    }
                },
            }
        }
    }

    result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PointLocationInTriangle {
    Outside,
    Inside,
    OnEdge,
    OnVertex,
}

trait IntPointInTriangleLocator {
    fn locate_point(&self, point: IntPoint) -> PointLocationInTriangle;
}

impl IntPointInTriangleLocator for [IntPoint; 3] {
    #[inline]
    fn locate_point(&self, point: IntPoint) -> PointLocationInTriangle {
        let [p0, p1, p2] = *self;

        if point == p0 || point == p1 || point == p2 {
            return PointLocationInTriangle::OnVertex;
        }

        let px = point.x as i64;
        let py = point.y as i64;
        let x0 = p0.x as i64;
        let y0 = p0.y as i64;
        let x1 = p1.x as i64;
        let y1 = p1.y as i64;
        let x2 = p2.x as i64;
        let y2 = p2.y as i64;

        let q0 = (px - x1) * (y0 - y1) - (py - y1) * (x0 - x1);
        let q1 = (px - x2) * (y1 - y2) - (py - y2) * (x1 - x2);
        let q2 = (px - x0) * (y2 - y0) - (py - y0) * (x2 - x0);

        let has_neg = q0 < 0 || q1 < 0 || q2 < 0;
        let has_pos = q0 > 0 || q1 > 0 || q2 > 0;

        if has_neg && has_pos {
            PointLocationInTriangle::Outside
        } else if q0 == 0 || q1 == 0 || q2 == 0 {
            PointLocationInTriangle::OnEdge
        } else {
            PointLocationInTriangle::Inside
        }
    }
}

impl<I: IndexType> IntTriangulation<I> {
    #[inline]
    pub fn locate_points(&self, points: &[IntPoint]) -> Vec<PointLocationInTriangulation> {
        locate_points_in_triangles(self.triangles(), points)
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
