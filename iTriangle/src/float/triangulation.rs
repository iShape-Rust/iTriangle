use crate::int::triangulation::IntTriangulation;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::float::adapter::PathToFloat;
use serde::Serialize;

/// A triangulation result based on integer computation, with float mapping.
///
/// Internally uses an [`IntTriangulation`] for performance and robustness,
/// and maps results back to user-provided float types via a [`FloatPointAdapter`].
///
/// # Parameters
/// - `P`: Float point type (e.g., `Vec2`, `[f32; 2]`, etc.)
/// - `T`: Float scalar type (e.g., `f32`, `f64`)
pub struct RawTriangulation<P: FloatPointCompatible<T>, T: FloatNumber> {
    pub raw: IntTriangulation,
    pub adapter: FloatPointAdapter<P, T>,
}

/// A flat triangulation result consisting of float points and triangle indices.
///
/// Useful for rendering, exporting, or post-processing the mesh in float space.
#[derive(Debug, Serialize)]
pub struct Triangulation<P> {
    pub points: Vec<P>,
    pub indices: Vec<usize>,
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
    pub fn triangle_indices(&self) -> Vec<usize> {
        self.raw.triangle_indices()
    }

    /// Converts this raw triangulation into a flat [`Triangulation`] (points + indices).
    #[inline]
    pub fn to_triangulation(&self) -> Triangulation<P> {
        Triangulation {
            indices: self.triangle_indices(),
            points: self.points(),
        }
    }
}
