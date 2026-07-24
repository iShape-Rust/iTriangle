use alloc::vec::Vec;
use core::marker::PhantomData;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;

pub trait PointAdapter: Clone {
    type Point: Clone;
    type Int: IntNumber;
    type Measure: Copy;

    fn to_int_point(&self, point: &Self::Point) -> IntPoint<Self::Int>;
    fn from_int_point(&self, point: &IntPoint<Self::Int>) -> Self::Point;

    fn measure_to_int_area(&self, measure: Self::Measure) -> <Self::Int as IntNumber>::WideUInt;

    #[inline]
    fn points_to_int(&self, points: &[Self::Point]) -> Vec<IntPoint<Self::Int>> {
        points.iter().map(|p| self.to_int_point(p)).collect()
    }

    #[inline]
    fn points_from_int(&self, points: &[IntPoint<Self::Int>]) -> Vec<Self::Point> {
        points.iter().map(|p| self.from_int_point(p)).collect()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IntPointAdapter<I: IntNumber> {
    marker: PhantomData<I>,
}

impl<I: IntNumber> IntPointAdapter<I> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<I: IntNumber> PointAdapter for IntPointAdapter<I> {
    type Point = IntPoint<I>;
    type Int = I;
    type Measure = I::WideUInt;

    #[inline]
    fn to_int_point(&self, point: &Self::Point) -> IntPoint<I> {
        *point
    }

    #[inline]
    fn from_int_point(&self, point: &IntPoint<I>) -> Self::Point {
        *point
    }

    #[inline]
    fn measure_to_int_area(&self, measure: Self::Measure) -> I::WideUInt {
        measure
    }
}

impl<P, I> PointAdapter for FloatPointAdapter<P, I>
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    type Point = P;
    type Int = I;
    type Measure = P::Scalar;

    #[inline]
    fn to_int_point(&self, point: &P) -> IntPoint<I> {
        self.float_to_int(point)
    }

    #[inline]
    fn from_int_point(&self, point: &IntPoint<I>) -> P {
        self.int_to_float(point)
    }

    #[inline]
    fn measure_to_int_area(&self, measure: P::Scalar) -> I::WideUInt {
        self.round_sqr_len_to_int(measure).to_uint()
    }
}
