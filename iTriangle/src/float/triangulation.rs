use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::float::adapter::PathToFloat;

/// A triangulation result based on integer computation, with float mapping.
///
/// Internally uses an [`Triangulation`] for performance and robustness,
/// and maps results back to user-provided float types via a [`FloatPointAdapter`].
///
/// # Parameters
/// - `P`: Float point type (e.g., `Vec2`, `[f32; 2]`, etc.)
pub struct RawTriangulation<P: FloatPointCompatible, I: IntNumber = i32> {
    pub raw: RawIntTriangulation<I>,
    pub adapter: FloatPointAdapter<P, I>,
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

impl<P: FloatPointCompatible, I: IntNumber> RawTriangulation<P, I> {
    /// Returns the float-mapped points used in the triangulation.
    ///
    /// The points are guaranteed to match the input shape geometry within adapter precision.
    #[inline]
    pub fn points(&self) -> Vec<P> {
        self.raw.points.to_float(&self.adapter)
    }

    /// Returns the triangle indices for the mesh, ordered counter-clockwise.
    #[inline]
    pub fn triangle_indices<N: IndexType>(&self) -> Vec<N> {
        self.raw.triangle_indices()
    }

    /// Converts this flat triangulation into a flat [`Triangulation`] (points + indices).
    #[inline]
    pub fn to_triangulation<N: IndexType>(&self) -> Triangulation<P, N> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}

impl<P, N: IndexType> Triangulation<P, N> {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(3 * capacity),
        }
    }

    #[inline]
    pub fn set_with_int<I: IntNumber>(
        &mut self,
        triangulation: &IntTriangulation<I, N>,
        adapter: &FloatPointAdapter<P, I>,
    ) where
        P: FloatPointCompatible,
    {
        self.points.clear();
        self.points.reserve(triangulation.points.capacity());
        self.points
            .extend(triangulation.points.iter().map(|p| adapter.int_to_float(p)));

        self.indices.clear();
        self.indices.extend_from_slice(&triangulation.indices);
    }
}

impl<I: IntNumber, N: IndexType> IntTriangulation<I, N> {
    #[inline]
    pub fn into_float<P: FloatPointCompatible>(
        self,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Triangulation<P, N> {
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
    pub fn to_float<P: FloatPointCompatible>(
        &self,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Triangulation<P, N> {
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
    pub fn validate(&self, shape_area: P::Scalar, epsilon: P::Scalar)
    where
        P: FloatPointCompatible,
    {
        let mut s = P::Scalar::from_float(0.0);
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

        s = P::Scalar::from_float(0.5) * s;

        let eps = epsilon * P::Scalar::from_usize(self.indices.len() / 3);
        let delta = (shape_area - s).abs();

        assert!(delta <= eps);
    }

    fn triangle_area_x2(a: &P, b: &P, c: &P) -> P::Scalar
    where
        P: FloatPointCompatible,
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

        let triangulation = Triangulator::<u32>::default().triangulate(&rect);
        assert_eq!(triangulation.points.len(), 4);
        assert_eq!(triangulation.indices.len(), 6);

        triangulation.validate(40.0, 0.000_0001);
    }
}
