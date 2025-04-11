use i_overlay::i_float::float::number::FloatNumber;

pub struct StrokeStyle<T: FloatNumber> {
    pub width: T,
    pub point_count: usize,
    pub start_cap: bool,
    pub end_cap: bool,
}

impl<T: FloatNumber> StrokeStyle<T> {
    pub fn with_width(width: T) -> StrokeStyle<T> {
        Self {
            width,
            point_count: 0,
            start_cap: true,
            end_cap: true,
        }
    }
}