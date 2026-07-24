use crate::generic::adapter::{IntPointAdapter, PointAdapter};
use crate::generic::triangulation::RawTriangulation;
use crate::int::triangulatable::IntTriangulatable;
use crate::int::triangulation::RawIntTriangulation;
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::rect::RectInit;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use i_tree::{Expiration, LayoutNumber};

/// A trait for triangulating geometry with default validation.
///
/// Automatically converts the input to integer space when needed, applies validation,
/// and returns an adapter-mapped result.
///
/// # Implemented For
/// - `[P]` / `[Contour<P>]` / `[Shape<P>]` (float)
/// - [`IntContour`] / [`IntShape`] / [`IntShapes`] (integer, [`IntPointAdapter`])
pub trait Triangulatable<P> {
    type Adapter: PointAdapter<Point = P>;

    /// Triangulates the shape(s) using the default [`Triangulator`] configuration.
    ///
    /// Validation includes contour simplification, direction correction, and area filtering.
    fn triangulate(&self) -> RawTriangulation<Self::Adapter>;

    /// Triangulates the shape(s) and inserts the given Steiner points.
    ///
    /// Points must lie strictly within the interior of the geometry.
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<Self::Adapter>;
}

/// Float-only. You can choose the integer coordinate type to be used internally
/// by the triangulator.
pub trait TriangulatableAs<P: FloatPointCompatible>: Triangulatable<P> {
    /// Triangulates the shape(s) using the requested integer coordinate type.
    fn triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;

    /// Triangulates the shape(s) with Steiner points using the requested integer coordinate type.
    fn triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;
}

impl<P> Triangulatable<P> for [P]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.triangulate_as::<i32>()
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<Self::Adapter> {
        self.triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> TriangulatableAs<P> for [P]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntTriangulatable::triangulate(&int_contour);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntTriangulatable::triangulate_with_steiner_points(&int_contour, &int_points);
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
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.triangulate_as::<i32>()
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<Self::Adapter> {
        self.triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> TriangulatableAs<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntTriangulatable::triangulate(&int_shape);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntTriangulatable::triangulate_with_steiner_points(&int_shape, &int_points);
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
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.triangulate_as::<i32>()
    }

    #[inline]
    fn triangulate_with_steiner_points(&self, points: &[P]) -> RawTriangulation<Self::Adapter> {
        self.triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> TriangulatableAs<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shapes: IntShapes<I> = self
                .iter()
                .map(|shape| shape.iter().map(|c| adapter.points_to_int(c)).collect())
                .collect();
            let raw = IntTriangulatable::triangulate(&int_shapes);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_shapes: IntShapes<I> = self
                .iter()
                .map(|shape| shape.iter().map(|c| adapter.points_to_int(c)).collect())
                .collect();
            let raw = IntTriangulatable::triangulate_with_steiner_points(&int_shapes, &int_points);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<I> Triangulatable<IntPoint<I>> for IntContour<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(IntTriangulatable::triangulate(self), IntPointAdapter::new())
    }

    #[inline]
    fn triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntTriangulatable::triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}

impl<I> Triangulatable<IntPoint<I>> for IntShape<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(IntTriangulatable::triangulate(self), IntPointAdapter::new())
    }

    #[inline]
    fn triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntTriangulatable::triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}

impl<I> Triangulatable<IntPoint<I>> for IntShapes<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(IntTriangulatable::triangulate(self), IntPointAdapter::new())
    }

    #[inline]
    fn triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntTriangulatable::triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}
