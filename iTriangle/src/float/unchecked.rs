use crate::float::triangulation::RawTriangulation;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::unchecked::IntUncheckedTriangulatable;
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::adapter::{PathToInt, ShapeToInt, ShapesToInt};
use i_overlay::i_shape::float::rect::RectInit;
use i_tree::Expiration;

/// A trait for triangulating already valid float-based geometry.
///
/// Skips topology validation for performance. Coordinate bounds are still validated.
///
/// # Safety Requirements
/// - Outer contours must be counter-clockwise
/// - Holes must be clockwise
/// - Steiner points must lie strictly within the shape
pub trait UncheckedTriangulatable<P: FloatPointCompatible> {
    /// Triangulates float geometry without validation or simplification.
    fn unchecked_triangulate(&self) -> RawTriangulation<P> {
        self.unchecked_triangulate_as::<i32>()
    }

    /// Triangulates float geometry without validation using the requested integer coordinate type.
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey;

    /// Same as `unchecked_triangulate`, but inserts user-defined Steiner points.
    fn unchecked_triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P> {
        self.unchecked_triangulate_with_steiner_points_as::<i32>(points)
    }

    /// Same as `unchecked_triangulate_as`, but inserts user-defined Steiner points.
    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey;
}

impl<P> UncheckedTriangulatable<P> for [P]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self).expect("Invalid triangulation bounds") {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let raw = self.to_int(&adapter).uncheck_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self).expect("Invalid triangulation bounds") {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .uncheck_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }
}

impl<P> UncheckedTriangulatable<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self).expect("Invalid triangulation bounds") {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let raw = self.to_int(&adapter).uncheck_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self).expect("Invalid triangulation bounds") {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .uncheck_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }
}

impl<P> UncheckedTriangulatable<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) =
            FloatRect::with_list_of_paths(self).expect("Invalid triangulation bounds")
        {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let raw = self.to_int(&adapter).uncheck_triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<P, I>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) =
            FloatRect::with_list_of_paths(self).expect("Invalid triangulation bounds")
        {
            let adapter = FloatPointAdapter::<P, I>::new_conservative(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .uncheck_triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new_conservative(FloatRect::zero()),
            }
        }
    }
}
