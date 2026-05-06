use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::{i_float::adapter::FloatPointAdapter, i_shape::float::adapter::PathToInt};

use crate::{
    float::triangulation::Triangulation,
    int::triangulation::{IndexType, IntTriangulation},
    location::PointLocationInTriangulation,
};

pub trait PointInTriangulationLocator<P> {
    fn locate_points<T>(&self, points: &[P]) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
        T: FloatNumber;
}

impl<P, I: IndexType> Triangulation<P, I> {
    pub fn locate_points<T: FloatNumber>(&self, points: &[P]) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
    {
        let adapter = FloatPointAdapter::with_iter(self.points.iter().chain(points.iter()));

        let int_triangulation = IntTriangulation {
            points: self.points.to_int(&adapter),
            indices: self.indices.clone(),
        };

        int_triangulation.locate_points(&points.to_int(&adapter))
    }
}

impl<P, I: IndexType> PointInTriangulationLocator<P> for Triangulation<P, I> {
    #[inline]
    fn locate_points<T>(&self, points: &[P]) -> Vec<PointLocationInTriangulation>
    where
        P: FloatPointCompatible<Scalar = T>,
        T: FloatNumber,
    {
        Triangulation::locate_points::<T>(self, points)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::{
        float::triangulation::Triangulation,
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

        let locations = triangulation.locate_points::<f64>(&points_to_locate);

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
