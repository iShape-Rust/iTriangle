use alloc::vec::Vec;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};

pub trait SliceContour {
    fn slice_contour(&self, max_edge_length: u32) -> Self;
}

const SCALE: u32 = 29;

impl SliceContour for IntContour {
    #[inline]
    fn slice_contour(&self, max_edge_length: u32) -> Self {
        let mut a = if let Some(last) = self.last() {
            *last
        } else {
            return Vec::new();
        };

        let radius = max_edge_length as u64;
        let sqr_radius = radius.pow(2);

        let mut contour = IntContour::with_capacity(2 * self.len());

        for &b in self.iter() {
            extract(a, b, radius, sqr_radius, &mut contour);
            a = b;
        }

        contour
    }
}

impl SliceContour for IntShape {
    #[inline]
    fn slice_contour(&self, max_edge_length: u32) -> Self {
        let mut shape = Vec::with_capacity(self.len());

        for contour in self.iter() {
            shape.push(contour.slice_contour(max_edge_length));
        }

        shape
    }
}

impl SliceContour for IntShapes {
    #[inline]
    fn slice_contour(&self, max_edge_length: u32) -> Self {
        let mut shapes = Vec::with_capacity(self.len());

        for shape in self.iter() {
            shapes.push(shape.slice_contour(max_edge_length));
        }

        shapes
    }
}

#[inline]
fn extract(a: IntPoint, b: IntPoint, radius: u64, sqr_radius: u64, contour: &mut IntContour) {
    let ab = b.subtract(a);
    let sqr_len = ab.sqr_length() as u64;
    if sqr_len <= sqr_radius {
        contour.push(b);
        return;
    }
    let len = sqr_len.isqrt();
    let n = ((len + (radius >> 1)) / radius) as i64;

    if n <= 1 {
        contour.push(b);
        return;
    }

    if n == 2 {
        let x = ((a.x as i64 + b.x as i64) / 2) as i32;
        let y = ((a.y as i64 + b.y as i64) / 2) as i32;

        contour.push(IntPoint::new(x, y));
        contour.push(b);
        return;
    }


    let sx = (ab.x << SCALE) / n;
    let sy = (ab.y << SCALE) / n;

    let mut dx = 0i64;
    let mut dy = 0i64;

    for _ in 1..n {
        dx += sx;
        dy += sy;
        
        let x = a.x + (dx >> SCALE) as i32;
        let y = a.y + (dy >> SCALE) as i32;
        
        contour.push(IntPoint::new(x, y));
    }
    contour.push(b);
}


#[cfg(test)]
mod tests {
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;
    use crate::tessellation::split::SliceContour;

    #[test]
    fn test_0() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let s0 = contour.slice_contour(8);
        assert_eq!(s0.len(), 4);

        let s1 = contour.slice_contour(7);
        assert_eq!(s1.len(), 4);

        let s2 = contour.slice_contour(6);
        assert_eq!(s2.len(), 8);

        let s3 = contour.slice_contour(5);
        assert_eq!(s3.len(), 8);

        let s4 = contour.slice_contour(3);
        assert_eq!(s4.len(), 12);
    }
}