use i_overlay::core::fill_rule::FillRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::base::data::Contour;
use i_overlay::i_shape::float::adapter::{ShapeToFloat, ShapeToInt};
use i_overlay::i_shape::float::rect::RectInit;
use crate::triangulation::int::IntTriangulate;

#[derive(Debug)]
pub struct Triangulation<P> {
    pub points: Vec<P>,
    pub indices: Vec<usize>,
}

pub trait FloatTriangulate<P: FloatPointCompatible<T>, T: FloatNumber> {
    fn to_triangulation(&self, validate_rule: Option<FillRule>, min_area: T) -> Triangulation<P>;
    fn to_convex_polygons(&self, validate_rule: Option<FillRule>, min_area: T) -> Vec<Contour<P>>;
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> FloatTriangulate<P, T> for [Contour<P>] {
    fn to_triangulation(&self, validate_rule: Option<FillRule>, min_area: T) -> Triangulation<P> {
        let rect = if let Some(rect) = FloatRect::with_paths(self) {
            rect
        } else {
            return Triangulation { points: vec![], indices: vec![] };
        };

        let adapter = FloatPointAdapter::<P, T>::new(rect);
        let shape = self.to_int(&adapter);
        let int_min_area = adapter.sqr_float_to_int(min_area);

        let triangulation = shape.to_triangulation(validate_rule, int_min_area);

        let points = triangulation.points.iter().map(|p| adapter.int_to_float(p)).collect();

        Triangulation { points, indices: triangulation.indices }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>, min_area: T) -> Vec<Contour<P>> {
        let rect = if let Some(rect) = FloatRect::with_paths(self) {
            rect
        } else {
            return vec![];
        };

        let adapter = FloatPointAdapter::<P, T>::new(rect);
        let shape = self.to_int(&adapter);
        let int_min_area = adapter.sqr_float_to_int(min_area);

        let polygons = shape.to_convex_polygons(validate_rule, int_min_area);

        polygons.to_float(&adapter)
    }
}

pub struct TriangulationBuilder<P> {
    points: Vec<P>,
    indices: Vec<usize>,
}

impl<P> TriangulationBuilder<P> {
    /// Appends another `Triangulation` to the builder.
    ///
    /// This method correctly offsets the indices of the appended triangulation
    /// based on the current number of points in the builder.
    pub fn append(&mut self, triangulation: Triangulation<P>) -> &mut Self {
        let offset = self.points.len();
        self.points.extend(triangulation.points);
        self.indices.extend(
            triangulation
                .indices
                .iter()
                .map(|&i| i + offset),
        );
        self
    }

    /// Builds and returns the final `Triangulation`.
    pub fn build(self) -> Triangulation<P> {
        Triangulation {
            points: self.points,
            indices: self.indices,
        }
    }
}

impl<P> Default for TriangulationBuilder<P> {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }
}
