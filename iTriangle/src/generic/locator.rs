use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::adapter::FloatPointAdapter;

use crate::generic::adapter::PointAdapter;
use crate::int::locator::IntPointInTriangulationLocator;
use crate::{
    generic::triangulation::Triangulation, int::triangulation::IndexType,
    location::PointLocationInTriangulation,
};

pub trait PointInTriangulationLocator<P, I: IntNumber = i32> {
    fn locate_points<T>(&self, points: &[P]) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
        I: SortKey,
        T: FloatNumber;
}

impl<P, N: IndexType> Triangulation<P, N> {
    pub fn locate_points<T: FloatNumber, I: IntNumber + SortKey>(
        &self,
        points: &[P],
    ) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
    {
        let adapter = FloatPointAdapter::<P, I>::with_iter(self.points.iter().chain(points.iter()));

        let int_points = adapter.points_to_int(points);

        let triangles = self.indices.chunks_exact(3).map(|triangle| {
            let a = adapter.to_int_point(&self.points[triangle[0].into_usize()]);
            let b = adapter.to_int_point(&self.points[triangle[1].into_usize()]);
            let c = adapter.to_int_point(&self.points[triangle[2].into_usize()]);
            [a, b, c]
        });

        triangles.locate_points(&int_points)
    }
}

impl<P, I: IntNumber + SortKey, N: IndexType> PointInTriangulationLocator<P, I>
    for Triangulation<P, N>
{
    #[inline]
    fn locate_points<T>(&self, points: &[P]) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
        T: FloatNumber,
    {
        Triangulation::locate_points::<T, I>(self, points)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::{
        generic::triangulation::Triangulation,
        location::{PointLocationInTriangulation, TriangleIndex},
    };

    fn square_triangulation() -> Triangulation<[f64; 2], u16> {
        Triangulation {
            points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    #[test]
    fn test_locate_points() {
        let triangulation = square_triangulation();
        let points_to_locate = vec![
            [3.0, 1.0],
            [1.0, 3.0],
            [2.0, 2.0],
            [2.0, 0.0],
            [0.0, 0.0],
            [5.0, 1.0],
        ];

        let locations = triangulation.locate_points::<f64, i32>(&points_to_locate);

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
            &locations[4],
            PointLocationInTriangulation::OnVertex(triangles)
                if triangles.as_slice() == [TriangleIndex::new(0), TriangleIndex::new(1)]
        ));
        assert!(matches!(
            locations[5],
            PointLocationInTriangulation::Outside
        ));
    }
}
