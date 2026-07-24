use crate::generic::adapter::{IntPointAdapter, PointAdapter};
use crate::generic::triangulation::RawTriangulation;
use crate::int::custom::IntCustomTriangulatable;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::validation::Validation;
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

/// A trait for triangulating geometry with user-defined validation rules.
///
/// Accepts a custom [`Validation`] object for tuning fill rule, min area, etc.
pub trait CustomTriangulatable<P> {
    type Adapter: PointAdapter<Point = P>;

    /// Performs triangulation using the specified [`Validation`] settings.
    fn custom_triangulate(
        &self,
        validation: Validation<<Self::Adapter as PointAdapter>::Int>,
    ) -> RawTriangulation<Self::Adapter>;

    /// Performs triangulation with Steiner points and a custom [`Validation`] config.
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation<<Self::Adapter as PointAdapter>::Int>,
    ) -> RawTriangulation<Self::Adapter>;
}

/// Float-only. You can choose the integer coordinate type to be used internally
/// by the triangulator.
pub trait CustomTriangulatableAs<P: FloatPointCompatible>: CustomTriangulatable<P> {
    /// Performs triangulation using the requested integer coordinate type.
    fn custom_triangulate_as<I>(
        &self,
        validation: Validation<I>,
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;

    /// Performs triangulation with Steiner points using the requested integer coordinate type.
    fn custom_triangulate_with_steiner_points_as<I>(
        &self,
        points: &[P],
        validation: Validation<I>,
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;
}

impl<P> CustomTriangulatable<P> for [P]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<i32>) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_as(validation)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation<i32>,
    ) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_with_steiner_points_as(points, validation)
    }
}

impl<P> CustomTriangulatableAs<P> for [P]
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(
        &self,
        validation: Validation<I>,
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntCustomTriangulatable::custom_triangulate(&int_contour, validation);
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
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_path(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_contour: IntContour<I> = adapter.points_to_int(self);
            let raw = IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                &int_contour,
                &int_points,
                validation,
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

impl<P> CustomTriangulatable<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<i32>) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_as(validation)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation<i32>,
    ) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_with_steiner_points_as(points, validation)
    }
}

impl<P> CustomTriangulatableAs<P> for [Contour<P>]
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(
        &self,
        validation: Validation<I>,
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntCustomTriangulatable::custom_triangulate(&int_shape, validation);
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
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_points = adapter.points_to_int(points);
            let int_shape: IntShape<I> = self.iter().map(|c| adapter.points_to_int(c)).collect();
            let raw = IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                &int_shape,
                &int_points,
                validation,
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

impl<P> CustomTriangulatable<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    type Adapter = FloatPointAdapter<P, i32>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<i32>) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_as(validation)
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[P],
        validation: Validation<i32>,
    ) -> RawTriangulation<Self::Adapter> {
        self.custom_triangulate_with_steiner_points_as(points, validation)
    }
}

impl<P> CustomTriangulatableAs<P> for [Shape<P>]
where
    P: FloatPointCompatible,
{
    fn custom_triangulate_as<I>(
        &self,
        validation: Validation<I>,
    ) -> RawTriangulation<FloatPointAdapter<P, I>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        if let Some(rect) = FloatRect::with_list_of_paths(self) {
            let adapter = FloatPointAdapter::<P, I>::new(rect);
            let int_shapes: IntShapes<I> = self
                .iter()
                .map(|shape| shape.iter().map(|c| adapter.points_to_int(c)).collect())
                .collect();
            let raw = IntCustomTriangulatable::custom_triangulate(&int_shapes, validation);
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
            let raw = IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                &int_shapes,
                &int_points,
                validation,
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

impl<I> CustomTriangulatable<IntPoint<I>> for IntContour<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate(self, validation),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                self, points, validation,
            ),
            IntPointAdapter::new(),
        )
    }
}

impl<I> CustomTriangulatable<IntPoint<I>> for IntShape<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate(self, validation),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                self, points, validation,
            ),
            IntPointAdapter::new(),
        )
    }
}

impl<I> CustomTriangulatable<IntPoint<I>> for IntShapes<I>
where
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    type Adapter = IntPointAdapter<I>;

    #[inline]
    fn custom_triangulate(&self, validation: Validation<I>) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate(self, validation),
            IntPointAdapter::new(),
        )
    }

    #[inline]
    fn custom_triangulate_with_steiner_points(
        &self,
        points: &[IntPoint<I>],
        validation: Validation<I>,
    ) -> RawTriangulation<Self::Adapter> {
        RawTriangulation::new(
            IntCustomTriangulatable::custom_triangulate_with_steiner_points(
                self, points, validation,
            ),
            IntPointAdapter::new(),
        )
    }
}
