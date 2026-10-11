use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::point::IntPoint;
use i_triangle::advanced::convex::ConvexSplitStrategy;
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
    validate_strategy(mesh, limit, ConvexSplitStrategy::Fan)
}

fn validate_strategy(
    mesh: &IntDelaunay<i32>,
    limit: usize,
    strategy: ConvexSplitStrategy,
) -> Vec<Vec<Point>> {
    let original = mesh.to_convex_polygons();
    let parts = mesh.to_convex_polygons_with_max_vertices(limit, strategy);
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
        // Each part stays inside one region; only Centroid may introduce a new vertex.
        assert!(original.iter().any(|region| {
            (strategy == ConvexSplitStrategy::Centroid || part.iter().all(|p| region.contains(p)))
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
            for strategy in [ConvexSplitStrategy::EarClipping, ConvexSplitStrategy::Fan] {
                let parts = validate_strategy(&mesh, limit, strategy);
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
        for strategy in [
            ConvexSplitStrategy::EarClipping,
            ConvexSplitStrategy::Fan,
            ConvexSplitStrategy::Centroid,
        ] {
            let parts = validate_strategy(&mesh, limit, strategy);
            assert_eq!(parts.iter().map(|p| area_two(p)).sum::<i64>(), 600);
        }
    }
}

#[test]
fn empty_mesh_and_limits_below_three() {
    let mesh = IntDelaunay::<i32> {
        triangles: Vec::new(),
        points: Vec::new(),
    };
    for limit in 0..=3 {
        for strategy in [
            ConvexSplitStrategy::EarClipping,
            ConvexSplitStrategy::Fan,
            ConvexSplitStrategy::Centroid,
        ] {
            assert!(mesh
                .to_convex_polygons_with_max_vertices(limit, strategy)
                .is_empty());
        }
    }

    let contour: Vec<_> = (0..11).map(|x| Point::new(x, x * x)).collect();
    let mesh = contour.triangulate().into_delaunay();
    for strategy in [ConvexSplitStrategy::EarClipping, ConvexSplitStrategy::Fan] {
        let triangles = validate_strategy(&mesh, 3, strategy);
        for limit in 0..3 {
            assert_eq!(
                mesh.to_convex_polygons_with_max_vertices(limit, strategy),
                triangles
            );
        }
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
    let parts = mesh.to_convex_polygons_with_max_vertices(6, ConvexSplitStrategy::Fan);
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
    let clipped = mesh.to_convex_polygons_with_max_vertices(6, ConvexSplitStrategy::default());
    assert_eq!(clipped.iter().map(Vec::len).collect::<Vec<_>>(), [5, 5, 5]);
    assert!((clipped.iter().map(area).sum::<f64>() - area(&original[0])).abs() < 1e-8);
    assert!(clipped.iter().flatten().all(|p| original[0].contains(p)));
    assert!(clipped
        .windows(2)
        .all(|parts| parts[0].last() == parts[1].first()));
    let centered = mesh.to_convex_polygons_with_max_vertices(6, ConvexSplitStrategy::Centroid);
    assert!(centered.iter().all(|p| (3..=6).contains(&p.len())));
    assert!(centered.iter().all(|p| p[0] == centered[0][0]));
    assert!(!original[0].contains(&centered[0][0]));
    assert!((centered.iter().map(area).sum::<f64>() - area(&original[0])).abs() < 1e-8);
    assert_eq!(
        mesh.to_convex_polygons_with_max_vertices(11, ConvexSplitStrategy::Centroid),
        original
    );
    assert_eq!(
        mesh.to_convex_polygons_with_max_vertices(11, ConvexSplitStrategy::Fan),
        original
    );
    let triangles = mesh.to_convex_polygons_with_max_vertices(3, ConvexSplitStrategy::Fan);
    assert!(triangles.iter().all(|p| p.len() == 3));
    for limit in 0..3 {
        assert_eq!(
            mesh.to_convex_polygons_with_max_vertices(limit, ConvexSplitStrategy::Fan),
            triangles
        );
    }
}

#[test]
fn centroid_parts_preserve_convexity_and_coverage() {
    for n in 4..=40 {
        let contour: Vec<_> = (0..n).map(|x| Point::new(10 * x, 10 * x * x)).collect();
        let mesh = contour.triangulate().into_delaunay();
        for limit in [3, 4, 6, 9, n as usize, usize::MAX] {
            let parts = validate_strategy(&mesh, limit, ConvexSplitStrategy::Centroid);
            if n as usize <= limit {
                assert_eq!(parts, mesh.to_convex_polygons());
            } else {
                assert!(parts.iter().all(|part| part[0] == parts[0][0]));
                assert!(!contour.contains(&parts[0][0]));
            }
        }
    }
}

#[test]
fn centroid_is_weighted_by_area_and_low_limits_are_clamped() {
    let contour = vec![
        Point::new(0, 0),
        Point::new(15, 0),
        Point::new(6, 6),
        Point::new(0, 6),
    ];
    let mesh = contour.triangulate().into_delaunay();
    let parts = validate_strategy(&mesh, 3, ConvexSplitStrategy::Centroid);
    assert_eq!(parts.len(), 4);
    // Area centroid is (39/7, 18/7); vertex average is (21/4, 3).
    assert!(parts.iter().all(|part| part[0] == Point::new(6, 3)));
    for limit in 0..3 {
        assert_eq!(
            mesh.to_convex_polygons_with_max_vertices(limit, ConvexSplitStrategy::Centroid),
            parts
        );
    }
}

#[test]
fn centroid_rounding_to_boundary_or_outside_falls_back_to_fan() {
    for contour in [
        vec![
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(0, 1),
        ],
        vec![
            Point::new(0, 0),
            Point::new(2, 3),
            Point::new(3, 5),
            Point::new(1, 2),
        ],
    ] {
        let mesh = contour.triangulate().into_delaunay();
        assert_eq!(
            validate_strategy(&mesh, 3, ConvexSplitStrategy::Centroid),
            validate(&mesh, 3)
        );
    }
}

#[test]
fn centroid_groups_sectors_without_reflex_angles() {
    let contour: Vec<_> = [
        [-3, -1],
        [-1, -3],
        [1, -3],
        [3, -1],
        [3, 1],
        [1, 3],
        [-1, 3],
        [-3, 1],
    ]
    .map(|[x, y]| Point::new(x, y))
    .into();
    let mesh = contour.triangulate().into_delaunay();
    let parts = validate_strategy(&mesh, 5, ConvexSplitStrategy::Centroid);
    assert_eq!(parts.iter().map(Vec::len).collect::<Vec<_>>(), [5, 5, 4]);
    assert!(parts.iter().all(|part| part[0] == Point::new(0, 0)));
    // Two equal groups would each have a flat angle at the center.
    let parts = validate_strategy(&mesh, 7, ConvexSplitStrategy::Centroid);
    assert!(parts.len() > 2);
}

#[test]
fn ear_clipping_is_default_and_moves_the_diagonal_origin() {
    assert_eq!(
        ConvexSplitStrategy::default(),
        ConvexSplitStrategy::EarClipping
    );
    let contour: Vec<_> = (0..12).map(|x| Point::new(x, x * x)).collect();
    let mesh = contour.triangulate().into_delaunay();
    let original = mesh.to_convex_polygons();
    let vertices = &original[0];
    let parts = validate_strategy(&mesh, 6, ConvexSplitStrategy::default());
    assert_eq!(
        parts,
        vec![
            vertices[0..=5].to_vec(),
            vertices[5..=9].to_vec(),
            vec![
                vertices[9],
                vertices[10],
                vertices[11],
                vertices[0],
                vertices[5]
            ],
        ]
    );
    // A triangle limit makes the ring wrap around more than once.
    let triangles = validate_strategy(&mesh, 3, ConvexSplitStrategy::EarClipping);
    assert!(triangles
        .windows(2)
        .all(|parts| parts[0].last() == parts[1].first()));
}
