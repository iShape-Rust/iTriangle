use alloc::vec;
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_key_sort::sort::two_keys::TwoKeysSort;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::int::IntPoint;

use crate::{
    int::triangulation::{IndexType, IntTriangulation},
    location::{PointLocationInTriangulation, TriangleIndex},
};

pub trait IntPointInTriangulationLocator<I: IntNumber> {
    fn locate_points(&self, points: &[IntPoint<I>]) -> Vec<PointLocationInTriangulation>;
}

impl<I, T> IntPointInTriangulationLocator<I> for T
where
    I: IntNumber + SortKey,
    T: Iterator<Item = [IntPoint<I>; 3]> + Clone,
{
    fn locate_points(&self, points: &[IntPoint<I>]) -> Vec<PointLocationInTriangulation> {
        locate_points_in_triangles(self.clone(), points)
    }
}

fn locate_points_in_triangles<I: IntNumber + SortKey>(
    triangles: impl Iterator<Item = [IntPoint<I>; 3]>,
    points: &[IntPoint<I>],
) -> Vec<PointLocationInTriangulation> {
    let mut result = vec![PointLocationInTriangulation::Outside; points.len()];
    let mut sorted_points: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(index, &point)| IndexedPoint { index, point })
        .collect();
    sorted_points.sort_by_two_keys(false, |p| p.point.x, |p| p.point.y);

    for (index, triangle) in triangles.enumerate() {
        let triangle_index = TriangleIndex::new(index);
        let rect = triangle.boundary();
        let min = IntPoint::new(rect.min_x, rect.min_y);
        let max = IntPoint::new(rect.max_x, rect.max_y);
        let start = sorted_points.partition_point(|p| p.point < min);

        for &IndexedPoint {
            index: point_index,
            point,
        } in sorted_points[start..].iter().take_while(|p| p.point <= max)
        {
            if point.y < rect.min_y || point.y > rect.max_y {
                continue;
            }

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

#[derive(Clone, Copy)]
struct IndexedPoint<I: IntNumber> {
    index: usize,
    point: IntPoint<I>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PointLocationInTriangle {
    Outside,
    Inside,
    OnEdge,
    OnVertex,
}

trait IntPointInTriangleLocator {
    type Int: IntNumber;
    fn locate_point(&self, point: IntPoint<Self::Int>) -> PointLocationInTriangle;

    fn boundary(&self) -> IntRect<Self::Int>;
}

impl<I: IntNumber> IntPointInTriangleLocator for [IntPoint<I>; 3] {
    type Int = I;

    #[inline]
    fn locate_point(&self, point: IntPoint<I>) -> PointLocationInTriangle {
        let [p0, p1, p2] = *self;

        if point == p0 || point == p1 || point == p2 {
            return PointLocationInTriangle::OnVertex;
        }

        let q0 = (point - p1).cross_product(p0 - p1);
        let q1 = (point - p2).cross_product(p1 - p2);
        let q2 = (point - p0).cross_product(p2 - p0);

        let has_neg = q0 < I::Wide::ZERO || q1 < I::Wide::ZERO || q2 < I::Wide::ZERO;
        let has_pos = q0 > I::Wide::ZERO || q1 > I::Wide::ZERO || q2 > I::Wide::ZERO;

        if has_neg && has_pos {
            PointLocationInTriangle::Outside
        } else if q0 == I::Wide::ZERO || q1 == I::Wide::ZERO || q2 == I::Wide::ZERO {
            PointLocationInTriangle::OnEdge
        } else {
            PointLocationInTriangle::Inside
        }
    }

    #[inline]
    fn boundary(&self) -> IntRect<I> {
        let mut rect = IntRect::with_point(self[0]);
        rect.unsafe_add_point(&self[1]);
        rect.unsafe_add_point(&self[2]);
        rect
    }
}

impl<I: IntNumber + SortKey, N: IndexType> IntTriangulation<I, N> {
    #[inline]
    pub fn locate_points(&self, points: &[IntPoint<I>]) -> Vec<PointLocationInTriangulation> {
        locate_points_in_triangles(self.triangles(), points)
    }
}

impl<I: IntNumber + SortKey, N: IndexType> IntPointInTriangulationLocator<I>
    for IntTriangulation<I, N>
{
    #[inline]
    fn locate_points(&self, points: &[IntPoint<I>]) -> Vec<PointLocationInTriangulation> {
        IntTriangulation::locate_points(self, points)
    }
}

#[cfg(test)]
mod tests {
    use crate::int::triangulator::IntTriangulator;
    use crate::int::validation::Validation;
    use crate::{int::triangulation::IntTriangulation, location::PointLocationInTriangulation};
    use alloc::vec;
    use alloc::vec::Vec;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::i_shape::int::IntPoint;
    use i_overlay::i_shape::int_path;

    fn square_triangulation() -> IntTriangulation<i32, u16> {
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

        locations[0].assert_inside(0);
        locations[1].assert_inside(1);
        locations[2].assert_on_edge(&[0, 1]);
        locations[3].assert_on_edge(&[0]);
        locations[4].assert_on_vertex(&[0, 1]);
        assert_eq!(locations[5], PointLocationInTriangulation::Outside);
    }

    #[test]
    fn test_two_stacked_squares() {
        let path = int_path![[0, 8], [0, 4], [0, 0], [4, 0], [4, 4], [4, 8]];
        let validation = Validation {
            fill_rule: Default::default(),
            options: IntOverlayOptions::keep_all_points(),
        };
        let mut triangulator = IntTriangulator::<i32, u16>::new(32, validation, Default::default());
        triangulator.delaunay = true;
        let triangulation = triangulator.triangulate_contour(&path);

        let points_on_vertex = int_path![[0, 8], [0, 4], [0, 0], [4, 0], [4, 4], [4, 8]];
        let locations_on_vertex = triangulation.locate_points(&points_on_vertex);

        locations_on_vertex[0].assert_on_vertex(&[2, 3]);
        locations_on_vertex[1].assert_on_vertex(&[0, 1, 2]);
        locations_on_vertex[2].assert_on_vertex(&[0]);
        locations_on_vertex[3].assert_on_vertex(&[0, 1]);
        locations_on_vertex[4].assert_on_vertex(&[1, 2, 3]);
        locations_on_vertex[5].assert_on_vertex(&[3]);

        let points_on_edge = int_path![
            [0, 2],
            [0, 6],
            [2, 0],
            [2, 2],
            [2, 4],
            [2, 6],
            [2, 8],
            [4, 2],
            [4, 6]
        ];
        let locations_on_edge = triangulation.locate_points(&points_on_edge);

        locations_on_edge[0].assert_on_edge(&[0]);
        locations_on_edge[1].assert_on_edge(&[2]);
        locations_on_edge[2].assert_on_edge(&[0]);
        locations_on_edge[3].assert_on_edge(&[0, 1]);
        locations_on_edge[4].assert_on_edge(&[1, 2]);
        locations_on_edge[5].assert_on_edge(&[2, 3]);
        locations_on_edge[6].assert_on_edge(&[3]);
        locations_on_edge[7].assert_on_edge(&[1]);
        locations_on_edge[8].assert_on_edge(&[3]);

        let points_inside = int_path![[1, 1], [3, 3], [1, 5], [3, 7]];
        let locations_inside = triangulation.locate_points(&points_inside);

        locations_inside[0].assert_inside(0);
        locations_inside[1].assert_inside(1);
        locations_inside[2].assert_inside(2);
        locations_inside[3].assert_inside(3);

        let mut points_outside = Vec::new();
        for x in -10..=10 {
            for y in -10..=10 {
                if !(0..=4).contains(&x) && !(0..=8).contains(&y) {
                    points_outside.push(IntPoint::new(x, y));
                }
            }
        }

        let locations_inside = triangulation.locate_points(&points_outside);

        for location in locations_inside {
            assert_eq!(location, PointLocationInTriangulation::Outside);
        }
    }

    impl PointLocationInTriangulation {
        fn assert_on_vertex(&self, triangles: &[usize]) {
            if let PointLocationInTriangulation::OnVertex(vec) = self {
                let mut vertex_triangles: Vec<_> = vec.iter().map(|e| e.index()).collect();
                vertex_triangles.sort();

                let mut template_triangles = triangles.to_vec();
                template_triangles.sort();

                assert_eq!(vertex_triangles, template_triangles);
            } else {
                panic!("not on Vertex");
            }
        }

        fn assert_on_edge(&self, triangles: &[usize]) {
            match self {
                PointLocationInTriangulation::OnExteriorEdge(triangle) => {
                    assert_eq!(triangles[0], triangle.index())
                }
                PointLocationInTriangulation::OnInteriorEdge(t0, t1) => {
                    assert!(
                        t0.index() == triangles[0] && t1.index() == triangles[1]
                            || t0.index() == triangles[1] && t1.index() == triangles[0]
                    );
                }
                _ => {
                    panic!("not on Edge");
                }
            }
        }

        fn assert_inside(&self, index: usize) {
            if let PointLocationInTriangulation::InsideTriangle(triangle) = self {
                assert_eq!(triangle.index(), index);
            } else {
                panic!("not Inside");
            }
        }
    }
}
