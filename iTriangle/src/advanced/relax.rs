use crate::advanced::buffer::DelaunayBuffer;
use crate::advanced::delaunay::{DelaunayRefine, IntDelaunay};
use crate::geom::triangle::IntTriangle;
use alloc::vec;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::signed_product::SignedProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_float::triangle::Triangle;

/// Configuration for centroid-net relaxation of an integer Delaunay mesh.
///
/// Boundary vertices, including vertices on hole boundaries, remain fixed.
/// Interior vertices move synchronously toward the area centroids of their
/// centroid-net cells. Every displacement is conservatively limited to less
/// than one quarter of the minimum altitude of each incident triangle before
/// the Delaunay property is restored with edge flips.
#[derive(Clone, Copy)]
pub struct RelaxationOptions<I: IntNumber> {
    /// Maximum number of relaxation iterations.
    pub max_iterations: usize,

    /// Absolute convergence tolerance in input coordinate units.
    ///
    /// Relaxation stops before the next move when every unconstrained
    /// centroid displacement is no greater than this value. Set it to zero to
    /// stop only when integer rounding leaves no vertex to move.
    pub tolerance: I::WideUInt,
}

impl<I: IntNumber> RelaxationOptions<I> {
    /// Creates options with zero convergence tolerance.
    #[inline]
    pub fn new(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            tolerance: I::WideUInt::ZERO,
        }
    }

    /// Sets the absolute convergence tolerance.
    #[inline]
    pub fn with_tolerance(mut self, tolerance: I::WideUInt) -> Self {
        self.tolerance = tolerance;
        self
    }
}

impl<I: IntNumber> Default for RelaxationOptions<I> {
    #[inline]
    fn default() -> Self {
        Self::new(8)
    }
}

/// Outcome of an in-place relaxation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelaxationResult {
    /// Number of completed vertex-movement iterations.
    pub iterations: usize,

    /// `true` when relaxation stopped by tolerance or because no integer
    /// vertex position changed; `false` when it reached the iteration limit.
    pub converged: bool,
}

struct RelaxationBuffer<I: IntNumber> {
    fixed: Vec<bool>,
    starts: Vec<usize>,
    proposals: Vec<IntPoint<I>>,
    order: Vec<usize>,
    cell: Vec<IntPoint<I>>,
    ring: Vec<usize>,
    delaunay: DelaunayBuffer,
}

impl<I: IntNumber> RelaxationBuffer<I> {
    fn new(delaunay: &IntDelaunay<I>) -> Self {
        let point_count = delaunay.points.len();
        let mut fixed = vec![false; point_count];

        for triangle in &delaunay.triangles {
            for edge in 0..3 {
                if triangle.neighbors[edge] >= delaunay.triangles.len() {
                    fixed[triangle.vertices[(edge + 1) % 3].index] = true;
                    fixed[triangle.vertices[(edge + 2) % 3].index] = true;
                }
            }
        }

        Self {
            fixed,
            starts: vec![usize::MAX; point_count],
            proposals: delaunay.points.clone(),
            order: (0..point_count).collect(),
            cell: Vec::with_capacity(16),
            ring: Vec::with_capacity(8),
            delaunay: DelaunayBuffer::new(),
        }
    }

    fn rebuild_starts(&mut self, triangles: &[IntTriangle<I>]) {
        self.starts.fill(usize::MAX);
        for (triangle_index, triangle) in triangles.iter().enumerate() {
            for vertex in &triangle.vertices {
                self.starts[vertex.index] = triangle_index;
            }
        }
    }
}

impl<I: IntNumber> IntDelaunay<I> {
    /// Relaxes interior vertices toward their centroid-net area centroids.
    ///
    /// Boundary vertices are detected from triangle edges without a neighbor
    /// and remain fixed. Vertex indices and the number of vertices are
    /// preserved. The Delaunay property is restored with edge flips after each
    /// synchronous move.
    ///
    /// Internal constrained edges are not retained because [`IntDelaunay`]
    /// does not store which interior edges are constraints.
    #[must_use]
    #[inline]
    pub fn relax(mut self, options: RelaxationOptions<I>) -> Self {
        self.relax_mut(options);
        self
    }

    /// In-place version of [`IntDelaunay::relax`].
    pub fn relax_mut(&mut self, options: RelaxationOptions<I>) -> RelaxationResult {
        if options.max_iterations == 0 {
            return RelaxationResult {
                iterations: 0,
                converged: false,
            };
        }

        debug_assert_positive_areas(&self.triangles);

        let mut buffer = RelaxationBuffer::new(self);

        for iteration in 0..options.max_iterations {
            buffer.rebuild_starts(&self.triangles);
            buffer.proposals.clone_from(&self.points);

            let mut within_tolerance = true;

            for vertex_index in 0..self.points.len() {
                if buffer.fixed[vertex_index] {
                    continue;
                }

                let start = buffer.starts[vertex_index];
                if start >= self.triangles.len()
                    || !self.collect_centroid_cell(
                        vertex_index,
                        start,
                        &mut buffer.cell,
                        &mut buffer.ring,
                    )
                {
                    continue;
                }

                let point = self.points[vertex_index];
                let Some(centroid) = polygon_centroid(&buffer.cell, point) else {
                    continue;
                };

                let displacement = centroid - point;
                let sqr_distance = displacement.sqr_length();
                if !is_within_tolerance::<I>(sqr_distance, options.tolerance) {
                    within_tolerance = false;
                }

                buffer.proposals[vertex_index] =
                    self.limit_displacement(point, displacement, &buffer.ring);
            }

            if within_tolerance {
                return RelaxationResult {
                    iterations: iteration,
                    converged: true,
                };
            }

            resolve_collisions(
                &self.points,
                &mut buffer.proposals,
                &buffer.fixed,
                &mut buffer.order,
            );

            if buffer.proposals == self.points {
                return RelaxationResult {
                    iterations: iteration,
                    converged: true,
                };
            }

            self.points.clone_from(&buffer.proposals);
            self.sync_triangle_points();

            debug_assert_positive_areas(&self.triangles);

            self.triangles.build_with_buffer(&mut buffer.delaunay);

            debug_assert_positive_areas(&self.triangles);
        }

        RelaxationResult {
            iterations: options.max_iterations,
            converged: false,
        }
    }

    fn collect_centroid_cell(
        &self,
        vertex_index: usize,
        start: usize,
        cell: &mut Vec<IntPoint<I>>,
        ring: &mut Vec<usize>,
    ) -> bool {
        cell.clear();
        ring.clear();

        let mut triangle_index = start;
        loop {
            if triangle_index >= self.triangles.len() || ring.len() >= self.triangles.len() {
                cell.clear();
                ring.clear();
                return false;
            }

            let triangle = &self.triangles[triangle_index];
            debug_assert!(triangle
                .vertices
                .iter()
                .any(|vertex| vertex.index == vertex_index));

            let (next, mid) = triangle.left_neighbor_and_mid_edge(vertex_index);
            ring.push(triangle_index);
            cell.push(triangle.center());
            cell.push(mid);

            if next == start {
                return true;
            }
            if next >= self.triangles.len() {
                cell.clear();
                ring.clear();
                return false;
            }
            triangle_index = next;
        }
    }

    fn limit_displacement(
        &self,
        point: IntPoint<I>,
        displacement: IntVector<I>,
        ring: &[usize],
    ) -> IntPoint<I> {
        let mut dx = displacement.x;
        let mut dy = displacement.y;

        while dx != I::Wide::ZERO || dy != I::Wide::ZERO {
            let candidate_displacement = IntVector::<I>::new(dx, dy);
            if ring.iter().all(|&triangle_index| {
                displacement_is_safe(candidate_displacement, &self.triangles[triangle_index])
            }) {
                break;
            }

            dx = dx / I::Wide::TWO;
            dy = dy / I::Wide::TWO;
        }

        IntPoint::new(
            I::from_wide(point.x.to_wide() + dx),
            I::from_wide(point.y.to_wide() + dy),
        )
    }

    fn sync_triangle_points(&mut self) {
        for triangle in &mut self.triangles {
            for vertex in &mut triangle.vertices {
                debug_assert!(vertex.index < self.points.len());
                vertex.point = self.points[vertex.index];
            }
        }
    }
}

fn polygon_centroid<I: IntNumber>(
    polygon: &[IntPoint<I>],
    origin: IntPoint<I>,
) -> Option<IntPoint<I>> {
    if polygon.len() < 3 {
        return None;
    }

    let mut area_two = I::Wide::ZERO;
    let mut x_numerator = SignedProduct::<I::Wide>::multiply(I::Wide::ZERO, I::Wide::ZERO);
    let mut y_numerator = SignedProduct::<I::Wide>::multiply(I::Wide::ZERO, I::Wide::ZERO);

    for index in 0..polygon.len() {
        let a = polygon[index] - origin;
        let b = polygon[(index + 1) % polygon.len()] - origin;
        let cross = a.cross_product(b);

        area_two = area_two + cross;
        x_numerator = x_numerator.checked_add(SignedProduct::multiply(a.x + b.x, cross))?;
        y_numerator = y_numerator.checked_add(SignedProduct::multiply(a.y + b.y, cross))?;
    }

    if area_two <= I::Wide::ZERO {
        return None;
    }

    let three = I::WideUInt::from_u64(3);
    let area = area_two.to_uint();
    if area > (I::WideUInt::LAST_BIT - I::WideUInt::ONE) / three {
        return None;
    }
    let divisor = area * three;

    let dx = divide_signed_product(x_numerator, divisor)?;
    let dy = divide_signed_product(y_numerator, divisor)?;

    Some(IntPoint::new(
        I::from_wide(origin.x.to_wide() + dx),
        I::from_wide(origin.y.to_wide() + dy),
    ))
}

fn divide_signed_product<I: WideIntNumber>(value: SignedProduct<I>, divisor: I::UInt) -> Option<I> {
    debug_assert!(divisor > I::UInt::ZERO);
    debug_assert!(divisor < I::UInt::LAST_BIT);

    let magnitude = value.magnitude().divide_with_rounding(divisor);
    if magnitude >= I::UInt::LAST_BIT {
        return None;
    }

    let result = I::from_uint(magnitude);
    Some(if value.is_negative() { -result } else { result })
}

fn displacement_is_safe<I: IntNumber>(
    displacement: IntVector<I>,
    triangle: &IntTriangle<I>,
) -> bool {
    if displacement.x == I::Wide::ZERO && displacement.y == I::Wide::ZERO {
        return true;
    }

    let sqr_displacement = displacement.sqr_length();
    if sqr_displacement <= I::Wide::ZERO {
        return false;
    }

    let a = triangle.vertices[0].point;
    let b = triangle.vertices[1].point;
    let c = triangle.vertices[2].point;

    let area_two = Triangle::area_two(a, b, c);
    debug_assert!(area_two > I::Wide::ZERO);
    if area_two <= I::Wide::ZERO {
        return false;
    }

    let max_sqr_edge = a
        .sqr_distance(b)
        .max(b.sqr_distance(c))
        .max(c.sqr_distance(a));
    if max_sqr_edge <= I::Wide::ZERO {
        return false;
    }

    let left = <I::WideUInt as UIntNumber>::Product::multiply(
        sqr_displacement.to_uint(),
        max_sqr_edge.to_uint(),
    );

    // |d| < h_min / 4, where h_min = area_two / longest_edge.
    // Squaring and rearranging avoids both division and square roots:
    // 16 * |d|^2 * longest_edge^2 < area_two^2.
    let Some(left2) = left.checked_add(left) else {
        return false;
    };
    let Some(left4) = left2.checked_add(left2) else {
        return false;
    };
    let Some(left8) = left4.checked_add(left4) else {
        return false;
    };
    let Some(left16) = left8.checked_add(left8) else {
        return false;
    };

    let area = area_two.to_uint();
    let right = <I::WideUInt as UIntNumber>::Product::multiply(area, area);

    left16 < right
}

fn is_within_tolerance<I: IntNumber>(sqr_distance: I::Wide, tolerance: I::WideUInt) -> bool {
    if sqr_distance < I::Wide::ZERO {
        return false;
    }

    let distance = <I::WideUInt as UIntNumber>::Product::from_uint(sqr_distance.to_uint());
    let tolerance = <I::WideUInt as UIntNumber>::Product::multiply(tolerance, tolerance);
    distance <= tolerance
}

fn resolve_collisions<I: IntNumber>(
    points: &[IntPoint<I>],
    proposals: &mut [IntPoint<I>],
    fixed: &[bool],
    order: &mut [usize],
) {
    loop {
        // Topology normalization may represent regions touching at a former
        // self-intersection with distinct vertex indices at the same point.
        // Preserve those existing coincidences and reject only newly merged
        // coordinates.
        order.sort_unstable_by_key(|&index| (proposals[index], points[index]));
        if !order.windows(2).any(|pair| {
            proposals[pair[0]] == proposals[pair[1]] && points[pair[0]] != points[pair[1]]
        }) {
            return;
        }

        // Back off every movable proposal together so the update remains
        // synchronous and deterministic.
        let mut changed = false;
        for index in 0..proposals.len() {
            if fixed[index] {
                continue;
            }

            let point = points[index];
            let displacement = proposals[index] - point;
            let next = IntPoint::new(
                I::from_wide(point.x.to_wide() + displacement.x / I::Wide::TWO),
                I::from_wide(point.y.to_wide() + displacement.y / I::Wide::TWO),
            );
            changed |= next != proposals[index];
            proposals[index] = next;
        }

        if !changed {
            proposals.clone_from_slice(points);
            return;
        }
    }
}

#[inline]
fn debug_assert_positive_areas<I: IntNumber>(_triangles: &[IntTriangle<I>]) {
    #[cfg(debug_assertions)]
    for triangle in _triangles {
        debug_assert!(
            Triangle::area_two(
                triangle.vertices[0].point,
                triangle.vertices[1].point,
                triangle.vertices[2].point,
            ) > I::Wide::ZERO
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{RelaxationOptions, RelaxationResult};
    use crate::advanced::delaunay::DelaunayCondition;
    use crate::int::triangulatable::IntTriangulatable;
    use crate::int::uniform::IntUniformTriangulatable;
    use alloc::vec;
    use alloc::vec::Vec;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn zero_iterations_does_not_change_mesh() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];
        let mut delaunay = contour.uniform_triangulate(20u64);
        let points = delaunay.points.clone();

        let result = delaunay.relax_mut(RelaxationOptions::new(0));

        assert_eq!(
            result,
            RelaxationResult {
                iterations: 0,
                converged: false,
            }
        );
        assert_eq!(delaunay.points, points);
    }

    #[test]
    fn large_tolerance_stops_before_moving() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];
        let mut delaunay = contour
            .triangulate_with_steiner_points(&[
                IntPoint::new(25, 35),
                IntPoint::new(72, 42),
                IntPoint::new(43, 79),
            ])
            .into_delaunay();
        let points = delaunay.points.clone();

        let result = delaunay.relax_mut(RelaxationOptions::new(10).with_tolerance(1_000u64));

        assert_eq!(result.iterations, 0);
        assert!(result.converged);
        assert_eq!(delaunay.points, points);
    }

    #[test]
    fn moves_interior_vertices_and_keeps_boundary_fixed() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];
        let mut delaunay = contour
            .triangulate_with_steiner_points(&[
                IntPoint::new(25, 35),
                IntPoint::new(72, 42),
                IntPoint::new(43, 79),
            ])
            .into_delaunay();
        let points = delaunay.points.clone();
        let boundary: Vec<_> = points
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                (point.x == 0 || point.x == 100) && (point.y == 0 || point.y == 100)
            })
            .map(|(index, _)| index)
            .collect();

        let result = delaunay.relax_mut(RelaxationOptions::new(2));

        assert_eq!(result.iterations, 2);
        assert!(!result.converged);
        for &index in &boundary {
            assert_eq!(delaunay.points[index], points[index]);
        }
        assert!(delaunay
            .points
            .iter()
            .zip(&points)
            .enumerate()
            .any(|(index, (a, b))| !boundary.contains(&index) && a != b));
        assert_is_delaunay(&delaunay);
    }

    #[test]
    fn keeps_outer_and_hole_boundaries_fixed() {
        let shape = vec![
            vec![
                IntPoint::new(0, 0),
                IntPoint::new(120, 0),
                IntPoint::new(120, 120),
                IntPoint::new(0, 120),
            ],
            vec![
                IntPoint::new(45, 45),
                IntPoint::new(45, 75),
                IntPoint::new(75, 75),
                IntPoint::new(75, 45),
            ],
        ];
        let mut delaunay = shape.uniform_triangulate(15u64);
        let points = delaunay.points.clone();
        let boundary: Vec<_> = points
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                point.x == 0
                    || point.x == 120
                    || point.y == 0
                    || point.y == 120
                    || ((point.x == 45 || point.x == 75) && (45..=75).contains(&point.y))
                    || ((point.y == 45 || point.y == 75) && (45..=75).contains(&point.x))
            })
            .map(|(index, _)| index)
            .collect();

        delaunay.relax_mut(RelaxationOptions::new(3));

        for index in boundary {
            assert_eq!(delaunay.points[index], points[index]);
        }
        assert_is_delaunay(&delaunay);
    }

    fn assert_is_delaunay<I: i_overlay::i_float::int::number::int::IntNumber>(
        delaunay: &crate::advanced::delaunay::IntDelaunay<I>,
    ) {
        for (triangle_index, triangle) in delaunay.triangles.iter().enumerate() {
            for &neighbor in &triangle.neighbors {
                if neighbor <= triangle_index || neighbor >= delaunay.triangles.len() {
                    continue;
                }

                let abc = triangle.abc_by_neighbor(neighbor);
                let pcb = delaunay.triangles[neighbor].abc_by_neighbor(triangle_index);
                assert!(DelaunayCondition::is_flip_not_required(
                    pcb.v0.vertex.point,
                    abc.v0.vertex.point,
                    abc.v1.vertex.point,
                    abc.v2.vertex.point,
                ));
            }
        }
    }
}
