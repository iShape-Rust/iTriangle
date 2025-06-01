use crate::float::triangulation::Triangulation;
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::triangulator::IntTriangulator;
use crate::int::validation::Validation;
use i_overlay::core::solver::Solver;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::source::resource::ShapeResource;

/// A reusable triangulator that converts float-based shapes into triangle meshes.
pub struct Triangulator<I> {
    flat_buffer: Option<FlatContoursBuffer>,
    int_buffer: Option<IntTriangulation<I>>,
    int_triangulator: IntTriangulator<I>,
}

impl<I: IndexType> Triangulator<I> {

    /// Enables or disables Delaunay refinement for triangulation.
    ///
    /// When enabled, the triangulator will attempt to generate a mesh that satisfies the
    /// Delaunay condition (no point lies inside the circumcircle of any triangle).
    ///
    /// This can improve triangle quality at the cost of slightly increased computation.
    pub fn delaunay(&mut self, enable: bool) {
        self.int_triangulator.delaunay = enable;
    }

    /// Returns whether Delaunay refinement is currently enabled.
    pub fn is_delaunay(&self) -> bool {
        self.int_triangulator.delaunay
    }

    /// Enables or disables Earcut64 optimization for small contours.
    ///
    /// When enabled, the triangulator will automatically use the highly optimized Earcut64
    /// algorithm for any contour with fewer or equal than 64 points. This reduces overhead
    /// for small polygons while maintaining correctness.
    pub fn earcut(&mut self, enable: bool) {
        self.int_triangulator.earcut = enable;
    }

    /// Returns whether Earcut64 optimization is currently enabled.
    pub fn is_earcut(&self) -> bool {
        self.int_triangulator.earcut
    }

    /// Performs triangulation on the given shape resource and returns a new `Triangulation`.
    ///
    /// - `resource`: A shape container implementing `ShapeResource` (e.g., contour, contours, or shapes).
    /// - `delaunay`: Enables Delaunay-compatible triangulation when set to true.
    /// - Returns: A new `Triangulation` containing triangle indices and corresponding points.
    ///
    /// Uses internal buffers to minimize allocations and speed up repeated calls.
    #[inline]
    pub fn new(max_points_count: usize, validation: Validation, solver: Solver) -> Self {
        Self {
            flat_buffer: Some(FlatContoursBuffer::with_capacity(max_points_count)),
            int_buffer: Some(IntTriangulation::with_capacity(max_points_count)),
            int_triangulator: IntTriangulator::new(max_points_count, validation, solver),
        }
    }
}

impl<I: IndexType> Default for Triangulator<I> {
    #[inline]
    fn default() -> Self {
        Self::new(64, Default::default(), Default::default())
    }
}

impl<I: IndexType> Triangulator<I> {

    /// Performs triangulation on the provided shape resource and returns a new `Triangulation`.
    ///
    /// - `resource`: A `ShapeResource` that define contours.
    ///   `ShapeResource` can be one of the following:
    ///     - `Contour`: A contour representing a closed path. This path is interpreted as closed, so it doesn’t require the start and endpoint to be the same for processing.
    ///     - `Contours`: A collection of contours, each representing a closed path.
    ///     - `Shapes`: A collection of shapes, where each shape may consist of multiple contours.
    ///
    /// Uses internal buffers to reduce allocations and preserve performance.
    #[inline]
    pub fn triangulate<R, P, T>(&mut self, resource: &R) -> Triangulation<P, I>
    where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, &mut int_buffer);

        let triangulation = int_buffer.to_float(&adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);

        triangulation
    }

    /// Triangulates the given shape resource and stores the result into an existing `Triangulation` instance.
    ///
    /// Avoids allocating a new `Triangulation` by reusing the provided output buffer.
    ///
    /// - `resource`: A `ShapeResource` that define contours.
    ///   `ShapeResource` can be one of the following:
    ///     - `Contour`: A contour representing a closed path. This path is interpreted as closed, so it doesn’t require the start and endpoint to be the same for processing.
    ///     - `Contours`: A collection of contours, each representing a closed path.
    ///     - `Shapes`: A collection of shapes, where each shape may consist of multiple contours.
    /// - `triangulation`: Output buffer to store the resulting triangle mesh.
    ///
    /// Uses internal buffers to reduce allocations and preserve performance.
    #[inline]
    pub fn triangulate_into<R, P, T>(
        &mut self,
        resource: &R,
        triangulation: &mut Triangulation<P, I>,
    ) where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, &mut int_buffer);

        triangulation.set_with_int(&int_buffer, &adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);
    }

    /// Performs triangulation on the provided shape resource and returns a new `Triangulation`.
    ///
    /// Skips input validation (e.g., area checks or self-intersections), offering faster performance
    /// at the cost of issues.
    ///
    /// - `resource`: A `ShapeResource` that define contours.
    ///   `ShapeResource` can be one of the following:
    ///     - `Contour`: A contour representing a closed path. This path is interpreted as closed, so it doesn’t require the start and endpoint to be the same for processing.
    ///     - `Contours`: A collection of contours, each representing a closed path.
    ///     - `Shapes`: A collection of shapes, where each shape may consist of multiple contours.
    /// - `delaunay`: if true, applies Delaunay refinement.
    ///
    /// Uses internal buffers to reduce allocations and preserve performance.
    #[inline]
    pub fn uncheck_triangulate<R, P, T>(
        &mut self,
        resource: &R,
    ) -> Triangulation<P, I>
    where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .uncheck_triangulate_flat_into(&flat_buffer, &mut int_buffer);

        let triangulation = int_buffer.to_float(&adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);

        triangulation
    }

    /// Triangulates the given shape resource and stores the result into an existing `Triangulation` instance.
    ///
    /// Skips input validation (e.g., area checks or self-intersections), offering faster performance
    /// at the cost of issues.
    /// 
    /// Avoids allocating a new `Triangulation` by reusing the provided output buffer.
    ///
    /// - `resource`: A `ShapeResource` that define contours.
    ///   `ShapeResource` can be one of the following:
    ///     - `Contour`: A contour representing a closed path. This path is interpreted as closed, so it doesn’t require the start and endpoint to be the same for processing.
    ///     - `Contours`: A collection of contours, each representing a closed path.
    ///     - `Shapes`: A collection of shapes, where each shape may consist of multiple contours.
    /// - `triangulation`: Output buffer to store the resulting triangle mesh.
    ///
    /// Uses internal buffers to reduce allocations and preserve performance.
    #[inline]
    pub fn uncheck_triangulate_into<R, P, T>(
        &mut self,
        resource: &R,
        triangulation: &mut Triangulation<P, I>,
    ) where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .uncheck_triangulate_flat_into(&flat_buffer, &mut int_buffer);

        triangulation.set_with_int(&int_buffer, &adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);
    }
}
