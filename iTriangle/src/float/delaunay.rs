use crate::advanced::buffer::DelaunayBuffer;
use crate::advanced::delaunay::IntDelaunay;
use crate::float::triangulation::{RawTriangulation, Triangulation};
use crate::int::triangulation::IndexType;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::float::adapter::PathToFloat;

/// A Delaunay-refined triangle mesh with float-mapped geometry.
///
/// Produced from [`RawTriangulation::into_delaunay`] by applying edge flips
/// to satisfy the Delaunay condition.
pub struct Delaunay<P: FloatPointCompatible, I: IntNumber = i32> {
    pub(super) delaunay: IntDelaunay<I>,
    pub(super) adapter: FloatPointAdapter<P, I>,
}

impl<P: FloatPointCompatible, I: IntNumber> Default for Delaunay<P, I> {
    #[inline]
    fn default() -> Self {
        Self {
            delaunay: IntDelaunay {
                points: Vec::new(),
                triangles: Vec::new(),
            },
            adapter: FloatPointAdapter::new_conservative(FloatRect::zero()),
        }
    }
}

impl<P: FloatPointCompatible, I: IntNumber> RawTriangulation<P, I> {
    #[inline]
    pub fn into_delaunay(self) -> Delaunay<P, I> {
        let mut buffer = DelaunayBuffer::new();
        self.into_delaunay_with_buffer(&mut buffer)
    }

    #[inline]
    pub fn into_delaunay_with_buffer(self, buffer: &mut DelaunayBuffer) -> Delaunay<P, I> {
        Delaunay {
            delaunay: self.raw.into_delaunay_with_buffer(buffer),
            adapter: self.adapter,
        }
    }
}

impl<P: FloatPointCompatible, I: IntNumber> Delaunay<P, I> {
    /// Returns the float-mapped vertex positions in the triangulation.
    #[inline]
    pub fn points(&self) -> Vec<P> {
        self.delaunay.points.to_float(&self.adapter)
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

    /// Converts this refined mesh into a flat float [`Triangulation`].
    #[inline]
    pub fn to_triangulation<N: IndexType>(&self) -> Triangulation<P, N> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}
