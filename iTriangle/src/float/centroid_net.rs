use crate::float::delaunay::Delaunay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::base::data::Contour;
use i_overlay::i_shape::float::adapter::ShapeToFloat;

impl<P: FloatPointCompatible<T>, T: FloatNumber> Delaunay<P, T> {
    #[inline]
    pub fn to_centroid_net(&self, min_area: T) -> Vec<Contour<P>> {
        let int_area = self.adapter.sqr_float_to_int(min_area);
        self.delaunay.centroid_net(int_area).to_float(&self.adapter)
    }
}
