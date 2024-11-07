use std::marker::PhantomData;
use i_float::float::compatible::FloatPointCompatible;
use i_float::float::number::FloatNumber;
use crate::stroke::style::StrokeStyle;
use crate::triangulation::float::Triangulation;

struct Segment<P, T> {
    a: P,
    b: P,
    direction: P,
    phantom: PhantomData<T>,
}

impl<P, T> Segment<P, T>
where
    P: FloatPointCompatible<T>,
    T: FloatNumber,
{
    fn new(a: P, b: P) -> Segment<P, T> {
        let dx = b.x() - a.x();
        let dy = b.y() - a.y();

        let length = (dx * dx + dy * dy).sqrt();

        let x = dx / length;
        let y = dy / length;

        let direction = P::from_xy(x, y);
        Self {
            a,
            b,
            direction,
            phantom: PhantomData::default(),
        }
    }
}

pub struct ButtStrokeBuilder<T: FloatNumber> {
    stroke_style: StrokeStyle<T>,
}

impl<T: FloatNumber> ButtStrokeBuilder<T> {
    pub fn new(stroke_style: StrokeStyle<T>) -> ButtStrokeBuilder<T> {
        Self { stroke_style }
    }

    pub fn build_closed_path_mesh<P: FloatPointCompatible<T>>(&self, path: &[P]) -> Triangulation<P> {
        let n = path.len();
        if n < 2{
            return Triangulation { points: vec![], indices: vec![] };
        }

        let mut points = Vec::with_capacity(4 * n);
        let mut indices = Vec::with_capacity(12 * n);

        let r = T::from_float(0.5) * self.stroke_style.width;

        let a = path[(n + n - 2) % n];
        let b = path[n - 1];

        let mut seg0 = Segment::new(a, b);

        for i in 0..n {
            let c = path[i];
            let seg1 = Segment::new(seg0.b, c);
            self.join_butt_segment(&mut points, &mut indices, &seg0, r);

            let is_close = i + 1 == n;
            self.join_butt_joint(&mut points, &mut indices, &seg0, &seg1, is_close);

            seg0 = seg1;
        }

        Triangulation { points, indices }
    }

    fn join_butt_joint<P: FloatPointCompatible<T>>(&self, points: &mut Vec<P>, indices: &mut Vec<usize>, seg0: &Segment<P, T>, seg1: &Segment<P, T>, is_last: bool) {
        let v0 = Self::cw_rotate_90(&seg0.direction);
        let v1 = Self::cw_rotate_90(&seg1.direction);
        let cross = Self::cross_product(&v0, &v1).to_f64();

        if cross.abs() < 1e-7 {
            return;
        }

        let m = points.len();            // Current vertex as a midpoint
        points.push(seg0.b);

        let (a, b) = if cross > 0.0 {
            let a = points.len() - 2;
            let b = if is_last { 1 } else { points.len() + 1 };
            (a, b)
        } else {
            let a = points.len() - 3;
            let b = if is_last { 0 } else { points.len() };
            (a, b)
        };

        indices.extend(&[a, m, b]);
    }

    fn join_butt_segment<P: FloatPointCompatible<T>>(&self, points: &mut Vec<P>, indices: &mut Vec<usize>, segment: &Segment<P, T>, r: T) {
        let ortho = Self::cw_rotate_90(&segment.direction);
        let vr = Self::mul(r, &ortho);

        // Define top and bottom points for the cap
        let s_top = Self::add(&segment.a, &vr);
        let s_bot = Self::sub(&segment.a, &vr);
        let e_top = Self::add(&segment.b, &vr);
        let e_bot = Self::sub(&segment.b, &vr);

        // Indices for the mesh
        let start_index = points.len();
        points.push(s_top);
        points.push(s_bot);
        points.push(e_top);
        points.push(e_bot);

        indices.extend(&[
            start_index, start_index + 1, start_index + 2,
            start_index + 1, start_index + 3, start_index + 2,
        ]);
    }

    #[inline]
    fn cw_rotate_90<P: FloatPointCompatible<T>>(vector: &P) -> P {
        P::from_xy(-vector.y(), vector.x())
    }

    #[inline]
    fn cross_product<P: FloatPointCompatible<T>>(a: &P, b: &P) -> T {
        a.x() * b.y() - a.y() * b.x()
    }

    #[inline]
    fn add<P: FloatPointCompatible<T>>(a: &P, b: &P) -> P {
        P::from_xy(a.x() + b.x(), a.y() + b.y())
    }

    #[inline]
    fn sub<P: FloatPointCompatible<T>>(a: &P, b: &P) -> P {
        P::from_xy(a.x() - b.x(), a.y() - b.y())
    }

    #[inline]
    fn mul<P: FloatPointCompatible<T>>(s: T, a: &P) -> P {
        P::from_xy(s * a.x(), s * a.y())
    }
}

#[cfg(test)]
mod tests {
    use i_float::float::point::FloatPoint;
    use crate::stroke::builder::ButtStrokeBuilder;
    use crate::stroke::style::StrokeStyle;

    #[test]
    fn test_0() {
        let path = [
            FloatPoint::new(0.0, 0.0),
            FloatPoint::new(0.0, 10.0),
            FloatPoint::new(10.0, 10.0),
            FloatPoint::new(10.0, 0.0),
        ];

        let stroke_builder = ButtStrokeBuilder::new(StrokeStyle::with_width(2.0));
        let triangulation = stroke_builder.build_closed_path_mesh(&path);


        println!("points: {:?}", triangulation.points);
        println!("indices: {:?}", triangulation.indices);
    }
}