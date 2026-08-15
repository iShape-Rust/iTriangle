use crate::advanced::relax::RelaxationOptions as IntRelaxationOptions;
use crate::float::delaunay::Delaunay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;

pub use crate::advanced::relax::RelaxationResult;

/// Configuration for centroid-net relaxation in float coordinate units.
///
/// This is a thin wrapper over the integer relaxation implementation. The
/// convergence tolerance is converted to the integer coordinate system by the
/// same adapter that was used to construct the mesh.
#[derive(Clone, Copy)]
pub struct RelaxationOptions<F: FloatNumber> {
    /// Maximum number of relaxation iterations.
    pub max_iterations: usize,

    /// Absolute convergence tolerance in input float coordinate units.
    ///
    /// Relaxation stops before the next move when every unconstrained
    /// centroid displacement is no greater than this value. It must be finite
    /// and non-negative.
    pub tolerance: F,
}

impl<F: FloatNumber> RelaxationOptions<F> {
    /// Creates options with zero convergence tolerance.
    #[inline]
    pub fn new(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            tolerance: F::ZERO,
        }
    }

    /// Sets the absolute convergence tolerance in float coordinate units.
    #[inline]
    pub fn with_tolerance(mut self, tolerance: F) -> Self {
        self.tolerance = tolerance;
        self
    }
}

impl<F: FloatNumber> Default for RelaxationOptions<F> {
    #[inline]
    fn default() -> Self {
        Self::new(8)
    }
}

impl<P: FloatPointCompatible, I: IntNumber> Delaunay<P, I> {
    /// Relaxes interior vertices toward their centroid-net area centroids.
    ///
    /// Boundary vertices, including vertices on hole boundaries, remain fixed.
    /// The operation is performed entirely by the integer mesh and then mapped
    /// back to `P` when points are requested.
    #[must_use]
    #[inline]
    pub fn relax(mut self, options: RelaxationOptions<P::Scalar>) -> Self {
        self.relax_mut(options);
        self
    }

    /// In-place version of [`Delaunay::relax`].
    #[inline]
    pub fn relax_mut(&mut self, options: RelaxationOptions<P::Scalar>) -> RelaxationResult {
        assert!(
            options.tolerance.is_finite() && options.tolerance >= P::Scalar::ZERO,
            "tolerance must be finite and non-negative"
        );

        let tolerance = self.adapter.round_len_to_int(options.tolerance).to_uint();
        self.delaunay.relax_mut(IntRelaxationOptions {
            max_iterations: options.max_iterations,
            tolerance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RelaxationOptions;
    use crate::float::triangulatable::Triangulatable;
    use crate::float::uniform::UniformTriangulatable;

    #[test]
    fn large_float_tolerance_stops_before_moving() {
        let contour = [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
        let steiner = [[25.0, 35.0], [72.0, 42.0], [43.0, 79.0]];
        let mut delaunay = contour
            .triangulate_with_steiner_points(&steiner)
            .into_delaunay();
        let points = delaunay.points();

        let result = delaunay.relax_mut(RelaxationOptions::new(10).with_tolerance(1_000.0));

        assert_eq!(result.iterations, 0);
        assert!(result.converged);
        assert_eq!(delaunay.points(), points);
    }

    #[test]
    fn float_wrapper_moves_interior_and_keeps_boundary_fixed() {
        let contour = [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
        let steiner = [[25.0, 35.0], [72.0, 42.0], [43.0, 79.0]];
        let mut delaunay = contour
            .triangulate_with_steiner_points(&steiner)
            .into_delaunay();
        let points = delaunay.points();
        let boundary: alloc::vec::Vec<_> = points
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                (point[0] == 0.0 || point[0] == 100.0) && (point[1] == 0.0 || point[1] == 100.0)
            })
            .map(|(index, _)| index)
            .collect();

        let result = delaunay.relax_mut(RelaxationOptions::new(2));
        let relaxed = delaunay.points();

        assert_eq!(result.iterations, 2);
        assert!(!result.converged);
        for &index in &boundary {
            assert_eq!(relaxed[index], points[index]);
        }
        assert!(relaxed
            .iter()
            .zip(&points)
            .enumerate()
            .any(|(index, (a, b))| !boundary.contains(&index) && a != b));
        delaunay
            .to_triangulation::<u32>()
            .validate(10_000.0, 0.000_001);
    }

    #[test]
    #[should_panic(expected = "tolerance must be finite and non-negative")]
    fn rejects_negative_tolerance() {
        let contour = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let mut delaunay = contour.triangulate().into_delaunay();

        delaunay.relax_mut(RelaxationOptions::new(1).with_tolerance(-1.0));
    }

    #[test]
    fn relaxes_uniform_mesh_from_self_intersecting_contour() {
        struct RelaxationStats {
            iterations: usize,
            converged: bool,
        }

        let shape = [[0.0, 0.0], [100.0, 100.0], [0.0, 100.0], [100.0, 0.0]];
        let edge_length = 10.0;
        let relax_enabled = true;
        let relax_iterations = 4;

        let mut delaunay = shape.uniform_triangulate(edge_length);
        let relaxation = relax_enabled.then(|| {
            let result = delaunay.relax_mut(RelaxationOptions::new(relax_iterations));
            RelaxationStats {
                iterations: result.iterations,
                converged: result.converged,
            }
        });

        let relaxation = relaxation.unwrap();
        assert!(relaxation.iterations <= relax_iterations);
        assert_eq!(
            relaxation.converged,
            relaxation.iterations < relax_iterations
        );
        delaunay
            .to_triangulation::<u32>()
            .validate(5_000.0, 0.000_001);
    }
}
