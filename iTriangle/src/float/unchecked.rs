use crate::float::triangulation::RawTriangulation;
use crate::int::triangulation::IntTriangulation;
use crate::int::unchecked::IntUncheckedTriangulatable;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;

/// A trait for triangulating already valid float-based geometry.
///
/// Skips all validation for performance. Ideal when input is generated programmatically.
///
/// # Safety Requirements
/// - Outer contours must be counter-clockwise
/// - Holes must be clockwise
/// - Steiner points must lie strictly within the shape
pub trait UncheckedTriangulatable<P: FloatPointCompatible<T>, T: FloatNumber> {
    /// Triangulates float geometry without validation or simplification.
    fn unchecked_triangulate(&self) -> RawTriangulation<P, T>;
    /// Same as `unchecked_triangulate`, but inserts user-defined Steiner points.
    fn unchecked_triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T>;
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> UncheckedTriangulatable<P, T> for [P] {
    fn unchecked_triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).unchecked_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .unchecked_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> UncheckedTriangulatable<P, T> for [Contour<P>] {
    fn unchecked_triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).unchecked_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .unchecked_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> UncheckedTriangulatable<P, T> for [Shape<P>] {
    fn unchecked_triangulate(&self) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).unchecked_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .unchecked_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: IntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}
