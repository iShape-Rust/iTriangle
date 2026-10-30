use i_triangle::i_overlay::core::fill_rule::FillRule;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::core::overlay::Overlay;
use i_triangle::i_overlay::core::overlay_rule::OverlayRule;
use i_triangle::i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::i_overlay::i_shape::int::path::ContourExtension;
use i_triangle::i_overlay::i_shape::int::shape::{IntContour, IntShapes};
use i_triangle::i_overlay::i_shape::source::int::resource::IntShapeResource;
use i_triangle::i_overlay::i_shape::{int_path, int_shapes};
use i_triangle::int::minkowski::{IntMinkowskiOperand, IntMinkowskiSolver, MinkowskiError};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::collections::BTreeSet;

fn assert_geometry<I: OverlayInt, R: IntShapeResource<I> + ?Sized>(
    actual: &IntShapes<I>,
    expected: &R,
) {
    assert!(Overlay::from_subj_and_clip(actual, expected)
        .overlay(OverlayRule::Xor, FillRule::NonZero)
        .is_empty());
}

#[test]
fn adds_translated_rectangles_for_all_integer_types() {
    fn check<I: OverlayInt + From<i16>>() {
        let contour = |points: [(i16, i16); 4]| -> IntContour<I> {
            points
                .map(|(x, y)| IntPoint::new(I::from(x), I::from(y)))
                .into()
        };
        let a = contour([(-10, 3), (-4, 3), (-4, 8), (-10, 8)]);
        let b = contour([(2, -5), (6, -5), (6, -2), (2, -2)]);
        let expected = contour([(-8, -2), (2, -2), (2, 6), (-8, 6)]);
        let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
        let rhs = IntMinkowskiOperand::with_source(&b, FillRule::NonZero).unwrap();
        let mut solver = IntMinkowskiSolver::default();
        assert_geometry(&solver.sum(&lhs, &rhs).unwrap(), &expected);
        assert_geometry(&solver.sum(&rhs, &lhs).unwrap(), &expected);
    }

    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn builds_a_hexagon_from_triangles_with_opposite_directions() {
    let a = int_path![[0, 0], [4, 0], [0, 4]];
    let b = int_path![[0, 0], [-2, 0], [0, -2]];
    let expected = int_path![[-2, 0], [0, -2], [4, -2], [4, 0], [0, 4], [-2, 4]];
    let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
    let rhs = IntMinkowskiOperand::with_source(&b, FillRule::NonZero).unwrap();
    let mut solver = IntMinkowskiSolver::default();
    assert_geometry(&solver.sum(&lhs, &rhs).unwrap(), &expected);
    assert_geometry(&solver.sum(&rhs, &lhs).unwrap(), &expected);
}

#[test]
fn merges_partial_sums_without_filling_concavities() {
    let a = int_path![[0, 0], [4, 0], [4, 1], [1, 1], [1, 4], [0, 4]];
    let b = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let expected = int_path![[0, 0], [5, 0], [5, 2], [2, 2], [2, 5], [0, 5]];
    let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
    let rhs = IntMinkowskiOperand::with_source(&b, FillRule::NonZero).unwrap();
    let mut solver = IntMinkowskiSolver::default();
    assert_geometry(&solver.sum(&lhs, &rhs).unwrap(), &expected);
    assert_geometry(&solver.sum(&rhs, &lhs).unwrap(), &expected);
}

#[test]
fn preserves_and_closes_holes() {
    let a = int_shapes![[
        [[0, 0], [10, 0], [10, 10], [0, 10]],
        [[3, 3], [3, 7], [7, 7], [7, 3]]
    ]];
    let small = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let large = int_path![[0, 0], [4, 0], [4, 4], [0, 4]];
    let expected = int_shapes![[
        [[0, 0], [11, 0], [11, 11], [0, 11]],
        [[4, 4], [4, 7], [7, 7], [7, 4]]
    ]];
    let filled = int_path![[0, 0], [14, 0], [14, 14], [0, 14]];
    let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
    let small = IntMinkowskiOperand::with_source(&small, FillRule::NonZero).unwrap();
    let large = IntMinkowskiOperand::with_source(&large, FillRule::NonZero).unwrap();
    let mut solver = IntMinkowskiSolver::default();
    assert_geometry(&solver.sum(&lhs, &small).unwrap(), &expected);
    assert_geometry(&solver.sum(&lhs, &large).unwrap(), &filled);
}

#[test]
fn reuses_solver_for_disconnected_and_merged_results() {
    let a = int_shapes![
        [[[0, 0], [1, 0], [1, 1], [0, 1]]],
        [[[4, 0], [5, 0], [5, 1], [4, 1]]]
    ];
    let small = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let large = int_path![[0, 0], [4, 0], [4, 1], [0, 1]];
    let separate = int_shapes![
        [[[0, 0], [2, 0], [2, 2], [0, 2]]],
        [[[4, 0], [6, 0], [6, 2], [4, 2]]]
    ];
    let merged = int_path![[0, 0], [9, 0], [9, 2], [0, 2]];
    let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
    let small = IntMinkowskiOperand::with_source(&small, FillRule::NonZero).unwrap();
    let large = IntMinkowskiOperand::with_source(&large, FillRule::NonZero).unwrap();
    let mut solver = IntMinkowskiSolver::new(0);
    for _ in 0..3 {
        let result = solver.sum(&lhs, &small).unwrap();
        assert_eq!(result.len(), 2);
        assert_geometry(&result, &separate);
        let result = solver.sum(&lhs, &large).unwrap();
        assert_eq!(result.len(), 1);
        assert_geometry(&result, &merged);
        solver.clear();
    }
}

#[test]
fn checks_all_bbox_bounds_and_recovers_after_errors() {
    fn check<I: OverlayInt>() {
        let limit = I::Wide::ONE << (I::BITS - 2);
        let one = I::Wide::ONE;
        let two = I::Wide::TWO;
        let four = I::Wide::FOUR;
        for swap_axes in [false, true] {
            let rectangle = |min: I::Wide, max: I::Wide| -> IntContour<I> {
                [
                    (min, I::Wide::ZERO),
                    (max, I::Wide::ZERO),
                    (max, two),
                    (min, two),
                ]
                .map(|(x, y)| {
                    let (x, y) = if swap_axes { (y, x) } else { (x, y) };
                    IntPoint::new(I::from_wide(x), I::from_wide(y))
                })
                .into()
            };
            let mut solver = IntMinkowskiSolver::default();
            for positive in [false, true] {
                let a = if positive {
                    rectangle(limit - four, limit - two)
                } else {
                    rectangle(-limit + two, -limit + four)
                };
                let valid = if positive {
                    rectangle(I::Wide::ZERO, one)
                } else {
                    rectangle(-one, I::Wide::ZERO)
                };
                let invalid = if positive {
                    rectangle(one, two)
                } else {
                    rectangle(-two, -one)
                };
                let lhs = IntMinkowskiOperand::with_source(&a, FillRule::NonZero).unwrap();
                let valid = IntMinkowskiOperand::with_source(&valid, FillRule::NonZero).unwrap();
                let invalid =
                    IntMinkowskiOperand::with_source(&invalid, FillRule::NonZero).unwrap();
                let before = solver.sum(&lhs, &valid).unwrap();
                assert!(matches!(
                    solver.sum(&lhs, &invalid),
                    Err(MinkowskiError::CoordinateOutOfRange)
                ));
                assert!(matches!(
                    solver.sum(&lhs, &lhs),
                    Err(MinkowskiError::CoordinateOutOfRange)
                ));
                assert_geometry(&solver.sum(&lhs, &valid).unwrap(), &before);
            }
        }
    }

    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn rejects_source_bbox_even_when_normalized_sum_would_fit() {
    let limit = 1_i32 << 30;
    let square = int_path![[0, 0], [2, 0], [2, 2], [0, 2]];
    let remote = int_path![
        [limit - 4, 0],
        [limit - 2, 0],
        [limit - 2, 2],
        [limit - 4, 2]
    ];
    let source = vec![square.clone(), remote.clone(), remote];
    let kernel = int_path![[3, 0], [5, 0], [5, 2], [3, 2]];
    let lhs = IntMinkowskiOperand::with_source(&source, FillRule::EvenOdd).unwrap();
    let rhs = IntMinkowskiOperand::with_source(&kernel, FillRule::NonZero).unwrap();
    let retained = IntMinkowskiOperand::with_source(&square, FillRule::NonZero).unwrap();
    let mut solver = IntMinkowskiSolver::default();
    assert!(matches!(
        solver.sum(&lhs, &rhs),
        Err(MinkowskiError::CoordinateOutOfRange)
    ));
    assert_geometry(
        &solver.sum(&retained, &rhs).unwrap(),
        &int_path![[3, 0], [7, 0], [7, 4], [3, 4]],
    );
}

#[test]
fn preparation_and_sum_share_overlay_without_leaking_fill_rules_or_geometry() {
    let paths = vec![
        int_path![[0, 0], [10, 0], [10, 10], [0, 10]],
        int_path![[3, 3], [7, 3], [7, 7], [3, 7]],
    ];
    let square = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let filled = int_path![[0, 0], [11, 0], [11, 11], [0, 11]];
    let hole = int_shapes![[
        [[0, 0], [11, 0], [11, 11], [0, 11]],
        [[4, 4], [4, 7], [7, 7], [7, 4]]
    ]];
    let empty: IntContour<i32> = Vec::new();
    let mut solver = IntMinkowskiSolver::default();
    let kernel = solver
        .prepare(square.as_slice(), FillRule::NonZero)
        .unwrap();
    let original_kernel = kernel.convex_polygons().to_vec();

    for _ in 0..3 {
        for fill_rule in [FillRule::EvenOdd, FillRule::NonZero, FillRule::Positive] {
            let operand = solver.prepare(&paths, fill_rule).unwrap();
            assert!(solver.prepare(&empty, FillRule::NonZero).is_none());
            assert!(solver.prepare(&paths, FillRule::Negative).is_none());
            let result = solver.sum(&operand, &kernel).unwrap();
            if fill_rule == FillRule::EvenOdd {
                assert_geometry(&result, &hole);
            } else {
                assert_geometry(&result, &filled);
            }
        }
        let reversed: Vec<_> = paths
            .iter()
            .map(|path| path.iter().rev().copied().collect::<IntContour<i32>>())
            .collect();
        let operand = solver.prepare(&reversed, FillRule::Negative).unwrap();
        assert_geometry(&solver.sum(&operand, &kernel).unwrap(), &filled);
        assert_eq!(kernel.convex_polygons(), original_kernel);
    }
}

#[test]
fn output_collinear_option_survives_preparation() {
    let source = int_path![[0, 0], [4, 0], [4, 1], [1, 1], [1, 4], [0, 4]];
    let square = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let expected = int_path![[0, 0], [5, 0], [5, 2], [2, 2], [2, 5], [0, 5]];
    let empty: IntContour<i32> = Vec::new();
    for preserve_output_collinear in [false, true] {
        let mut solver = IntMinkowskiSolver::with_options(0, preserve_output_collinear);
        for _ in 0..3 {
            let operand = solver.prepare(&source, FillRule::NonZero).unwrap();
            let kernel = solver.prepare(&square, FillRule::NonZero).unwrap();
            assert!(operand
                .convex_polygons()
                .iter()
                .all(|p| !p.is_clockwise_ordered()));
            assert!(solver.prepare(&empty, FillRule::NonZero).is_none());
            assert!(solver.prepare(&square, FillRule::Negative).is_none());

            let result = solver.sum(&operand, &kernel).unwrap();
            assert_eq!(result.len(), 1);
            assert!(!result[0][0].is_clockwise_ordered());
            assert_geometry(&result, &expected);
            if preserve_output_collinear {
                assert!(result[0][0].len() > expected.len());
            } else {
                assert_eq!(result[0][0].len(), expected.len());
            }
            solver.clear();
        }
    }
}

fn cells_to_contours(cells: &BTreeSet<(i32, i32)>) -> Vec<IntContour<i32>> {
    cells
        .iter()
        .map(|&(x, y)| int_path![[x, y], [x + 1, y], [x + 1, y + 1], [x, y + 1]])
        .collect()
}

fn contains_cell_center(shapes: &IntShapes<i32>, x: i32, y: i32) -> bool {
    let px = 2 * x + 1;
    let py = 2 * y + 1;
    let mut winding = 0;
    for contour in shapes.iter().flatten() {
        for (&a, &b) in contour.iter().zip(contour.iter().cycle().skip(1)) {
            let cross = (b.x - a.x) * (py - 2 * a.y) - (b.y - a.y) * (px - 2 * a.x);
            if 2 * a.y <= py && 2 * b.y > py && cross > 0 {
                winding += 1;
            } else if 2 * a.y > py && 2 * b.y <= py && cross < 0 {
                winding -= 1;
            }
        }
    }
    winding != 0
}

#[test]
fn random_cell_shapes_match_independent_discrete_sum_and_difference() {
    let mut rng = StdRng::seed_from_u64(0x4d_49_4e_4b);
    let mut solver = IntMinkowskiSolver::new(0);
    for case in 0..128 {
        let mut make_cells = || {
            let mut cells = BTreeSet::new();
            for x in -2..=2 {
                for y in -2..=2 {
                    if rng.random_bool(0.3) {
                        cells.insert((x, y));
                    }
                }
            }
            if cells.is_empty() {
                cells.insert((0, 0));
            }
            cells
        };
        let a = make_cells();
        let b = make_cells();
        let mut expected_sum = BTreeSet::new();
        let mut expected_difference = BTreeSet::new();
        for &(ax, ay) in &a {
            for &(bx, by) in &b {
                for dx in 0..=1 {
                    for dy in 0..=1 {
                        expected_sum.insert((ax + bx + dx, ay + by + dy));
                        expected_difference.insert((ax - bx - 1 + dx, ay - by - 1 + dy));
                    }
                }
            }
        }

        let oblique = case % 2 != 0;
        let transform = |cells: &BTreeSet<(i32, i32)>| {
            let mut contours = cells_to_contours(cells);
            if oblique {
                for p in contours.iter_mut().flatten() {
                    *p = IntPoint::new(2 * p.x + p.y, p.x + p.y);
                }
            }
            contours
        };
        let lhs = solver.prepare(&transform(&a), FillRule::NonZero).unwrap();
        let rhs = solver.prepare(&transform(&b), FillRule::NonZero).unwrap();
        for (lhs, rhs, difference) in [(&lhs, &rhs, false), (&rhs, &lhs, false), (&lhs, &rhs, true)]
        {
            let (mut result, expected) = if difference {
                (solver.difference(lhs, rhs).unwrap(), &expected_difference)
            } else {
                (solver.sum(lhs, rhs).unwrap(), &expected_sum)
            };
            if oblique {
                for p in result.iter_mut().flatten().flatten() {
                    *p = IntPoint::new(p.x - p.y, 2 * p.y - p.x);
                }
            }
            let mut area_two = 0;
            for contour in result.iter().flatten() {
                for (&p, &q) in contour.iter().zip(contour.iter().cycle().skip(1)) {
                    assert!(
                        p.x == q.x || p.y == q.y,
                        "case {case}: non-axis-aligned boundary"
                    );
                    area_two += p.x * q.y - p.y * q.x;
                }
            }
            assert_eq!(
                area_two,
                2 * expected.len() as i32,
                "case {case}: area mismatch"
            );
            for x in -5..=6 {
                for y in -5..=6 {
                    assert_eq!(
                        contains_cell_center(&result, x, y),
                        expected.contains(&(x, y)),
                        "case {case}, cell ({x}, {y}), a={a:?}, b={b:?}"
                    );
                }
            }
        }
    }
}
