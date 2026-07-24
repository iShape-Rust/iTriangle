use crate::generic::adapter::{IntPointAdapter, PointAdapter};
use crate::generic::triangulation::RawTriangulation;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::unchecked::IntUncheckedTriangulatable;
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::base::data::{Contour, Shape};
use i_overlay::i_shape::float::rect::RectInit;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use i_tree::Expiration;

/// A trait for triangulating already valid geometry.
///
/// Skips all validation for performance. Ideal when input is generated programmatically.
///
/// # Safety Requirements
/// - Outer contours must be counter-clockwise
/// - Holes must be clockwise
/// - Steiner points must lie strictly within the shape
pub trait UncheckedTriangulatable<P> {
    type Adapter: PointAdapter<Point = P>;

    /// Triangulates geometry without validation or simplification.
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter>;

    /// Same as `unchecked_triangulate`, but inserts user-defined Steiner points.
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[P],
    ) -> RawTriangulation<Self::Adapter>;
}

/// Float-only. You can choose the integer coordinate type to be used internally
/// by the triangulator.
pub trait UncheckedTriangulatableAs<P: FloatPointCompatible>: UncheckedTriangulatable<P> {
    /// Triangulates without validation using the requested integer coordinate type.
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey;

    /// Same as `unchecked_triangulate_as`, but inserts user-defined Steiner points.
    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey;
}

impl<P> UncheckedTriangulatable<P> for [P]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_as::<i32>()
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[P],
    ) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> UncheckedTriangulatableAs<P> for [P]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntUncheckedTriangulatable::uncheck_triangulate(&int_contour);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(
                &int_contour,
                &int_points,
            );
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> UncheckedTriangulatable<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_as::<i32>()
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[P],
    ) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> UncheckedTriangulatableAs<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntUncheckedTriangulatable::uncheck_triangulate(&int_shape);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(
                &int_shape,
                &int_points,
            );
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<P> UncheckedTriangulatable<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_as::<i32>()
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[P],
    ) -> RawTriangulation<Self::Adapter> {
        self.unchecked_triangulate_with_steiner_points_as::<i32>(points)
    }
}

impl<P> UncheckedTriangulatableAs<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn unchecked_triangulate_as<I>(&self) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shapes: IntShapes<I> = self
                .iter()
                .map(|shape| shape.iter().map(|c| adapter.points_to_int(c)).collect())
                .collect();
            let raw = IntUncheckedTriangulatable::uncheck_triangulate(&int_shapes);
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }

    fn unchecked_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + SortKey,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_shapes: IntShapes<I> = self
                .iter()
                .map(|shape| shape.iter().map(|c| adapter.points_to_int(c)).collect())
                .collect();
            let raw = IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(
                &int_shapes,
                &int_points,
            );
            RawTriangulation { raw, adapter }
        } else {
            RawTriangulation {
                raw: RawIntTriangulation::default(),
                adapter: FloatPointAdapter::<P, I>::new(FloatRect::zero()),
            }
        }
    }
}

impl<I> UncheckedTriangulatable<IntPoint<I>> for IntContour<I>
where
    I: IntNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate(self),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}

impl<I> UncheckedTriangulatable<IntPoint<I>> for IntShape<I>
where
    I: IntNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate(self),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}

impl<I> UncheckedTriangulatable<IntPoint<I>> for IntShapes<I>
where
    I: IntNumber + Expiration + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn unchecked_triangulate(&self) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate(self),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn unchecked_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntUncheckedTriangulatable::uncheck_triangulate_with_steiner_points(self, points),
            IntPointAdapter::new(),
        )
    }
}
