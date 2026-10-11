use crate::float::delaunay::Delaunay;
use crate::float::triangulation::Triangulation;
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::triangulator::IntTriangulator;
use crate::int::validation::Validation;
use i_overlay::core::integer::OverlayInt;
use i_overlay::core::solver::Solver;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::source::float::resource::ShapeResource;

/// A reusable triangulator that converts float-based shapes into triangle meshes.
pub struct Triangulator<N = u16, I = i32>
where
    I: OverlayInt,
{
    flat_buffer: Option<FlatContoursBuffer<I>>,
    int_buffer: Option<IntTriangulation<I, N>>,
    int_triangulator: IntTriangulator<I, N>,
}

impl<N, I> Triangulator<N, I>
where
    I: OverlayInt,
    N: IndexType,
{
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
    pub fn new(max_points_count: usize, validation: Validation<I>, solver: Solver) -> Self {
        Self {
            flat_buffer: Some(FlatContoursBuffer::with_capacity(max_points_count)),
            int_buffer: Some(IntTriangulation::with_capacity(max_points_count)),
            int_triangulator: IntTriangulator::new(max_points_count, validation, solver),
        }
    }
}

impl<N, I> Default for Triangulator<N, I>
where
    I: OverlayInt,
    N: IndexType,
{
    #[inline]
    fn default() -> Self {
        Self::new(64, Default::default(), Default::default())
    }
}

impl<N, I> Triangulator<N, I>
where
    I: OverlayInt,
    N: IndexType,
{
    /// Simplifies a resource and fills a Delaunay mesh
    #[inline]
    pub fn delaunay_into<R, P>(&mut self, resource: &R, output: &mut Delaunay<P, I>)
    where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let flat_buffer = self.flat_buffer.get_or_insert_with(Default::default);
        let adapter = flat_buffer
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");

        self.int_triangulator
            .delaunay_flat_into(flat_buffer, &mut output.delaunay);
        output.adapter = adapter;
    }

    /// Fills a Delaunay mesh from a prepared resource
    #[inline]
    pub fn uncheck_delaunay_into<R, P>(&mut self, resource: &R, output: &mut Delaunay<P, I>)
    where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let flat_buffer = self.flat_buffer.get_or_insert_with(Default::default);
        let adapter = flat_buffer
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");
        debug_assert!(!flat_buffer.is_empty());

        self.int_triangulator
            .uncheck_delaunay_flat_into(flat_buffer, &mut output.delaunay);
        output.adapter = adapter;
    }

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
    pub fn triangulate<R, P>(&mut self, resource: &R) -> Triangulation<P, N>
    where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let adapter = self
            .flat_buffer
            .get_or_insert_with(Default::default)
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

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
    pub fn triangulate_into<R, P>(&mut self, resource: &R, triangulation: &mut Triangulation<P, N>)
    where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let adapter = self
            .flat_buffer
            .get_or_insert_with(Default::default)
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

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
    pub fn uncheck_triangulate<R, P>(&mut self, resource: &R) -> Triangulation<P, N>
    where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let adapter = self
            .flat_buffer
            .get_or_insert_with(Default::default)
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");
        let flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

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
    pub fn uncheck_triangulate_into<R, P>(
        &mut self,
        resource: &R,
        triangulation: &mut Triangulation<P, N>,
    ) where
        R: ShapeResource<P> + ?Sized,
        P: FloatPointCompatible,
    {
        let adapter = self
            .flat_buffer
            .get_or_insert_with(Default::default)
            .set_with_resource(resource)
            .expect("Invalid triangulation bounds");
        let flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

        self.int_triangulator
            .uncheck_triangulate_flat_into(&flat_buffer, &mut int_buffer);

        triangulation.set_with_int(&int_buffer, &adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::Triangulator;
    use crate::float::delaunay::Delaunay;
    use crate::float::triangulation::Triangulation;
    use alloc::{vec, vec::Vec};
    use i_overlay::core::integer::OverlayInt;
    use i_overlay::i_shape::flat::float::{FloatFlatContoursBuffer, FloatFlatShapesBuffer};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn delaunay_reuses_buffers_and_updates_the_adapter() {
        fn check<I: OverlayInt>() {
            let large = [
                [0., 0.],
                [2., 0.],
                [4., 0.],
                [4., 2.],
                [4., 4.],
                [2., 4.],
                [0., 4.],
                [0., 2.],
            ];
            let shifted = [[100., 200.], [108., 200.], [100., 204.]];
            let mut triangulator = Triangulator::<u32, I>::default();
            let mut mesh = Delaunay::<[f64; 2], I>::default();
            assert!(mesh.points().is_empty());
            assert!(mesh.triangle_indices::<u32>().is_empty());
            let mut render = Triangulation::<_, u32>::with_capacity(0);
            triangulator.uncheck_delaunay_into(&large, &mut mesh);
            render.set_with_delaunay(&mesh);
            let pointers = (
                mesh.delaunay.points.as_ptr(),
                mesh.delaunay.triangles.as_ptr(),
            );
            let capacities = (
                mesh.delaunay.points.capacity(),
                mesh.delaunay.triangles.capacity(),
            );
            let render_pointers = (render.points.as_ptr(), render.indices.as_ptr());
            let flat = triangulator.flat_buffer.as_ref().unwrap();
            let flat_pointers = (flat.points.as_ptr(), flat.ranges.as_ptr());
            for earcut in [false, true] {
                triangulator.earcut(earcut);
                for checked in [false, true] {
                    for input in [shifted.as_slice(), large.as_slice()] {
                        if checked {
                            triangulator.delaunay_into(input, &mut mesh);
                        } else {
                            triangulator.uncheck_delaunay_into(input, &mut mesh);
                        }
                        render.set_with_delaunay(&mesh);
                        render.validate(16., 0.000_001);
                        for point in &render.points {
                            assert!(input.contains(point));
                        }
                        assert_eq!(
                            capacities,
                            (
                                mesh.delaunay.points.capacity(),
                                mesh.delaunay.triangles.capacity()
                            )
                        );
                        assert_eq!(
                            pointers,
                            (
                                mesh.delaunay.points.as_ptr(),
                                mesh.delaunay.triangles.as_ptr()
                            )
                        );
                        assert_eq!(
                            render_pointers,
                            (render.points.as_ptr(), render.indices.as_ptr())
                        );
                        let flat = triangulator.flat_buffer.as_ref().unwrap();
                        assert_eq!(flat_pointers, (flat.points.as_ptr(), flat.ranges.as_ptr()));
                    }
                }
            }
            let mut other = Delaunay::default();
            let previous = mesh.points();
            triangulator.delaunay_into(&shifted, &mut other);
            assert_eq!(mesh.points(), previous);
        }
        check::<i16>();
        check::<i32>();
        check::<i64>();
    }

    fn validate(mesh: &Delaunay<[f64; 2]>, area: f64) {
        mesh.to_triangulation::<u32>().validate(area, 0.000_001);
        for (index, triangle) in mesh.delaunay.triangles.iter().enumerate() {
            for (side, &neighbor) in triangle.neighbors.iter().enumerate() {
                if neighbor == usize::MAX {
                    continue;
                }
                let other = &mesh.delaunay.triangles[neighbor];
                let other_side = other.neighbors.iter().position(|&n| n == index).unwrap();
                assert_eq!(
                    triangle.vertices[(side + 1) % 3].index,
                    other.vertices[(other_side + 2) % 3].index
                );
                assert_eq!(
                    triangle.vertices[(side + 2) % 3].index,
                    other.vertices[(other_side + 1) % 3].index
                );
            }
        }
        let convex_area: f64 = mesh
            .to_convex_polygons()
            .iter()
            .map(|polygon| {
                polygon
                    .iter()
                    .zip(polygon.iter().cycle().skip(1))
                    .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
                    .sum::<f64>()
                    * 0.5
            })
            .sum();
        assert!((convex_area - area).abs() < 0.000_001);
    }

    #[test]
    fn delaunay_accepts_shapes_holes_and_float_flat_resources() {
        let shape = vec![
            vec![[0., 0.], [20., 0.], [20., 20.], [0., 20.]],
            vec![[5., 5.], [5., 15.], [15., 15.], [15., 5.]],
        ];
        let shapes = vec![
            shape.clone(),
            vec![vec![[30., 0.], [40., 0.], [40., 10.], [30., 10.]]],
        ];
        let mut flat = FloatFlatContoursBuffer::default();
        flat.set_with_resource(&shapes);
        let mut flat_shapes = FloatFlatShapesBuffer::default();
        flat_shapes.set_with_resource(&shapes);
        let mut triangulator = Triangulator::<u32>::default();
        let mut mesh = Delaunay::default();
        for earcut in [false, true] {
            triangulator.earcut(earcut);
            triangulator.delaunay_into(&shape, &mut mesh);
            validate(&mesh, 300.);
            triangulator.uncheck_delaunay_into(&shape, &mut mesh);
            validate(&mesh, 300.);
            triangulator.delaunay_into(&shapes, &mut mesh);
            validate(&mesh, 400.);
            triangulator.uncheck_delaunay_into(&shapes, &mut mesh);
            validate(&mesh, 400.);
            triangulator.delaunay_into(&flat, &mut mesh);
            validate(&mesh, 400.);
            triangulator.uncheck_delaunay_into(&flat, &mut mesh);
            validate(&mesh, 400.);
            triangulator.delaunay_into(&flat_shapes, &mut mesh);
            validate(&mesh, 400.);
            triangulator.uncheck_delaunay_into(&flat_shapes, &mut mesh);
            validate(&mesh, 400.);
        }
    }

    #[test]
    fn checked_delaunay_simplifies_and_clears_empty_results() {
        use i_overlay::core::fill_rule::FillRule;
        let bowtie = [[0., 0.], [10., 10.], [0., 10.], [10., 0.]];
        let mut triangulator = Triangulator::<u32>::default();
        let mut mesh = Delaunay::default();
        let mut render = Triangulation::<_, u32>::with_capacity(0);
        triangulator.delaunay_into(&bowtie, &mut mesh);
        validate(&mesh, 50.);
        render.set_with_delaunay(&mesh);
        let capacities = (
            mesh.delaunay.points.capacity(),
            mesh.delaunay.triangles.capacity(),
        );
        let render_capacities = (render.points.capacity(), render.indices.capacity());
        let empty: Vec<Vec<Vec<[f64; 2]>>> = vec![vec![vec![]], vec![]];
        triangulator.delaunay_into(&empty, &mut mesh);
        assert!(mesh.delaunay.points.is_empty() && mesh.delaunay.triangles.is_empty());
        assert_eq!(
            capacities,
            (
                mesh.delaunay.points.capacity(),
                mesh.delaunay.triangles.capacity()
            )
        );
        render.set_with_delaunay(&mesh);
        assert!(render.points.is_empty() && render.indices.is_empty());
        assert_eq!(
            render_capacities,
            (render.points.capacity(), render.indices.capacity())
        );

        triangulator.delaunay_into(&bowtie, &mut mesh);
        triangulator.int_triangulator.fill_rule = FillRule::EvenOdd;
        triangulator.delaunay_into(&[bowtie.as_slice(), bowtie.as_slice()], &mut mesh);
        assert!(mesh.delaunay.points.is_empty() && mesh.delaunay.triangles.is_empty());
    }

    #[test]
    fn delaunay_f32_and_export_indices_are_independent_of_triangulator_indices() {
        let input = [[0_f32, 0.], [4., 0.], [4., 4.], [0., 4.]];
        let mut triangulator = Triangulator::<u8, i64>::default();
        let mut mesh = Delaunay::default();
        let mut render = Triangulation::<_, u32>::with_capacity(0);
        for earcut in [false, true] {
            triangulator.earcut(earcut);
            triangulator.uncheck_delaunay_into(&input, &mut mesh);
            render.set_with_delaunay(&mesh);
            render.validate(16., 0.000_001);
            triangulator.delaunay_into(&input, &mut mesh);
            render.set_with_delaunay(&mesh);
            render.validate(16., 0.000_001);
        }
    }

    #[test]
    fn delaunay_refines_without_enabling_the_flat_output_flag() {
        use crate::float::unchecked::UncheckedTriangulatable;
        let input = [[0., 0.], [10., 0.], [10., 6.], [0., 10.]];
        let raw = input.unchecked_triangulate();
        let before = raw.triangle_indices::<u32>();
        let expected = raw.into_delaunay();
        assert_ne!(before, expected.triangle_indices::<u32>());
        let mut triangulator = Triangulator::<u32>::default();
        let mut mesh = Delaunay::default();
        assert!(!triangulator.is_delaunay());
        triangulator.uncheck_delaunay_into(&input, &mut mesh);
        assert_eq!(
            mesh.triangle_indices::<u32>(),
            expected.triangle_indices::<u32>()
        );
        assert_eq!(mesh.triangle_neighbors(), expected.triangle_neighbors());
        validate(&mesh, 80.);
    }

    #[test]
    fn delaunay_export_rejects_indices_that_cannot_represent_the_mesh() {
        let input: Vec<_> = (0..=128)
            .map(|x| [x as f64, 0.])
            .chain((0..=128).rev().map(|x| [x as f64, 128.]))
            .collect();
        let mut triangulator = Triangulator::<u8>::default();
        let mut mesh = Delaunay::default();
        triangulator.uncheck_delaunay_into(&input, &mut mesh);
        assert_eq!(mesh.delaunay.points.len(), 258);
        let mut narrow = Triangulation::<_, u8>::with_capacity(0);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            narrow.set_with_delaunay(&mesh);
        }))
        .is_err());
        let mut render = Triangulation::<_, u32>::with_capacity(0);
        render.set_with_delaunay(&mesh);
        render.validate(16384., 0.000_001);
    }

    #[test]
    fn invalid_float_bounds_preserve_the_previous_delaunay() {
        let valid = [[0., 0.], [4., 0.], [0., 4.]];
        let invalid = [[f64::INFINITY, 0.], [4., 0.], [0., 4.]];
        let mut triangulator = Triangulator::<u32>::default();
        let mut mesh = Delaunay::default();
        triangulator.delaunay_into(&valid, &mut mesh);
        let points = mesh.points();
        let indices = mesh.triangle_indices::<u32>();
        for checked in [false, true] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                if checked {
                    triangulator.delaunay_into(&invalid, &mut mesh);
                } else {
                    triangulator.uncheck_delaunay_into(&invalid, &mut mesh);
                }
            }))
            .is_err());
            assert_eq!(mesh.points(), points);
            assert_eq!(mesh.triangle_indices::<u32>(), indices);
            triangulator.uncheck_delaunay_into(&valid, &mut mesh);
            validate(&mesh, 8.);
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn unchecked_delaunay_asserts_nonempty_contours_before_replacing_the_mesh() {
        let valid = vec![[0., 0.], [4., 0.], [0., 4.]];
        let mut triangulator = Triangulator::<u32>::default();
        let mut mesh = Delaunay::default();
        triangulator.uncheck_delaunay_into(&valid, &mut mesh);
        let points = mesh.points();
        let indices = mesh.triangle_indices::<u32>();
        for input in [
            vec![],
            vec![vec![]],
            vec![valid[..2].to_vec()],
            vec![valid, vec![]],
        ] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                triangulator.uncheck_delaunay_into(&input, &mut mesh);
            }))
            .is_err());
            assert_eq!(mesh.points(), points);
            assert_eq!(mesh.triangle_indices::<u32>(), indices);
        }
    }
}
