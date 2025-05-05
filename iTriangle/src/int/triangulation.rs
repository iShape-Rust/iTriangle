use crate::geom::triangle::IntTriangle;
use i_overlay::i_float::int::point::IntPoint;

pub trait IndexType: Copy + Clone + TryFrom<usize> {
    const MAX: usize;
    const ZERO: Self;
    fn add(self, other: Self) -> Self;
}

impl IndexType for u8 {
    const MAX: usize = u8::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self { self + other }
}
impl IndexType for u16 {
    const MAX: usize = u16::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self { self + other }
}
impl IndexType for u32 {
    const MAX: usize = u32::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self { self + other }
}
impl IndexType for u64 {
    const MAX: usize = u64::MAX as usize;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self { self + other }
}
impl IndexType for usize {
    const MAX: usize = usize::MAX;
    const ZERO: Self = 0;
    #[inline]
    fn add(self, other: Self) -> Self { self + other }
}

#[derive(Debug, Clone)]
pub struct IntTriangulation<I> {
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
        let points_count = self.points.len();
        if points_count > I::MAX {
            panic!(
                "Index type `{}` cannot hold {} points",
                std::any::type_name::<I>(),
                points_count
            );
        }

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
