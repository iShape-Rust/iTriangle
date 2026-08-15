use crate::float::delaunay::Delaunay;
use crate::int::uniform::IntUniformTriangulatable;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_shape::float::adapter::PathToInt;
use i_overlay::i_shape::int::shape::IntShape;
use i_overlay::i_shape::source::resource::ShapeResource;

/// Float wrapper for the integer uniform triangulation pipeline.
///
/// The input is converted once with a shared [`FloatPointAdapter`]. Boundary
/// splitting, topology normalization, lattice generation, boundary clearance,
/// and Delaunay triangulation are all performed by the integer implementation.
///
/// # Example
///
/// ```
/// use i_triangle::float::uniform::UniformTriangulatable;
///
/// let contour = [
///     [0.0, 0.0],
///     [10.0, 0.0],
///     [10.0, 10.0],
///     [0.0, 10.0],
/// ];
///
/// let mesh = contour
///     .uniform_triangulate(2.0)
///     .to_triangulation::<u32>();
///
/// assert!(!mesh.indices.is_empty());
/// ```
pub trait UniformTriangulatable<P: FloatPointCompatible> {
    /// Triangulates with the default `i32` integer engine.
    fn uniform_triangulate(&self, edge_length: P::Scalar) -> Delaunay<P> {
        self.uniform_triangulate_as::<i32>(edge_length)
    }

    /// Triangulates with the requested integer engine.
    fn uniform_triangulate_as<I>(&self, edge_length: P::Scalar) -> Delaunay<P, I>
    where
        I: OverlayInt;
}

impl<S, P> UniformTriangulatable<P> for S
where
    S: ShapeResource<P>,
    P: FloatPointCompatible,
{
    fn uniform_triangulate_as<I>(&self, edge_length: P::Scalar) -> Delaunay<P, I>
    where
        I: OverlayInt,
    {
        assert!(
            edge_length.is_finite() && edge_length > P::Scalar::ZERO,
            "edge_length must be finite and positive"
        );

        let rect =
            FloatRect::with_iter(self.iter_paths().flatten()).unwrap_or_else(FloatRect::zero);
        let adapter = FloatPointAdapter::<P, I>::new(rect);
        let int_edge_length = adapter.round_len_to_int(edge_length);
        assert!(
            int_edge_length > I::ONE,
            "edge_length is below the precision of the selected integer engine"
        );

        let shape: IntShape<I> = self
            .iter_paths()
            .map(|path| path.to_int(&adapter))
            .collect();
        let delaunay = shape.uniform_triangulate(int_edge_length.to_uint());

        Delaunay { delaunay, adapter }
    }
}

#[cfg(test)]
mod tests {
    use super::UniformTriangulatable;
    use alloc::vec;

    #[test]
    fn fills_square_with_boundary_and_grid_points() {
        let contour = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];

        let triangulation = contour.uniform_triangulate(2.0).to_triangulation::<u32>();

        assert!(triangulation.points.len() > 20);
        triangulation.validate(100.0, 0.000_001);
    }

    #[test]
    fn fills_shape_without_filling_hole() {
        let shape = vec![
            vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]],
            vec![[7.0, 7.0], [7.0, 13.0], [13.0, 13.0], [13.0, 7.0]],
        ];

        let triangulation = shape.uniform_triangulate(2.0).to_triangulation::<u32>();

        assert!(triangulation.points.len() > 40);
        triangulation.validate(364.0, 0.000_001);
    }

    #[test]
    fn narrow_shape_falls_back_to_split_boundary() {
        let contour = [[0.0, 0.0], [10.0, 0.0], [10.0, 0.5], [0.0, 0.5]];

        let triangulation = contour.uniform_triangulate(2.0).to_triangulation::<u32>();

        assert!(!triangulation.indices.is_empty());
        triangulation.validate(5.0, 0.000_001);
    }
}
