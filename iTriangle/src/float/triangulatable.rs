use crate::float::triangulation::RawTriangulation;
use crate::int::triangulatable::IntTriangulatable;
use crate::int::triangulation::RawIntTriangulation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;

/// A trait for triangulating float-based geometry with default validation.
///
/// Automatically converts the input to integer space, applies validation,
/// and returns a float-mapped result.
///
/// # Implemented For
/// - `Contour<P>`
/// - `[Contour<P>]`
/// - `[Shape<P>]`
pub trait Triangulatable<P: FloatPointCompatible> {
    /// Triangulates the shape(s) using the default [`Triangulator`] configuration.
    ///
    /// Validation includes contour simplification, direction correction, and area filtering.
    fn triangulate(&self) -> RawTriangulation<P> {
        self.triangulate_as::<i32>()
    }

    /// Triangulates the shape(s) using the requested integer coordinate type.
    fn triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: OverlayInt;

    /// Triangulates the shape(s) and inserts the given Steiner points.
    ///
    /// Points must lie strictly within the interior of the geometry.
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P> {
        self.triangulate_with_steiner_points_as::<i32>(points)
    }

    /// Triangulates the shape(s) with Steiner points using the requested integer coordinate type.
    fn triangulate_with_steiner_points_as<I>(&self, points: &[P]) -> RawTriangulation<P, I>
    where
        I: OverlayInt;
}

impl<P> Triangulatable<P> for [P]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(&self, points: &[P]) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> Triangulatable<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(&self, points: &[P]) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> Triangulatable<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(&self, points: &[P]) -> RawTriangulation<P, I>
    where
        I: OverlayInt,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}
