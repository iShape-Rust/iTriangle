use crate::geom::triangle::ABCTriangle;
use crate::triangulation::int::Triangulation;
use i_overlay::i_float::int::point::IntPoint;
use crate::fit::delaunay::Delaunay;

#[derive(Debug)]
pub struct RawTriangulation {
    pub(crate) triangles: Vec<ABCTriangle>,
    pub(crate) points: Vec<IntPoint>,
}

impl RawTriangulation {

    #[inline]
    pub(super) fn new(triangles: Vec<ABCTriangle>, points: Vec<IntPoint>) -> Self {
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

    #[inline]
    pub fn into_pretty_triangulation(self, max_iter_count: usize) -> Triangulation {
        if max_iter_count == 0 {
            self.into_triangulation()
        } else {
            let mut delaunay = Delaunay {
                triangles: self.triangles,
                points: self.points,
            };
            delaunay.build(max_iter_count);

            delaunay.into_triangulation()
        }
    }
}
