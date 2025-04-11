use i_triangle::i_overlay::i_float::float::point::FloatPoint;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use iced::Vector;
use crate::compat::convert::Convert;

impl Convert<FloatPoint<f32>> for IntPoint {
    fn convert(&self) -> FloatPoint<f32> {
        FloatPoint::new(self.x as f32, self.y as f32)
    }
}

impl Convert<FloatPoint<f32>> for Vector<f32> {
    fn convert(&self) -> FloatPoint<f32> {
        FloatPoint::new(self.x, self.y)
    }
}

impl Convert<Vector<f32>> for IntPoint {
    fn convert(&self) -> Vector<f32> {
        Vector::new(self.x as f32, self.y as f32)
    }
}

impl Convert<Vector<f32>> for FloatPoint<f32> {
    fn convert(&self) -> Vector<f32> {
        Vector::new(self.x, self.y)
    }
}