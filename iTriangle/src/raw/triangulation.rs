use crate::raw::triangle::PlainTriangle;
use crate::triangulation::int::Triangulation;
use i_overlay::i_float::int::point::IntPoint;

#[derive(Debug)]
pub struct RawTriangulation {
    triangles: Vec<PlainTriangle>,
    points: Vec<IntPoint>,
}

impl RawTriangulation {

    #[inline]
    pub(super) fn new(triangles: Vec<PlainTriangle>, points: Vec<IntPoint>) -> Self {
        Self { triangles, points }
    }

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
    pub fn into_triangulation(self) -> Triangulation {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }
}
