use i_triangle::float::relax::RelaxationOptions;
use i_triangle::float::uniform::UniformTriangulatable;
use i_triangle::i_overlay::core::integer::OverlayInt;

const DEFAULT_CASE_COUNT: usize = 3_000;
const CASE_COUNT_ENV: &str = "ITRIANGLE_RELAX_STRESS_CASES";

#[test]
#[ignore = "deterministic stress test; run with `cargo test --test relax_stress -- --ignored`"]
fn self_intersecting_uniform_relax_stress() {
    let case_count = std::env::var(CASE_COUNT_ENV)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_CASE_COUNT);

    for case in 0..case_count {
        match case % 3 {
            0 => run_case::<i16>(case),
            1 => run_case::<i32>(case),
            _ => run_case::<i64>(case),
        }
    }
}

fn run_case<I: OverlayInt>(case: usize) {
    let mut rng = Rng::new(case as u64 + 1);
    let vertex_count = [5, 7, 9, 11, 13][case % 5];
    let radius = 30.0 + 70.0 * rng.unit();
    let center = [200.0 * rng.signed_unit(), 200.0 * rng.signed_unit()];
    let angle_step = core::f64::consts::TAU / vertex_count as f64;

    let mut ring = Vec::with_capacity(vertex_count);
    for index in 0..vertex_count {
        let angle = index as f64 * angle_step + 0.16 * angle_step * rng.signed_unit();
        let local_radius = radius * (0.82 + 0.36 * rng.unit());
        ring.push([
            center[0] + local_radius * angle.cos(),
            center[1] + local_radius * angle.sin(),
        ]);
    }

    // Visiting every second point of an odd ring creates a single
    // self-intersecting star contour.
    let mut shape = Vec::with_capacity(vertex_count);
    let mut index = 0;
    for _ in 0..vertex_count {
        shape.push(ring[index]);
        index = (index + 2) % vertex_count;
    }

    let edge_length = radius / (4 + case % 6) as f64;
    let relax_iterations = 1 + case % 8;
    let mut delaunay = shape.uniform_triangulate_as::<I>(edge_length);

    let before = delaunay.points();
    let indices_before = delaunay.triangle_indices::<u32>();
    let neighbors_before = delaunay.triangle_neighbors();
    assert!(!indices_before.is_empty(), "case {case}: empty mesh");

    let boundary = boundary_vertices(before.len(), &indices_before, &neighbors_before);
    let area_before = mesh_area(case, &before, &indices_before);

    let result = delaunay.relax_mut(RelaxationOptions::new(relax_iterations));

    assert!(
        result.iterations <= relax_iterations,
        "case {case}: iteration limit exceeded"
    );
    assert_eq!(
        result.converged,
        result.iterations < relax_iterations,
        "case {case}: inconsistent convergence result"
    );

    let after = delaunay.points();
    let indices_after = delaunay.triangle_indices::<u32>();
    assert_eq!(
        after.len(),
        before.len(),
        "case {case}: point count changed"
    );
    assert_eq!(
        indices_after.len(),
        indices_before.len(),
        "case {case}: triangle count changed"
    );

    for (index, is_boundary) in boundary.into_iter().enumerate() {
        if is_boundary {
            assert_eq!(after[index], before[index], "case {case}: boundary moved");
        }
    }

    assert_no_new_collisions(case, &before, &after);

    let area_after = mesh_area(case, &after, &indices_after);
    let area_tolerance = 1.0e-10 * area_before.max(1.0);
    assert!(
        (area_after - area_before).abs() <= area_tolerance,
        "case {case}: area changed from {area_before} to {area_after}"
    );
}

fn boundary_vertices(point_count: usize, indices: &[u32], neighbors: &[[usize; 3]]) -> Vec<bool> {
    let triangle_count = indices.len() / 3;
    assert_eq!(neighbors.len(), triangle_count);

    let mut result = vec![false; point_count];
    for (triangle_index, triangle) in indices.chunks_exact(3).enumerate() {
        for edge in 0..3 {
            if neighbors[triangle_index][edge] >= triangle_count {
                result[triangle[(edge + 1) % 3] as usize] = true;
                result[triangle[(edge + 2) % 3] as usize] = true;
            }
        }
    }
    result
}

fn mesh_area(case: usize, points: &[[f64; 2]], indices: &[u32]) -> f64 {
    let mut area_two = 0.0;
    for triangle in indices.chunks_exact(3) {
        let a = points[triangle[0] as usize];
        let b = points[triangle[1] as usize];
        let c = points[triangle[2] as usize];
        let triangle_area_two = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(
            triangle_area_two > 0.0,
            "case {case}: non-positive triangle area"
        );
        area_two += triangle_area_two;
    }
    0.5 * area_two
}

fn assert_no_new_collisions(case: usize, before: &[[f64; 2]], after: &[[f64; 2]]) {
    for left in 0..after.len() {
        for right in left + 1..after.len() {
            assert!(
                after[left] != after[right] || before[left] == before[right],
                "case {case}: vertices {left} and {right} collided"
            );
        }
    }
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn unit(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    fn signed_unit(&mut self) -> f64 {
        2.0 * self.unit() - 1.0
    }
}
