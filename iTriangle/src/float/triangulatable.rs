use crate::float::triangulation::RawTriangulation;
use crate::int::triangulatable::IntTriangulatable;
use crate::int::triangulation::RawIntTriangulation;
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
    fn triangulate(&self) -> RawTriangulation<P>;

    /// Triangulates the shape(s) and inserts the given Steiner points.
    ///
    /// Points must lie strictly within the interior of the geometry.
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P>;
}

impl<P: FloatPointCompatible> Triangulatable<P> for [P] {
    fn triangulate(&self) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible> Triangulatable<P> for [Contour<P>] {
    fn triangulate(&self) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P: FloatPointCompatible> Triangulatable<P> for [Shape<P>] {
    fn triangulate(&self) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let raw = self.to_int(&adapter).triangulate();
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<P> {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P>::new(rect);
            let float_points = points.to_int(&adapter);
            let raw = self
                .to_int(&adapter)
                .triangulate_with_steiner_points(&float_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P>::new(FloatRect::zero()),
            }
        }
    }
}
