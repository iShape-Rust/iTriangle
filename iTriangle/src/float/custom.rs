use crate::float::triangulation::RawTriangulation;
use crate::int::custom::IntCustomTriangulatable;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::validation::Validation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;

/// A trait for triangulating float geometry with user-defined validation rules.
///
/// Accepts a custom [`Validation`] object for tuning fill rule, min area, etc.
pub trait CustomTriangulatable<P: FloatPointCompatible> {
    /// Performs triangulation using the specified [`Validation`] settings.
    fn custom_triangulate(&self, validation: Validation<i32>) -> RawTriangulation<P> {
        self.custom_triangulate_as(validation)
    }

    /// Performs triangulation using the requested integer coordinate type.
    fn custom_triangulate_as<I>(&self, validation: Validation<I>) -> RawTriangulation<P, I>
    where
        I: OverlayInt;

    /// Performs triangulation with Steiner points and a custom [`Validation`] config.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation<i32>,
    ) -> RawTriangulation<P> {
        self.custom_triangulate_with_steiner_points_as(points, validation)
    }

    /// Performs triangulation with Steiner points using the requested integer coordinate type.
    fn custom_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
        validation: Validation<I>,
    ) -> RawTriangulation<P, I>
    where
        I: OverlayInt;
}

impl<P> CustomTriangulatable<P> for Contour<P>
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(&self, validation: Validation<I>) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
        validation: Validation<I>,
    ) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> CustomTriangulatable<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(&self, validation: Validation<I>) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
        validation: Validation<I>,
    ) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> CustomTriangulatable<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(&self, validation: Validation<I>) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).custom_triangulate(validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn custom_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
        validation: Validation<I>,
    ) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .custom_triangulate_with_steiner_points(&float_points, validation);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}
