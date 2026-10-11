use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::point::IntPoint;
use i_triangle::advanced::delaunay::IntDelaunay;
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::int::triangulatable::IntTriangulatable;

type Point = IntPoint<i32>;

fn cross(a: Point, b: Point, c: Point) -> i64 {
    (b.x as i64 - a.x as i64) * (c.y as i64 - a.y as i64)
        - (b.y as i64 - a.y as i64) * (c.x as i64 - a.x as i64)
}

fn area_two(polygon: &[Point]) -> i64 {
    (1..polygon.len() - 1)
        .map(|i| cross(polygon[0], polygon[i], polygon[i + 1]))
        .sum()
}

fn separated(a: &[Point], b: &[Point]) -> bool {
    (0..a.len()).any(|i| b.iter().all(|&p| cross(a[i], a[(i + 1) % a.len()], p) <= 0))
}

fn validate(mesh: &IntDelaunay<i32>, limit: usize) -> Vec<Vec<Point>> {
    let original = mesh.to_convex_polygons();
    let parts = mesh.to_convex_polygons_with_max_vertices(limit);
    assert_eq!(
        parts.iter().map(|p| area_two(p)).sum::<i64>(),
        original.iter().map(|p| area_two(p)).sum::<i64>()
    );
    for (i, part) in parts.iter().enumerate() {
        assert!((3..=limit).contains(&part.len()));
        for j in 0..part.len() {
            assert!(
                cross(
                    part[j],
                    part[(j + 1) % part.len()],
                    part[(j + 2) % part.len()]
                ) > 0
            );
        }
        // Each part stays inside one original convex region and uses its vertices.
        assert!(original.iter().any(|region| {
            part.iter().all(|p| region.contains(p))
                && (0..region.len()).all(|j| {
                    part.iter()
                        .all(|&p| cross(region[j], region[(j + 1) % region.len()], p) >= 0)
                })
        }));
        for other in &parts[..i] {
            assert!(separated(part, other) || separated(other, part));
        }
    }
    parts
}

#[test]
fn convex_contours_have_minimal_balanced_parts() {
    for n in (3..=40).chain([257]) {
        let contour: Vec<_> = (0..n).map(|x| Point::new(x, x * x)).collect();
        let mesh = contour.triangulate().into_delaunay();
        let original = mesh.to_convex_polygons();
        assert_eq!(original.len(), 1);
        assert_eq!(original[0].len(), n as usize);
        for limit in [3, 4, 5, 6, 9, n as usize, usize::MAX] {
            let parts = validate(&mesh, limit);
            assert_eq!(parts.len(), (n as usize - 2).div_ceil(limit - 2));
            let min = parts.iter().map(Vec::len).min().unwrap();
            let max = parts.iter().map(Vec::len).max().unwrap();
            assert!(max - min <= 1);
            if n as usize <= limit {
                assert_eq!(parts, original);
            }
        }
    }
}

#[test]
fn limit_applies_after_collinear_vertices_are_removed() {
    let contour = vec![
        Point::new(0, 0),
        Point::new(2, 0),
        Point::new(4, 0),
        Point::new(4, 2),
        Point::new(4, 4),
        Point::new(2, 4),
        Point::new(0, 4),
        Point::new(0, 2),
    ];
    let mesh = contour.triangulate().into_delaunay();
    let parts = validate(&mesh, 4);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].len(), 4);
    assert_eq!(parts, mesh.to_convex_polygons());
    assert_eq!(validate(&mesh, 3).len(), 2);
}

#[test]
fn decomposition_preserves_a_hole() {
    let outer: Vec<_> = [[0, 0], [20, 0], [20, 20], [0, 20]]
        .map(|[x, y]| Point::new(x, y))
        .into();
    let hole: Vec<_> = [[5, 5], [5, 15], [15, 15], [15, 5]]
        .map(|[x, y]| Point::new(x, y))
        .into();
    let mesh = vec![outer, hole].triangulate().into_delaunay();
    for limit in [3, 4, 6] {
        let parts = validate(&mesh, limit);
        assert_eq!(parts.iter().map(|p| area_two(p)).sum::<i64>(), 600);
    }
}

#[test]
fn empty_mesh_and_limits_below_three() {
    let mesh = IntDelaunay::<i32> {
        triangles: Vec::new(),
        points: Vec::new(),
    };
    for limit in 0..=3 {
        assert!(mesh.to_convex_polygons_with_max_vertices(limit).is_empty());
    }

    let contour: Vec<_> = (0..11).map(|x| Point::new(x, x * x)).collect();
    let mesh = contour.triangulate().into_delaunay();
    let triangles = validate(&mesh, 3);
    for limit in 0..3 {
        assert_eq!(mesh.to_convex_polygons_with_max_vertices(limit), triangles);
    }
}

#[test]
fn float_api_converts_the_integer_parts() {
    check_float::<i16>();
    check_float::<i32>();
    check_float::<i64>();
}

fn check_float<I: OverlayInt>() {
    let contour: Vec<[f64; 2]> = (0..11).map(|x| [x as f64, (x * x) as f64]).collect();
    let mesh = contour.triangulate_as::<I>().into_delaunay();
    let original = mesh.to_convex_polygons();
    assert_eq!(original.len(), 1);
    assert_eq!(original[0].len(), 11);
    let parts = mesh.to_convex_polygons_with_max_vertices(6);
    assert_eq!(parts.iter().map(Vec::len).collect::<Vec<_>>(), [5, 5, 5]);
    let area = |p: &Vec<[f64; 2]>| {
        (0..p.len())
            .map(|i| {
                let a = p[i];
                let b = p[(i + 1) % p.len()];
                a[0] * b[1] - a[1] * b[0]
            })
            .sum::<f64>()
    };
    assert!((parts.iter().map(area).sum::<f64>() - area(&original[0])).abs() < 1e-8);
    assert!(parts.iter().flatten().all(|p| original[0].contains(p)));
    assert_eq!(mesh.to_convex_polygons_with_max_vertices(11), original);
    let triangles = mesh.to_convex_polygons_with_max_vertices(3);
    assert!(triangles.iter().all(|p| p.len() == 3));
    for limit in 0..3 {
        assert_eq!(mesh.to_convex_polygons_with_max_vertices(limit), triangles);
    }
}
