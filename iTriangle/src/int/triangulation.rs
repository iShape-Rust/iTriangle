use crate::advanced::delaunay::IntDelaunay;
use crate::geom::triangle::IntTriangle;
use alloc::vec::Vec;
use core::iter::FusedIterator;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

pub trait IndexType: Copy + Clone + TryFrom<usize> + Default {
    const MAX: usize;
    const ZERO: Self;
    fn add(self, other: Self) -> Self;
    fn into_usize(self) -> usize;
}

impl IndexType for u8 {
    const MAX: usize = u8::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }
    #[inline]
    fn into_usize(self) -> usize {
        self as usize
    }
}
impl IndexType for u16 {
    const MAX: usize = u16::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }
    #[inline]
    fn into_usize(self) -> usize {
        self as usize
    }
}
impl IndexType for u32 {
    const MAX: usize = u32::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }
    #[inline]
    fn into_usize(self) -> usize {
        self as usize
    }
}
impl IndexType for u64 {
    const MAX: usize = u64::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }
    #[inline]
    fn into_usize(self) -> usize {
        self as usize
    }
}
impl IndexType for usize {
    const MAX: usize = usize::MAX;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }
    #[inline]
    fn into_usize(self) -> usize {
        self
    }
}

#[derive(Debug, Clone)]
pub struct IntTriangulation<I: IntNumber, N = u16> {
    pub points: Vec<IntPoint<I>>,
    pub indices: Vec<N>,
}

impl<I: IntNumber, N> Default for IntTriangulation<I, N> {
    #[inline]
    fn default() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Iterator over resolved triangles in a flat [`IntTriangulation`].
///
/// Each item contains the three triangle points addressed by one consecutive
/// triple in the triangulation index buffer.
#[derive(Clone)]
pub struct IntTriangleIterator<'a, I: IntNumber, N> {
    points: &'a [IntPoint<I>],
    indices: core::slice::ChunksExact<'a, N>,
}

impl<I: IntNumber, N: IndexType> Iterator for IntTriangleIterator<'_, I, N> {
    type Item = [IntPoint<I>; 3];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let indices = self.indices.next()?;
        let a = self.points[indices[0].into_usize()];
        let b = self.points[indices[1].into_usize()];
        let c = self.points[indices[2].into_usize()];
        Some([a, b, c])
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<I: IntNumber, N: IndexType> ExactSizeIterator for IntTriangleIterator<'_, I, N> {
    #[inline]
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl<I: IntNumber, N: IndexType> FusedIterator for IntTriangleIterator<'_, I, N> {}

/// A int triangle mesh produced by the triangulation process.
///
/// This is the low-level output containing full triangle and vertex data,
/// including adjacency and vertex indices. It can be converted into a higher-level
/// `Triangulation` (index buffer + point list) using [`into_triangulation`].
///
/// Use this when you need detailed control over topology, neighbor tracking, or
/// advanced mesh manipulation.
#[derive(Debug)]
pub struct RawIntTriangulation<I: IntNumber> {
    pub(crate) triangles: Vec<IntTriangle<I>>,
    pub(crate) points: Vec<IntPoint<I>>,
}

impl<I: IntNumber> Default for RawIntTriangulation<I> {
    #[inline]
    fn default() -> Self {
        Self {
            triangles: Vec::new(),
            points: Vec::new(),
        }
    }
}

impl<I: IntNumber> RawIntTriangulation<I> {
    #[inline]
    pub(super) fn new(triangles: Vec<IntTriangle<I>>, points: Vec<IntPoint<I>>) -> Self {
        Self { triangles, points }
    }

    /// Returns true if the triangulation contains no triangles.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Returns a reference to the list of points used in the triangulation.
    ///
    /// Each point corresponds to a coordinate used by one or more triangles.
    #[inline]
    pub fn points(&self) -> &Vec<IntPoint<I>> {
        &self.points
    }

    /// Returns a flat list of triangle vertex indices (ABC ordering).
    ///
    /// Each triangle contributes 3 indices into the `points` buffer.
    #[inline]
    pub fn triangle_indices<N: IndexType>(&self) -> Vec<N> {
        let mut indices = Vec::new();
        self.triangles.feed_indices(self.points.len(), &mut indices);
        indices
    }

    /// Returns the indices of each triangle's neighboring triangles.
    #[inline]
    pub fn triangle_neighbors(&self) -> Vec<[usize; 3]> {
        self.triangles
            .iter()
            .map(|triangle| triangle.neighbors)
            .collect()
    }

    /// Converts the int triangulation into a simpler index-based mesh.
    ///
    /// Returns a [`IntTriangulation`] with separate index buffer and point list.
    #[inline]
    pub fn into_triangulation<N: IndexType>(self) -> IntTriangulation<I, N> {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }

    /// Converts the int triangulation into a simpler index-based mesh.
    ///
    /// Returns a [`IntTriangulation`] with separate index buffer and point list.
    #[inline]
    pub fn to_triangulation<N: IndexType>(&self) -> IntTriangulation<I, N> {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points.as_slice().to_vec(),
        }
    }

    #[inline]
    pub(crate) fn shift(&mut self, points_offset: usize, triangle_offset: usize) {
        for t in self.triangles.iter_mut() {
            t.vertices[0].index += points_offset;
            t.vertices[1].index += points_offset;
            t.vertices[2].index += points_offset;
            t.neighbors[0] = t.neighbors[0].saturating_add(triangle_offset);
            t.neighbors[1] = t.neighbors[1].saturating_add(triangle_offset);
            t.neighbors[2] = t.neighbors[2].saturating_add(triangle_offset);
        }
    }
}
impl<I: IntNumber, N: IndexType> IntTriangulation<I, N> {
    #[inline]
    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(3 * capacity),
        }
    }

    #[inline]
    pub fn join(&mut self, other: &Self) {
        let points_offset = N::try_from(self.points.len()).unwrap_or(N::ZERO);
        for &i in other.indices.iter() {
            self.indices.push(i.add(points_offset));
        }
        self.points.extend_from_slice(&other.points)
    }

    /// Iterates over resolved triangle points.
    ///
    /// The iterator walks `indices` in exact triples and yields the matching
    /// `[IntPoint; 3]` for each triangle.
    #[inline]
    pub fn triangles(&self) -> IntTriangleIterator<'_, I, N> {
        IntTriangleIterator {
            points: &self.points,
            indices: self.indices.chunks_exact(3),
        }
    }

    #[inline]
    pub fn reserve_and_clear(&mut self, new_len: usize) {
        self.points.clear();
        self.points.reserve(new_len);
        self.indices.clear();
        self.indices.reserve(3 * new_len);
    }

    #[inline]
    pub fn fill_with_raw(&mut self, triangulation: &RawIntTriangulation<I>) {
        self.points.clear();
        self.points.extend_from_slice(&triangulation.points);

        triangulation
            .triangles
            .feed_indices(triangulation.points.len(), &mut self.indices);
    }

    #[inline]
    pub fn fill_with_delaunay(&mut self, delaunay: &IntDelaunay<I>) {
        self.points.clear();
        self.points.extend_from_slice(&delaunay.points);

        delaunay
            .triangles
            .feed_indices(delaunay.points.len(), &mut self.indices);
    }
}

#[cfg(test)]
mod tests {
    use super::IntTriangulation;
    use alloc::{vec, vec::Vec};
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn triangles_iterates_resolved_points() {
        let triangulation = IntTriangulation {
            points: vec![
                IntPoint::new(0, 0),
                IntPoint::new(10, 0),
                IntPoint::new(10, 10),
                IntPoint::new(0, 10),
            ],
            indices: vec![0_u16, 1, 2, 0, 2, 3],
        };

        let triangles: Vec<_> = triangulation.triangles().collect();

        assert_eq!(
            triangles,
            vec![
                [
                    IntPoint::new(0, 0),
                    IntPoint::new(10, 0),
                    IntPoint::new(10, 10),
                ],
                [
                    IntPoint::new(0, 0),
                    IntPoint::new(10, 10),
                    IntPoint::new(0, 10),
                ],
            ]
        );
    }
}

pub(crate) trait IndicesBuilder {
    fn feed_indices<N: IndexType>(&self, max_count: usize, indices: &mut Vec<N>);
}

impl<I: IntNumber> IndicesBuilder for [IntTriangle<I>] {
    #[inline]
    fn feed_indices<N: IndexType>(&self, max_count: usize, indices: &mut Vec<N>) {
        if max_count > N::MAX {
            panic!(
                "Index type `{}` cannot hold {} points",
                core::any::type_name::<N>(),
                max_count
            );
        }

        let count = 3 * self.len();
        indices.clear();
        indices.reserve(count);

        for t in self.iter() {
            let i0 = unsafe { N::try_from(t.vertices[0].index).unwrap_unchecked() };
            let i1 = unsafe { N::try_from(t.vertices[1].index).unwrap_unchecked() };
            let i2 = unsafe { N::try_from(t.vertices[2].index).unwrap_unchecked() };
            indices.push(i0);
            indices.push(i1);
            indices.push(i2);
        }
    }
}

impl<I: IntNumber> RawIntTriangulation<I> {
    pub fn validate(&self) {
        for (i, t) in self.triangles.iter().enumerate() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;
            let area = Triangle::area_two(a, b, c);
            assert!(area >= I::Wide::ZERO);

            let n0 = t.neighbors[0];
            let n1 = t.neighbors[1];
            let n2 = t.neighbors[2];

            if n0 != usize::MAX {
                assert!(self.triangles[n0].neighbors.contains(&i));
            }
            if n1 != usize::MAX {
                assert!(self.triangles[n1].neighbors.contains(&i));
            }
            if n2 != usize::MAX {
                assert!(self.triangles[n2].neighbors.contains(&i));
            }
        }
    }

    pub fn area_two(&self) -> I::Wide {
        let mut s = I::Wide::ZERO;
        for t in self.triangles.iter() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;

            s = s + Triangle::area_two(a, b, c);
        }
        s
    }
}

#[cfg(test)]
impl<I: IntNumber, N: IndexType> IntTriangulation<I, N> {
    pub fn validate(&self, shape_x2_area: I::Wide) {
        let mut s = I::Wide::ZERO;
        let mut i = 0;
        while i < self.indices.len() {
            let ai = self.indices[i];
            i += 1;
            let bi = self.indices[i];
            i += 1;
            let ci = self.indices[i];
            i += 1;

            let a = self.points[ai.into_usize()];
            let b = self.points[bi.into_usize()];
            let c = self.points[ci.into_usize()];

            let abc = Triangle::area_two(a, b, c);

            assert!(abc > I::Wide::ZERO);

            s = s + abc;
        }

        assert!(s == shape_x2_area);
    }
}
