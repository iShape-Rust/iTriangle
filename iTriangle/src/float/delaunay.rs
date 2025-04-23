use crate::advanced::delaunay::IntDelaunay;
use crate::float::triangulation::{RawTriangulation, Triangulation};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::float::adapter::PathToFloat;

/// A Delaunay-refined triangle mesh with float-mapped geometry.
///
/// Produced from [`RawTriangulation::into_delaunay`] by applying edge flips
/// to satisfy the Delaunay condition.
pub struct Delaunay<P: FloatPointCompatible<T>, T: FloatNumber> {
    pub(super) delaunay: IntDelaunay,
    pub(super) adapter: FloatPointAdapter<P, T>,
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> RawTriangulation<P, T> {
    #[inline]
    pub fn into_delaunay(self) -> Delaunay<P, T> {
        Delaunay {
            delaunay: self.raw.into_delaunay(),
            adapter: self.adapter,
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> Delaunay<P, T> {
    /// Returns the float-mapped vertex positions in the triangulation.
    #[inline]
    pub fn points(&self) -> Vec<P> {
        self.delaunay.points.to_float(&self.adapter)
    }

    /// Returns indices forming counter-clockwise triangles.
    #[inline]
    pub fn triangle_indices(&self) -> Vec<usize> {
        self.delaunay.triangle_indices()
    }

    /// Converts this refined mesh into a flat float [`Triangulation`].
    #[inline]
    pub fn to_triangulation(&self) -> Triangulation<P> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}
