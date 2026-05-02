use crate::advanced::delaunay::IntDelaunay;
use crate::float::triangulation::{RawTriangulation, Triangulation};
use crate::int::triangulation::IndexType;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_shape::float::adapter::PathToFloat;

/// A Delaunay-refined triangle mesh with float-mapped geometry.
///
/// Produced from [`Triangulation::into_delaunay`] by applying edge flips
/// to satisfy the Delaunay condition.
pub struct Delaunay<P: FloatPointCompatible> {
    pub(super) delaunay: IntDelaunay,
    pub(super) adapter: FloatPointAdapter<P>,
}

impl<P: FloatPointCompatible> RawTriangulation<P> {
    #[inline]
    pub fn into_delaunay(self) -> Delaunay<P> {
        Delaunay {
            delaunay: self.raw.into_delaunay(),
            adapter: self.adapter,
        }
    }
}

impl<P: FloatPointCompatible> Delaunay<P> {
    /// Returns the float-mapped vertex positions in the triangulation.
    #[inline]
    pub fn points(&self) -> Vec<P> {
        self.delaunay.points.to_float(&self.adapter)
    }

    /// Returns indices forming counter-clockwise triangles.
    #[inline]
    pub fn triangle_indices<I: IndexType>(&self) -> Vec<I> {
        self.delaunay.triangle_indices()
    }

    /// Converts this refined mesh into a flat float [`Triangulation`].
    #[inline]
    pub fn to_triangulation<I: IndexType>(&self) -> Triangulation<P, I> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}
