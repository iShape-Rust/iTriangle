use alloc::vec::Vec;
use crate::geom::triangle::IntTriangle;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::util::reserve::Reserve;

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

#[derive(Debug, Clone, Default)]
pub struct IntTriangulation<I = u16> {
    pub points: Vec<IntPoint>,
    pub indices: Vec<I>,
}

/// A int triangle mesh produced by the triangulation process.
///
/// This is the low-level output containing full triangle and vertex data,
/// including adjacency and vertex indices. It can be converted into a higher-level
/// `Triangulation` (index buffer + point list) using [`into_triangulation`].
///
/// Use this when you need detailed control over topology, neighbor tracking, or
/// advanced mesh manipulation.
#[derive(Debug, Default)]
pub struct RawIntTriangulation {
    pub(crate) triangles: Vec<IntTriangle>,
    pub(crate) points: Vec<IntPoint>,
}

impl RawIntTriangulation {

    #[inline]
    pub(super) fn new(triangles: Vec<IntTriangle>, points: Vec<IntPoint>) -> Self {
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
    pub fn points(&self) -> &Vec<IntPoint> {
        &self.points
    }

    /// Returns a flat list of triangle vertex indices (ABC ordering).
    ///
    /// Each triangle contributes 3 indices into the `points` buffer.
    ///
    #[inline]
    pub fn triangle_indices<I: IndexType>(&self) -> Vec<I> {
        let mut indices = Vec::new();
        self.triangles.feed_indices(self.points.len(), &mut indices);
        indices
    }

    /// Converts the int triangulation into a simpler index-based mesh.
    ///
    /// Returns a [`IntTriangulation`] with separate index buffer and point list.
    #[inline]
    pub fn into_triangulation<I: IndexType>(self) -> IntTriangulation<I> {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }

    /// Converts the int triangulation into a simpler index-based mesh.
    ///
    /// Returns a [`IntTriangulation`] with separate index buffer and point list.
    #[inline]
    pub fn to_triangulation<I: IndexType>(&self) -> IntTriangulation<I> {
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
impl<I: IndexType> IntTriangulation<I> {

    #[inline]
    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new()
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(3 * capacity)
        }
    }

    #[inline]
    pub fn join(&mut self, other: &Self) {
        let points_offset = I::try_from(self.points.len()).unwrap_or(I::ZERO);
        for &i in other.indices.iter() {
            self.indices.push(i.add(points_offset));
        }
        self.points.extend_from_slice(&other.points)
    }

    #[inline]
    pub fn reserve_and_clear(&mut self, new_len: usize) {
        self.points.reserve_capacity(new_len);
        self.points.clear();
        self.indices.reserve_capacity(3 * new_len);
        self.indices.clear();
    }

    #[inline]
    pub fn fill_with_raw(&mut self, triangulation: &RawIntTriangulation) {
        self.points.clear();
        self.points.extend_from_slice(&triangulation.points);

        triangulation.triangles.feed_indices(triangulation.points.len(), &mut self.indices);
    }
}

pub(crate) trait IndicesBuilder {
    fn feed_indices<I: IndexType>(&self, max_count: usize, indices: &mut Vec<I>);
}

impl IndicesBuilder for [IntTriangle] {

    #[inline]
    fn feed_indices<I: IndexType>(&self, max_count: usize, indices: &mut Vec<I>) {
        if max_count > I::MAX {
            panic!(
                "Index type `{}` cannot hold {} points",
                core::any::type_name::<I>(),
                max_count
            );
        }

        let count = 3 * self.len();
        indices.reserve_capacity(count);
        indices.clear();

        for t in self.iter() {
            let i0 = unsafe { I::try_from(t.vertices[0].index).unwrap_unchecked() };
            let i1 = unsafe { I::try_from(t.vertices[1].index).unwrap_unchecked() };
            let i2 = unsafe { I::try_from(t.vertices[2].index).unwrap_unchecked() };
            indices.push(i0);
            indices.push(i1);
            indices.push(i2);
        }
    }
}

impl RawIntTriangulation {
    pub fn validate(&self) {
        for (i, t) in self.triangles.iter().enumerate() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;
            let area = Triangle::area_two_point(a, b, c);
            assert!(area <= 0);

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

    pub fn area_two(&self) -> i64 {
        let mut s = 0;
        for t in self.triangles.iter() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;

            s += Triangle::area_two_point(a, b, c);
        }
        s
    }
}

#[cfg(test)]
impl<I: IndexType> IntTriangulation<I> {
        pub fn validate(&self, shape_x2_area: i64) {
            let mut s = 0;
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

                let abc = Triangle::area_two_point(a, b, c);

                assert!(abc < 0);

                s = s + abc;
            }

            assert!(s == shape_x2_area);
        }
    }