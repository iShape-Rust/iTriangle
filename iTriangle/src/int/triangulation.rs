use crate::geom::triangle::IntTriangle;
use i_overlay::i_float::int::point::IntPoint;

#[derive(Debug)]
pub struct Triangulation {
    pub points: Vec<IntPoint>,
    pub indices: Vec<usize>,
}

/// A int triangle mesh produced by the triangulation process.
///
/// This is the low-level output containing full triangle and vertex data,
/// including adjacency and vertex indices. It can be converted into a higher-level
/// `Triangulation` (index buffer + point list) using [`into_triangulation`].
///
/// Use this when you need detailed control over topology, neighbor tracking, or
/// advanced mesh manipulation.
#[derive(Debug)]
pub struct IntTriangulation {
    pub(crate) triangles: Vec<IntTriangle>,
    pub(crate) points: Vec<IntPoint>,
}

impl IntTriangulation {

    pub(crate) fn empty() -> Self {
        Self { triangles: vec![], points: vec![] }
    }
    
    #[inline]
    pub(super) fn new(triangles: Vec<IntTriangle>, points: Vec<IntPoint>) -> Self {
        Self { triangles, points }
    }

    /// Returns a reference to the list of points used in the triangulation.
    ///
    /// Each point corresponds to a coordinate used by one or more triangles.
    #[inline]
    pub fn points(&self) -> &Vec<IntPoint> {
        &self.points
    }

    /// Returns a flat list of triangle vertex indices (ABC ordering).
    ///
    /// Each triangle contributes 3 indices into the `points` buffer.
    ///
    #[inline]
    pub fn triangle_indices(&self) -> Vec<usize> {
        let mut result = Vec::with_capacity(3 * self.triangles.len());
        for t in &self.triangles {
            let v = &t.vertices;
            result.extend_from_slice(&[v[0].index, v[1].index, v[2].index]);
        }
        result
    }

    /// Converts the int triangulation into a simpler index-based mesh.
    ///
    /// Returns a [`Triangulation`] with separate index buffer and point list.
    #[inline]
    pub fn into_triangulation(self) -> Triangulation {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }

    #[inline]
    pub(crate) fn shift(&mut self, points_offset: usize, triangle_offset: usize) {
        for t in self.triangles.iter_mut() {
            t.vertices[0].index += points_offset;
            t.vertices[1].index += points_offset;
            t.vertices[2].index += points_offset;
            t.neighbors[0].saturating_add(triangle_offset);
            t.neighbors[1].saturating_add(triangle_offset);
            t.neighbors[2].saturating_add(triangle_offset);
        }
    }
}
