use crate::int::triangulatable::IntTriangulatable;
use crate::int::triangulation::IntTriangulation;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;
use crate::float::triangulation::RawTriangulation;

/// A trait for triangulating float-based geometry with default validation.
///
/// Automatically converts the input to integer space, applies validation,
/// and returns a float-mapped result.
///
/// # Implemented For
/// - `Contour<P>`
/// - `[Contour<P>]`
/// - `[Shape<P>]`
pub trait Triangulatable<P: FloatPointCompatible<T>, T: FloatNumber> {
    /// Triangulates the shape(s) using the default [`Triangulator`] configuration.
    ///
    /// Validation includes contour simplification, direction correction, and area filtering.
    fn triangulate(&self) -> RawTriangulation<P, T>;

    /// Triangulates the shape(s) and inserts the given Steiner points.
    ///
    /// Points must lie strictly within the interior of the geometry.
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T>;
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> Triangulatable<P, T> for Contour<P> {
    fn triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> Triangulatable<P, T> for [Contour<P>] {
    fn triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> Triangulatable<P, T> for [Shape<P>] {
    fn triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}