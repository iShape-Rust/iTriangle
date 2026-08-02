use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub trait SliceContour<I: IntNumber> {
    fn slice_contour(&self, max_edge_length: I::WideUInt) -> Self;
}

impl<I: IntNumber> SliceContour<I> for IntContour<I> {
    #[inline]
    fn slice_contour(&self, max_edge_length: I::WideUInt) -> Self {
        let mut a = if let Some(last) = self.last() {
            *last
        } else {
            return Vec::new();
        };

        let radius = max_edge_length;
        if radius > I::WideUInt::HALF_MASK {
            return self.clone();
        }

        let sqr_radius = radius * radius;

        let mut contour = IntContour::<I>::with_capacity(2 * self.len());

        for &b in self.iter() {
            extract(a, b, radius, sqr_radius, &mut contour);
            a = b;
        }

        contour
    }
}

impl<I: IntNumber> SliceContour<I> for IntShape<I> {
    #[inline]
    fn slice_contour(&self, max_edge_length: I::WideUInt) -> Self {
        let mut shape = Vec::with_capacity(self.len());

        for contour in self.iter() {
            shape.push(contour.slice_contour(max_edge_length));
        }

        shape
    }
}

impl<I: IntNumber> SliceContour<I> for IntShapes<I> {
    #[inline]
    fn slice_contour(&self, max_edge_length: I::WideUInt) -> Self {
        let mut shapes = Vec::with_capacity(self.len());

        for shape in self.iter() {
            shapes.push(shape.slice_contour(max_edge_length));
        }

        shapes
    }
}

#[inline]
fn extract<I: IntNumber>(
    a: IntPoint<I>,
    b: IntPoint<I>,
    radius: I::WideUInt,
    sqr_radius: I::WideUInt,
    contour: &mut IntContour<I>,
) {
    let ab = b - a;
    let sqr_len = ab.sqr_length().to_uint();
    if sqr_len <= sqr_radius {
        contour.push(b);
        return;
    }
    let len = i_overlay::i_float::float::number::FloatNumber::sqrt(sqr_len.to_f64());
    let n = ((len + 0.5 * radius.to_f64()) / radius.to_f64()) as usize;

    if n <= 1 {
        contour.push(b);
        return;
    }

    if n == 2 {
        let x = I::from_wide((a.x.to_wide() + b.x.to_wide()) / I::Wide::TWO);
        let y = I::from_wide((a.y.to_wide() + b.y.to_wide()) / I::Wide::TWO);

        contour.push(IntPoint::new(x, y));
        contour.push(b);
        return;
    }

    let n_w = I::Wide::from_usize(n);
    for i in 1..n {
        let i_w = I::Wide::from_usize(i);
        let x = I::from_wide(a.x.to_wide() + ab.x * i_w / n_w);
        let y = I::from_wide(a.y.to_wide() + ab.y * i_w / n_w);

        contour.push(IntPoint::new(x, y));
    }
    contour.push(b);
}

#[cfg(test)]
mod tests {
    use crate::tessellation::split::SliceContour;
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn test_0() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let s0 = contour.slice_contour(8u64);
        assert_eq!(s0.len(), 4);

        let s1 = contour.slice_contour(7u64);
        assert_eq!(s1.len(), 4);

        let s2 = contour.slice_contour(6u64);
        assert_eq!(s2.len(), 8);

        let s3 = contour.slice_contour(5u64);
        assert_eq!(s3.len(), 8);

        let s4 = contour.slice_contour(3u64);
        assert_eq!(s4.len(), 12);
    }
}
