use crate::advanced::buffer::DelaunayBuffer;
use crate::advanced::delaunay::IntDelaunay;
use crate::generic::adapter::PointAdapter;
use crate::generic::triangulation::{RawTriangulation, Triangulation};
use crate::int::triangulation::IndexType;
use alloc::vec::Vec;

/// A Delaunay-refined triangle mesh with adapter-mapped geometry.
///
/// Produced from [`RawTriangulation::into_delaunay`] by applying edge flips
/// to satisfy the Delaunay condition.
pub struct Delaunay<A: PointAdapter> {
    pub(super) delaunay: IntDelaunay<A::Int>,
    pub(super) adapter: A,
}

impl<A: PointAdapter> RawTriangulation<A> {
    #[inline]
    pub fn into_delaunay(self) -> Delaunay<A> {
        let mut buffer = DelaunayBuffer::new();
        self.into_delaunay_with_buffer(&mut buffer)
    }

    #[inline]
    pub fn into_delaunay_with_buffer(self, buffer: &mut DelaunayBuffer) -> Delaunay<A> {
        Delaunay {
            delaunay: self.raw.into_delaunay_with_buffer(buffer),
            adapter: self.adapter,
        }
    }
}

impl<A: PointAdapter> Delaunay<A> {
    /// Returns the adapter-mapped vertex positions in the triangulation.
    #[inline]
    pub fn points(&self) -> Vec<A::Point> {
        self.adapter.points_from_int(&self.delaunay.points)
    }

    /// Returns indices forming counter-clockwise triangles.
    #[inline]
    pub fn triangle_indices<N: IndexType>(&self) -> Vec<N> {
        self.delaunay.triangle_indices()
    }

    /// Returns the indices of each triangle's neighboring triangles.
    #[inline]
    pub fn triangle_neighbors(&self) -> Vec<[usize; 3]> {
        self.delaunay.triangle_neighbors()
    }

    /// Converts this refined mesh into a flat [`Triangulation`].
    #[inline]
    pub fn to_triangulation<N: IndexType>(&self) -> Triangulation<A::Point, N> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}
