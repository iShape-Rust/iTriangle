use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::float::adapter::PathToFloat;
use i_overlay::i_shape::util::reserve::Reserve;

/// A triangulation result based on integer computation, with float mapping.
///
/// Internally uses an [`Triangulation`] for performance and robustness,
/// and maps results back to user-provided float types via a [`FloatPointAdapter`].
///
/// # Parameters
/// - `P`: Float point type (e.g., `Vec2`, `[f32; 2]`, etc.)
/// - `T`: Float scalar type (e.g., `f32`, `f64`)
pub struct RawTriangulation<P: FloatPointCompatible<T>, T: FloatNumber> {
    pub raw: RawIntTriangulation,
    pub adapter: FloatPointAdapter<P, T>,
}

/// A flat triangulation result consisting of float points and triangle indices.
///
/// Useful for rendering, exporting, or post-processing the mesh in float space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Triangulation<P, I = u16> {
    pub points: Vec<P>,
    pub indices: Vec<I>,
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> RawTriangulation<P, T> {
    /// Returns the float-mapped points used in the triangulation.
    ///
    /// The points are guaranteed to match the input shape geometry within adapter precision.
    #[inline]
    pub fn points(&self) -> Vec<P> {
        self.raw.points.to_float(&self.adapter)
    }

    /// Returns the triangle indices for the mesh, ordered counter-clockwise.
    #[inline]
    pub fn triangle_indices<I: IndexType>(&self) -> Vec<I> {
        self.raw.triangle_indices()
    }

    /// Converts this raw triangulation into a flat [`Triangulation`] (points + indices).
    #[inline]
    pub fn to_triangulation<I: IndexType>(&self) -> Triangulation<P, I> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}

impl<P, I: IndexType> Triangulation<P, I> {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(3 * capacity),
        }
    }

    #[inline]
    pub fn set_with_int<T>(
        &mut self,
        triangulation: &IntTriangulation<I>,
        adapter: &FloatPointAdapter<P, T>,
    ) where
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        self.points.clear();
        self.points
            .reserve_capacity(triangulation.points.capacity());
        self.points
            .extend(triangulation.points.iter().map(|p| adapter.int_to_float(p)));

        self.indices.clear();
        self.indices.extend_from_slice(&triangulation.indices);
    }
}

impl<I: IndexType> IntTriangulation<I> {
    #[inline]
    pub fn into_float<P: FloatPointCompatible<T>, T: FloatNumber>(
        self,
        adapter: &FloatPointAdapter<P, T>,
    ) -> Triangulation<P, I> {
        let points = self
            .points
            .iter()
            .map(|p| adapter.int_to_float(p))
            .collect();
        Triangulation {
            points,
            indices: self.indices,
        }
    }

    #[inline]
    pub fn to_float<P: FloatPointCompatible<T>, T: FloatNumber>(
        &self,
        adapter: &FloatPointAdapter<P, T>,
    ) -> Triangulation<P, I> {
        let points = self
            .points
            .iter()
            .map(|p| adapter.int_to_float(p))
            .collect();
        Triangulation {
            points,
            indices: self.indices.clone(),
        }
    }
}

impl<P, I: IndexType> Triangulation<P, I> {

    pub fn validate<T: FloatNumber>(&self, shape_area: T, epsilon: T)
    where
        P: FloatPointCompatible<T>,
    {
        let mut s = T::from_float(0.0);
        let mut i = 0;
        let neg_eps = -epsilon;
        while i < self.indices.len() {
            let ai = self.indices[i];
            i += 1;
            let bi = self.indices[i];
            i += 1;
            let ci = self.indices[i];
            i += 1;

            let a = &self.points[ai.into_usize()];
            let b = &self.points[bi.into_usize()];
            let c = &self.points[ci.into_usize()];

            let abc = Self::triangle_area_x2(a, b, c);

            // check points direction by its area.
            // Since it's a float point operation in degenerate case it can be near 0 value
            assert!(abc > neg_eps);

            s = s + abc;
        }

        s = T::from_float(0.5) * s;

        let eps = epsilon * T::from_usize(self.indices.len() / 3);
        let delta = (shape_area - s).abs();

        assert!(delta <= eps);
    }

    fn triangle_area_x2<T: FloatNumber>(a: &P, b: &P, c: &P) -> T
    where
        P: FloatPointCompatible<T>,
    {
        let ax = a.x();
        let ay = a.y();
        let bx = b.x();
        let by = b.y();
        let cx = c.x();
        let cy = c.y();

        let v0x = ax - bx;
        let v0y = ay - by;
        let v1x = ax - cx;
        let v1y = ay - cy;

        v0x * v1y - v0y * v1x
    }
}

#[cfg(test)]
mod tests {
    use crate::float::triangulator::Triangulator;

    #[test]
    fn test_0() {
        let rect = [[0.0, 0.0], [5.0, 0.0], [5.0, 8.0], [0.0, 8.0]];

        let triangulation = Triangulator::<u32>::default().triangulate(&rect, false);
        assert_eq!(triangulation.points.len(), 4);
        assert_eq!(triangulation.indices.len(), 6);

        triangulation.validate(40.0, 0.000_0001);
    }
}
