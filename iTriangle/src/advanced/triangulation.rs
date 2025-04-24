use i_overlay::i_float::int::point::IntPoint;
use crate::advanced::delaunay::IntDelaunay;
use crate::int::triangulation::IntTriangulation;

impl IntDelaunay {
    #[inline]
    pub fn points(&self) -> &Vec<IntPoint> {
        &self.points
    }

    #[inline]
    pub fn triangle_indices(&self) -> Vec<usize> {
        let mut result = Vec::with_capacity(3 * self.triangles.len());
        for t in &self.triangles {
            let v = &t.vertices;
            result.extend_from_slice(&[v[0].index, v[1].index, v[2].index]);
        }
        result
    }

    #[inline]
    pub fn into_triangulation(self) -> IntTriangulation {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }
}