use crate::advanced::delaunay::IntDelaunay;
use crate::int::triangulation::{IndexType, IntTriangulation};
use alloc::vec::Vec;
use i_overlay::i_float::int::point::IntPoint;

impl IntDelaunay {
    /// Returns the vertex positions in the triangulation.
    #[inline]
    pub fn points(&self) -> &Vec<IntPoint> {
        &self.points
    }

    /// Returns indices forming counter-clockwise triangles.
    #[inline]
    pub fn triangle_indices<I: IndexType>(&self) -> Vec<I> {
        let mut result = Vec::with_capacity(3 * self.triangles.len());
        for t in &self.triangles {
            let v = &t.vertices;
            let i0 = I::try_from(v[0].index).unwrap_or(I::ZERO);
            let i1 = I::try_from(v[1].index).unwrap_or(I::ZERO);
            let i2 = I::try_from(v[2].index).unwrap_or(I::ZERO);

            result.extend_from_slice(&[i0, i1, i2]);
        }
        result
    }

    /// Returns the indices of each triangle's neighboring triangles.
    #[inline]
    pub fn triangle_neighbors(&self) -> Vec<[usize; 3]> {
        self.triangles
            .iter()
            .map(|triangle| triangle.neighbors)
            .collect()
    }

    #[inline]
    pub fn into_triangulation<I: IndexType>(self) -> IntTriangulation<I> {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }
}
