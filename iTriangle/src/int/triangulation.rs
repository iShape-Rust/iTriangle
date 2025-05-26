use alloc::vec;
use alloc::vec::Vec;
use crate::geom::triangle::IntTriangle;
use i_overlay::i_float::int::point::IntPoint;
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
#[derive(Debug)]
pub struct RawIntTriangulation {
    pub(crate) triangles: Vec<IntTriangle>,
    pub(crate) points: Vec<IntPoint>,
}

impl RawIntTriangulation {
    pub(crate) fn empty() -> Self {
        Self {
            triangles: vec![],
            points: vec![],
        }
    }

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