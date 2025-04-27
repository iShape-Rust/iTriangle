use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::base::data::Shape;
use i_overlay::i_shape::int::shape::IntShape;

pub(super) trait ToFloatShape {
    fn to_float_shape(&self) -> Shape<[f64; 2]>;
}

impl ToFloatShape for IntShape {
    fn to_float_shape(&self) -> Shape<[f64; 2]> {
        self.iter()
            .map(|contour| contour.iter().map(|p| [p.x as f64, p.y as f64]).collect())
            .collect()
    }
}

pub(super) trait ToIntShape {
    fn to_int_shape(&self) -> IntShape;
}

impl ToIntShape for Shape<[f64; 2]> {
    fn to_int_shape(&self) -> IntShape {
        self.iter()
            .map(|contour| {
                contour
                    .iter()
                    .map(|p| IntPoint::new(p[0] as i32, p[1] as i32))
                    .collect()
            })
            .collect()
    }
}
