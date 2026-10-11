use i_triangle::advanced::delaunay::IntDelaunay;
use i_triangle::i_overlay::core::fill_rule::FillRule;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::i_overlay::i_float::triangle::Triangle;
use i_triangle::i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_triangle::i_overlay::i_shape::int::shape::{IntContour, IntShape};
use i_triangle::int::triangulation::IntTriangulation;
use i_triangle::int::triangulator::IntTriangulator;
use i_triangle::int::unchecked::IntUncheckedTriangulatable;

fn contour<I: OverlayInt + From<i16>>(points: &[(i16, i16)]) -> IntContour<I> {
    points
        .iter()
        .map(|&(x, y)| IntPoint::new(I::from(x), I::from(y)))
        .collect()
}

fn empty_mesh<I: OverlayInt>() -> IntDelaunay<I> {
    IntDelaunay {
        points: Vec::new(),
        triangles: Vec::new(),
    }
}

fn validate<I: OverlayInt + From<i16> + core::fmt::Debug>(
    mesh: &IntDelaunay<I>,
    expected_area_two: i16,
) {
    let mut area = I::Wide::ZERO;
    for (index, triangle) in mesh.triangles.iter().enumerate() {
        let v = &triangle.vertices;
        let abc = Triangle::area_two(v[0].point, v[1].point, v[2].point);
        assert!(abc > I::Wide::ZERO);
        area = area + abc;
        for vertex in v {
            assert!(vertex.index < mesh.points.len());
            assert_eq!(mesh.points[vertex.index], vertex.point);
        }
        for (side, &neighbor) in triangle.neighbors.iter().enumerate() {
            if neighbor == usize::MAX {
                continue;
            }
            assert_ne!(neighbor, index);
            let other = &mesh.triangles[neighbor];
            let other_side = other.neighbors.iter().position(|&n| n == index).unwrap();
            assert_eq!(
                v[(side + 1) % 3].index,
                other.vertices[(other_side + 2) % 3].index
            );
            assert_eq!(
                v[(side + 2) % 3].index,
                other.vertices[(other_side + 1) % 3].index
            );
        }
    }
    assert!(area == I::from(expected_area_two).to_wide());
}

fn shape_with_hole() -> IntShape<i32> {
    vec![
        contour(&[(0, 0), (20, 0), (20, 20), (0, 20)]),
        contour(&[(5, 5), (5, 15), (15, 15), (15, 5)]),
    ]
}

#[test]
fn contour_reuses_buffers_across_sizes_algorithms_and_integer_types() {
    fn check<I: OverlayInt + From<i16> + core::fmt::Debug>() {
        let large = contour(&[
            (0, 0),
            (5, 0),
            (10, 0),
            (10, 5),
            (10, 10),
            (5, 10),
            (0, 10),
            (0, 5),
        ]);
        let small = contour(&[(0, 0), (10, 0), (0, 10)]);
        let mut triangulator = IntTriangulator::<I, u32>::default();
        let mut mesh = empty_mesh();
        triangulator.uncheck_delaunay_contour_into(&large, &mut mesh);
        let capacities = (mesh.points.capacity(), mesh.triangles.capacity());
        let pointers = (mesh.points.as_ptr(), mesh.triangles.as_ptr());
        for earcut in [true, false, true] {
            triangulator.earcut = earcut;
            for (input, area) in [(&small, 100), (&large, 200)] {
                triangulator.uncheck_delaunay_contour_into(input, &mut mesh);
                validate(&mesh, area);
                assert_eq!(mesh.points.len(), input.len());
                assert_eq!(
                    capacities,
                    (mesh.points.capacity(), mesh.triangles.capacity())
                );
                assert_eq!(pointers, (mesh.points.as_ptr(), mesh.triangles.as_ptr()));
                if earcut {
                    assert_eq!(&mesh.points, input);
                }
            }
        }
    }
    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn holes_and_disjoint_shapes_keep_boundary_edges_and_neighbors() {
    let shape = shape_with_hole();
    let second = vec![contour(&[(30, 0), (40, 0), (40, 10), (30, 10)])];
    let shapes = vec![shape.clone(), second];
    let mut flat = FlatContoursBuffer::default();
    flat.set_with_shapes(&shapes);
    let mut mesh = empty_mesh();
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    let mut render = IntTriangulation::<i32, u32>::default();
    for earcut in [false, true] {
        triangulator.earcut = earcut;
        triangulator.uncheck_delaunay_shape_into(&shape, &mut mesh);
        validate(&mesh, 600);
        for use_flat in [false, true] {
            if use_flat {
                triangulator.uncheck_delaunay_flat_into(&flat, &mut mesh);
            } else {
                triangulator.uncheck_delaunay_shapes_into(&shapes, &mut mesh);
            }
            validate(&mesh, 800);
            let mut boundary = Vec::new();
            for triangle in &mesh.triangles {
                for (side, &neighbor) in triangle.neighbors.iter().enumerate() {
                    if neighbor == usize::MAX {
                        boundary.push((
                            triangle.vertices[(side + 1) % 3].point,
                            triangle.vertices[(side + 2) % 3].point,
                        ));
                    }
                }
            }
            assert_eq!(boundary.len(), flat.points.len());
            for range in &flat.ranges {
                let points = &flat.points[range.clone()];
                for i in 0..points.len() {
                    assert!(boundary.contains(&(points[i], points[(i + 1) % points.len()])));
                }
            }
            render.fill_with_delaunay(&mesh);
            assert_eq!(render.indices.len(), 3 * mesh.triangles.len());
            assert_eq!(render.points, mesh.points);
            let convex_area: i64 = mesh
                .to_convex_polygons()
                .iter()
                .map(|polygon| {
                    (1..polygon.len() - 1)
                        .map(|i| Triangle::area_two(polygon[0], polygon[i], polygon[i + 1]))
                        .sum::<i64>()
                })
                .sum();
            assert_eq!(convex_area, 800);
        }
    }
}

#[test]
fn output_buffers_survive_switching_between_all_input_forms() {
    let shape = shape_with_hole();
    let small = contour(&[(30, 0), (40, 0), (30, 10)]);
    let shapes = vec![shape.clone(), vec![small.clone()]];
    let mut flat = FlatContoursBuffer::default();
    flat.set_with_shapes(&shapes);
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    let mut mesh = empty_mesh();
    triangulator.uncheck_delaunay_shapes_into(&shapes, &mut mesh);
    let capacities = (mesh.points.capacity(), mesh.triangles.capacity());
    let pointers = (mesh.points.as_ptr(), mesh.triangles.as_ptr());
    for earcut in [true, false] {
        triangulator.earcut = earcut;
        for method in 0..5 {
            let area = match method {
                0 => {
                    triangulator.uncheck_delaunay_shape_into(&shape, &mut mesh);
                    600
                }
                1 => {
                    triangulator.uncheck_delaunay_contour_into(&small, &mut mesh);
                    100
                }
                2 => {
                    triangulator.delaunay_shapes_into(&vec![], &mut mesh);
                    0
                }
                3 => {
                    triangulator.uncheck_delaunay_flat_into(&flat, &mut mesh);
                    700
                }
                _ => {
                    triangulator.uncheck_delaunay_shapes_into(&shapes, &mut mesh);
                    700
                }
            };
            validate(&mesh, area);
            assert_eq!(
                capacities,
                (mesh.points.capacity(), mesh.triangles.capacity())
            );
            assert_eq!(pointers, (mesh.points.as_ptr(), mesh.triangles.as_ptr()));
        }
    }
}

#[test]
fn checked_methods_simplify_self_intersections_and_reversed_winding() {
    let bowtie = contour(&[(0, 0), (10, 10), (0, 10), (10, 0)]);
    let reversed = contour(&[(0, 0), (0, 10), (10, 10), (10, 0)]);
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    let mut mesh = empty_mesh();
    for earcut in [false, true] {
        triangulator.earcut = earcut;
        for (input, area) in [(&bowtie, 100), (&reversed, 200)] {
            triangulator.delaunay_contour_into(input, &mut mesh);
            validate(&mesh, area);
            let shape = vec![input.clone()];
            triangulator.delaunay_shape_into(&shape, &mut mesh);
            validate(&mesh, area);
            triangulator.delaunay_shapes_into(&vec![shape.clone()], &mut mesh);
            validate(&mesh, area);
            let mut flat = FlatContoursBuffer::default();
            flat.set_with_shape(&shape);
            triangulator.delaunay_flat_into(&mut flat, &mut mesh);
            validate(&mesh, area);
        }
    }
}

#[test]
fn checked_shapes_and_flat_use_the_configured_fill_rule_together() {
    let shapes = vec![
        vec![contour(&[(0, 0), (10, 0), (10, 10), (0, 10)])],
        vec![contour(&[(5, 0), (15, 0), (15, 10), (5, 10)])],
    ];
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    let mut mesh = empty_mesh();
    for (fill_rule, area) in [(FillRule::NonZero, 300), (FillRule::EvenOdd, 200)] {
        triangulator.fill_rule = fill_rule;
        triangulator.delaunay_shapes_into(&shapes, &mut mesh);
        validate(&mesh, area);
        let mut flat = FlatContoursBuffer::default();
        flat.set_with_shapes(&shapes);
        triangulator.delaunay_flat_into(&mut flat, &mut mesh);
        validate(&mesh, area);
    }
}

#[test]
fn checked_entry_points_clear_empty_output_without_losing_capacity() {
    let valid = contour(&[(0, 0), (10, 0), (0, 10)]);
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    let mut mesh = empty_mesh();
    for earcut in [false, true] {
        triangulator.earcut = earcut;
        for method in 0..4 {
            for nested in [false, true] {
                triangulator.uncheck_delaunay_contour_into(&valid, &mut mesh);
                let capacities = (mesh.points.capacity(), mesh.triangles.capacity());
                let shape = if nested { vec![vec![], vec![]] } else { vec![] };
                let shapes = if nested {
                    vec![shape.clone(), vec![]]
                } else {
                    vec![]
                };
                let mut flat = FlatContoursBuffer::default();
                flat.set_with_shape(&shape);
                match method {
                    0 => triangulator.delaunay_contour_into(&vec![], &mut mesh),
                    1 => triangulator.delaunay_shape_into(&shape, &mut mesh),
                    2 => triangulator.delaunay_shapes_into(&shapes, &mut mesh),
                    _ => triangulator.delaunay_flat_into(&mut flat, &mut mesh),
                }
                assert!(mesh.points.is_empty() && mesh.triangles.is_empty());
                assert_eq!(
                    capacities,
                    (mesh.points.capacity(), mesh.triangles.capacity())
                );
                triangulator.uncheck_delaunay_contour_into(&valid, &mut mesh);
                validate(&mesh, 100);
            }
        }
        for len in 1..3 {
            let short = valid[..len].to_vec();
            let mut flat = FlatContoursBuffer::default();
            flat.set_with_contour(&short);
            for method in 0..3 {
                triangulator.uncheck_delaunay_contour_into(&valid, &mut mesh);
                match method {
                    0 => triangulator.delaunay_contour_into(&short, &mut mesh),
                    1 => triangulator.delaunay_shape_into(&vec![short.clone()], &mut mesh),
                    _ => triangulator.delaunay_flat_into(&mut flat, &mut mesh),
                }
                assert!(mesh.points.is_empty() && mesh.triangles.is_empty());
            }
        }
    }
}

#[test]
fn checked_entry_points_handle_nonempty_input_that_simplifies_to_empty() {
    let valid = contour(&[(0, 0), (10, 0), (0, 10)]);
    let doubled = valid.iter().chain(&valid).copied().collect();
    let shape = vec![valid.clone(), valid.clone()];
    let shapes = vec![vec![valid.clone()], vec![valid.clone()]];
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    triangulator.fill_rule = FillRule::EvenOdd;
    let mut mesh = empty_mesh();
    for method in 0..4 {
        triangulator.uncheck_delaunay_contour_into(&valid, &mut mesh);
        let capacities = (mesh.points.capacity(), mesh.triangles.capacity());
        let mut flat = FlatContoursBuffer::default();
        flat.set_with_shapes(&shapes);
        match method {
            0 => triangulator.delaunay_contour_into(&doubled, &mut mesh),
            1 => triangulator.delaunay_shape_into(&shape, &mut mesh),
            2 => triangulator.delaunay_shapes_into(&shapes, &mut mesh),
            _ => triangulator.delaunay_flat_into(&mut flat, &mut mesh),
        }
        assert!(mesh.points.is_empty() && mesh.triangles.is_empty());
        assert_eq!(
            capacities,
            (mesh.points.capacity(), mesh.triangles.capacity())
        );
    }
}

#[test]
fn earcut_dispatches_at_the_64_point_boundary() {
    let mut input = contour::<i32>(&[(0, 0)]);
    input.extend((1..=32).map(|x| IntPoint::new(x, 0)));
    input.extend((1..=32).map(|y| IntPoint::new(32, y)));
    input.push(IntPoint::new(0, 32));
    assert_eq!(input.len(), 66);
    let mut triangulator = IntTriangulator::<i32, u8>::default();
    triangulator.earcut = true;
    let mut mesh = empty_mesh();
    let mut flat = FlatContoursBuffer::default();
    for len in [66, 65, 64] {
        assert_eq!(input.len(), len);
        triangulator.uncheck_delaunay_contour_into(&input, &mut mesh);
        validate(&mesh, 2048);
        if len == 64 {
            assert_eq!(mesh.points, input);
        }
        flat.set_with_contour(&input);
        triangulator.uncheck_delaunay_flat_into(&flat, &mut mesh);
        validate(&mesh, 2048);
        if len == 64 {
            assert_eq!(mesh.points, input);
        }
        input.remove(1);
    }
}

#[test]
fn refinement_is_applied_even_when_the_flat_output_flag_is_false() {
    let input = contour::<i32>(&[(0, 0), (10, 0), (10, 6), (0, 10)]);
    let raw = input.uncheck_triangulate();
    let before = raw.triangle_indices::<u32>();
    let expected = raw.into_delaunay();
    assert_ne!(before, expected.triangle_indices::<u32>());
    let mut triangulator = IntTriangulator::<i32, u32>::default();
    assert!(!triangulator.delaunay);
    let mut mesh = empty_mesh();
    triangulator.uncheck_delaunay_contour_into(&input, &mut mesh);
    assert_eq!(mesh.points, expected.points);
    assert_eq!(
        mesh.triangle_indices::<u32>(),
        expected.triangle_indices::<u32>()
    );
    assert_eq!(mesh.triangle_neighbors(), expected.triangle_neighbors());
}
