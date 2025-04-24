use crate::int::custom::IntCustomTriangulatable;
use crate::int::triangulator::Validation;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;
use crate::float::triangulation::RawTriangulation;
use crate::int::triangulation::RawIntTriangulation;

/// A trait for triangulating float geometry with user-defined validation rules.
///
/// Accepts a custom [`Validation`] object for tuning fill rule, min area, etc.
pub trait CustomTriangulatable<P: FloatPointCompatible<T>, T: FloatNumber> {

    /// Performs triangulation using the specified [`Validation`] settings.
    fn custom_triangulate(&self, validation: Validation) -> RawTriangulation<P, T>;

    /// Performs triangulation with Steiner points and a custom [`Validation`] config.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation,
    ) -> RawTriangulation<P, T>;
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> CustomTriangulatable<P, T> for Contour<P> {
    fn custom_triangulate(&self, validation: Validation) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation,
    ) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> CustomTriangulatable<P, T> for [Contour<P>] {
    fn custom_triangulate(&self, validation: Validation) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation,
    ) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> CustomTriangulatable<P, T> for [Shape<P>] {
    fn custom_triangulate(&self, validation: Validation) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation,
    ) -> RawTriangulation<P, T> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, T>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::empty(),
                adapter: FloatPointAdapter::<P, T>::new(FloatRect::zero()),
            }
        }
    }
}
