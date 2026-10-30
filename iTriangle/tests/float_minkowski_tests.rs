use i_triangle::float::minkowski::{
    minkowski_difference, minkowski_sum, MinkowskiError, MinkowskiSolver,
};
use i_triangle::i_overlay::core::fill_rule::FillRule;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::core::overlay_rule::OverlayRule;
use i_triangle::i_overlay::float::overlay::FloatOverlay;
use i_triangle::i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_triangle::i_overlay::i_float::float::number::FloatNumber;
use i_triangle::i_overlay::i_float::float::rect::FloatRect;
use i_triangle::i_overlay::i_shape::base::data::Shapes;

fn domain<F: FloatNumber>(min: f64, max: f64) -> FloatRect<F> {
    FloatRect::new(
        F::from_float(min),
        F::from_float(max),
        F::from_float(min),
        F::from_float(max),
    )
    .unwrap()
}

fn rectangle<F: FloatNumber>(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<[F; 2]> {
    [
        [min_x, min_y],
        [max_x, min_y],
        [max_x, max_y],
        [min_x, max_y],
    ]
    .map(|[x, y]| [F::from_float(x), F::from_float(y)])
    .into()
}

fn assert_rectangle<F: FloatNumber>(result: &Shapes<[F; 2]>, expected: &[[F; 2]], tolerance: f64) {
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].len(), 1);
    assert_eq!(result[0][0].len(), 4);
    for p in &result[0][0] {
        assert!(
            expected
                .iter()
                .any(|q| (p[0].to_f64() - q[0].to_f64()).abs() <= tolerance
                    && (p[1].to_f64() - q[1].to_f64()).abs() <= tolerance),
            "unexpected point ({}, {})",
            p[0],
            p[1]
        );
    }
}

#[test]
fn fixed_grid_operands_work_for_all_coordinate_types() {
    fn check<F: FloatNumber, I: OverlayInt>() {
        let a = rectangle::<F>(-10.5, -4.5, 3.25, 8.25);
        let b = rectangle::<F>(2.0, 6.0, -5.0, -2.0);
        let mut solver = MinkowskiSolver::<[F; 2], I>::new(domain(-32.0, 32.0));
        let lhs = solver
            .prepare(a.as_slice(), FillRule::NonZero)
            .unwrap()
            .unwrap();
        let rhs = solver.prepare(&b, FillRule::NonZero).unwrap().unwrap();
        for _ in 0..3 {
            assert_rectangle(
                &solver.sum(&lhs, &rhs).unwrap(),
                &rectangle::<F>(-8.5, 1.5, -1.75, 6.25),
                1e-6,
            );
            assert_rectangle(
                &solver.difference(&lhs, &rhs).unwrap(),
                &rectangle::<F>(-16.5, -6.5, 5.25, 13.25),
                1e-6,
            );
            assert_rectangle(
                &solver.difference(&rhs, &lhs).unwrap(),
                &rectangle::<F>(6.5, 16.5, -13.25, -5.25),
                1e-6,
            );
        }
    }
    check::<f32, i16>();
    check::<f32, i32>();
    check::<f32, i64>();
    check::<f64, i16>();
    check::<f64, i32>();
    check::<f64, i64>();
}

#[test]
fn fractional_rectangles_use_the_fixed_coordinate_system() {
    fn check<F: FloatNumber, I: OverlayInt>() {
        let mut solver = MinkowskiSolver::<[F; 2], I>::new(domain(-32.0, 32.0));
        for i in 0..64 {
            let x = i as f64 * 0.137 - 4.1;
            let y = i as f64 * -0.083 + 2.3;
            let a = rectangle::<F>(x, x + 1.31, y, y + 0.79);
            let b = rectangle::<F>(-0.23, 0.48, 0.17, 0.88);
            let lhs = solver.prepare(&a, FillRule::NonZero).unwrap().unwrap();
            let rhs = solver.prepare(&b, FillRule::NonZero).unwrap().unwrap();
            assert_rectangle(
                &solver.sum(&lhs, &rhs).unwrap_or_else(|e| {
                    panic!(
                        "{e:?}, F={}, I={}, i={i}",
                        core::any::type_name::<F>(),
                        core::any::type_name::<I>()
                    )
                }),
                &rectangle::<F>(x - 0.23, x + 1.79, y + 0.17, y + 1.67),
                0.01,
            );
            assert_rectangle(
                &solver.difference(&lhs, &rhs).unwrap(),
                &rectangle::<F>(x - 0.48, x + 1.54, y - 0.88, y + 0.62),
                0.01,
            );
        }
    }
    check::<f32, i16>();
    check::<f32, i32>();
    check::<f32, i64>();
    check::<f64, i16>();
    check::<f64, i32>();
    check::<f64, i64>();
}

#[test]
fn holes_concavities_and_collinear_options_work_on_fixed_grid() {
    let concave = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 1.0],
        [1.0, 1.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ];
    let source = vec![
        rectangle::<f64>(0.0, 10.0, 0.0, 10.0),
        vec![[3.0, 3.0], [3.0, 7.0], [7.0, 7.0], [7.0, 3.0]],
    ];
    let kernel = rectangle::<f64>(0.0, 1.0, 0.0, 1.0);
    let expected = vec![
        rectangle::<f64>(0.0, 11.0, 0.0, 11.0),
        vec![[4.0, 4.0], [4.0, 7.0], [7.0, 7.0], [7.0, 4.0]],
    ];
    let expected_concave = vec![
        [-1.0, -1.0],
        [4.0, -1.0],
        [4.0, 1.0],
        [1.0, 1.0],
        [1.0, 4.0],
        [-1.0, 4.0],
    ];
    for keep_collinear in [false, true] {
        let mut solver =
            MinkowskiSolver::<[f64; 2]>::with_options(domain(-16.0, 16.0), 0, keep_collinear);
        let rhs = solver.prepare(&kernel, FillRule::NonZero).unwrap().unwrap();
        for _ in 0..3 {
            let lhs = solver.prepare(&source, FillRule::NonZero).unwrap().unwrap();
            let result = solver.sum(&lhs, &rhs).unwrap();
            assert!(
                FloatOverlay::<[f64; 2]>::from_subj_and_clip(&result, &expected)
                    .overlay(OverlayRule::Xor, FillRule::NonZero)
                    .is_empty()
            );
            let lhs = solver
                .prepare(&concave, FillRule::NonZero)
                .unwrap()
                .unwrap();
            let result = solver.difference(&lhs, &rhs).unwrap();
            assert!(
                FloatOverlay::<[f64; 2]>::from_subj_and_clip(&result, &expected_concave)
                    .overlay(OverlayRule::Xor, FillRule::NonZero)
                    .is_empty()
            );
            assert_eq!(result[0][0].len() > expected_concave.len(), keep_collinear);
        }
    }
}

#[test]
fn empty_and_unfilled_sources_have_no_operand() {
    let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain(-32.0, 32.0));
    assert!(solver
        .prepare(&Vec::<[f64; 2]>::new(), FillRule::NonZero)
        .unwrap()
        .is_none());
    assert!(solver
        .prepare(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]], FillRule::NonZero)
        .unwrap()
        .is_none());
    let square = rectangle::<f64>(0.0, 1.0, 0.0, 1.0);
    assert!(solver
        .prepare(&square, FillRule::Negative)
        .unwrap()
        .is_none());
    assert!(solver
        .prepare(&[square.clone(), square.clone()], FillRule::EvenOdd)
        .unwrap()
        .is_none());
    assert!(solver
        .prepare(&square, FillRule::NonZero)
        .unwrap()
        .is_some());
}

#[test]
fn out_of_range_result_is_reported_without_poisoning_the_solver() {
    fn check<F: FloatNumber>() {
        let limit = F::MAX_COORDINATE.to_f64();
        let a = rectangle::<F>(limit * 0.75, limit, 0.0, limit * 0.25);
        let mut solver = MinkowskiSolver::<[F; 2]>::new(domain(-limit, limit));
        let operand = solver.prepare(&a, FillRule::NonZero).unwrap().unwrap();
        assert!(matches!(
            solver.sum(&operand, &operand),
            Err(MinkowskiError::CoordinateOutOfRange)
        ));
        assert_rectangle(
            &solver.difference(&operand, &operand).unwrap(),
            &rectangle::<F>(-limit * 0.25, limit * 0.25, -limit * 0.25, limit * 0.25),
            limit * 1e-6,
        );
    }
    check::<f32>();
    check::<f64>();
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl FloatPointCompatible for Point {
    type Scalar = f64;
    fn from_xy(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
}

#[test]
fn supports_custom_float_points() {
    let points = rectangle::<f64>(0.0, 2.0, 0.0, 2.0)
        .into_iter()
        .map(|[x, y]| Point { x, y })
        .collect::<Vec<_>>();
    let mut solver = MinkowskiSolver::<Point>::new(domain(-32.0, 32.0));
    let operand = solver.prepare(&points, FillRule::NonZero).unwrap().unwrap();
    let result = solver.sum(&operand, &operand).unwrap();
    assert_eq!(result[0][0].len(), 4);
    assert!(result[0][0]
        .iter()
        .all(|p| (p.x == 0.0 || p.x == 4.0) && (p.y == 0.0 || p.y == 4.0)));
}

#[test]
fn nonzero_grid_origin_is_accounted_for_in_both_operations() {
    let mut solver = MinkowskiSolver::<[f64; 2], i16>::new(domain(-4.0, 24.0));
    let a = solver
        .prepare(&rectangle::<f64>(8.0, 10.0, 8.0, 10.0), FillRule::NonZero)
        .unwrap()
        .unwrap();
    let b = solver
        .prepare(&rectangle::<f64>(10.0, 12.0, 10.0, 12.0), FillRule::NonZero)
        .unwrap()
        .unwrap();
    let original = a.int_operand().convex_polygons().to_vec();
    for _ in 0..3 {
        assert_rectangle(
            &solver.sum(&a, &b).unwrap(),
            &rectangle::<f64>(18.0, 22.0, 18.0, 22.0),
            1e-8,
        );
        assert_rectangle(
            &solver.difference(&a, &b).unwrap(),
            &rectangle::<f64>(-4.0, 0.0, -4.0, 0.0),
            1e-8,
        );
        assert_eq!(original, a.int_operand().convex_polygons());
        solver.clear();
    }
}

#[test]
fn rejects_source_and_result_bounds_outside_the_domain() {
    let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain(-8.0, 8.0));
    for source in [
        rectangle::<f64>(-9.0, 0.0, 0.0, 1.0),
        rectangle::<f64>(0.0, 9.0, 0.0, 1.0),
        rectangle::<f64>(0.0, 1.0, -9.0, 0.0),
        rectangle::<f64>(0.0, 1.0, 0.0, 9.0),
    ] {
        assert!(matches!(
            solver.prepare(&source, FillRule::NonZero),
            Err(MinkowskiError::OutsideDomain)
        ));
    }
    let a = solver
        .prepare(&rectangle::<f64>(4.0, 8.0, 4.0, 8.0), FillRule::NonZero)
        .unwrap()
        .unwrap();
    let b = solver
        .prepare(&rectangle::<f64>(-8.0, -4.0, -8.0, -4.0), FillRule::NonZero)
        .unwrap()
        .unwrap();
    assert!(matches!(
        solver.sum(&a, &a),
        Err(MinkowskiError::OutsideDomain)
    ));
    assert!(matches!(
        solver.difference(&a, &b),
        Err(MinkowskiError::OutsideDomain)
    ));
    assert_rectangle(
        &solver.sum(&a, &b).unwrap(),
        &rectangle::<f64>(-4.0, 4.0, -4.0, 4.0),
        1e-8,
    );
}

#[test]
fn foreign_operands_are_rejected_even_with_identical_grids() {
    let source = rectangle::<f64>(0.0, 1.0, 0.0, 1.0);
    let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain(-16.0, 16.0));
    let a = solver.prepare(&source, FillRule::NonZero).unwrap().unwrap();
    for rect in [
        domain(-16.0, 16.0),
        domain(-15.0, 15.0),
        domain(-8.0, 8.0),
        domain(-15.0, 17.0),
    ] {
        let mut foreign = MinkowskiSolver::<[f64; 2]>::new(rect);
        let b = foreign
            .prepare(&source, FillRule::NonZero)
            .unwrap()
            .unwrap();
        for (lhs, rhs) in [(&a, &b), (&b, &a)] {
            assert!(matches!(
                solver.sum(lhs, rhs),
                Err(MinkowskiError::ForeignOperand)
            ));
            assert!(matches!(
                solver.difference(lhs, rhs),
                Err(MinkowskiError::ForeignOperand)
            ));
        }
    }
    let mut foreign = MinkowskiSolver::<[f64; 2]>::new(domain(100.0, 120.0));
    let b = foreign
        .prepare(
            &rectangle::<f64>(101.0, 102.0, 101.0, 102.0),
            FillRule::NonZero,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        solver.sum(&a, &b),
        Err(MinkowskiError::ForeignOperand)
    ));
    assert_rectangle(
        &solver.sum(&a, &a).unwrap(),
        &rectangle::<f64>(0.0, 2.0, 0.0, 2.0),
        1e-8,
    );
}

#[test]
fn operand_owner_survives_move_clear_and_further_preparation() {
    let rect = domain(-8.0, 8.0);
    let source = rectangle::<f64>(0.0, 1.0, 0.0, 1.0);
    let mut solver = MinkowskiSolver::<[f64; 2]>::new(rect);
    let a = solver.prepare(&source, FillRule::NonZero).unwrap().unwrap();
    let mut moved = core::mem::replace(&mut solver, MinkowskiSolver::new(rect));
    moved.clear();
    let b = moved.prepare(&source, FillRule::NonZero).unwrap().unwrap();
    moved.clear();
    assert_rectangle(
        &moved.sum(&a, &b).unwrap(),
        &rectangle::<f64>(0.0, 2.0, 0.0, 2.0),
        1e-8,
    );
    assert_rectangle(
        &moved.difference(&a, &b).unwrap(),
        &rectangle::<f64>(-1.0, 1.0, -1.0, 1.0),
        1e-8,
    );
    assert!(matches!(
        solver.sum(&a, &b),
        Err(MinkowskiError::ForeignOperand)
    ));
    drop(moved);
    let mut replacement = MinkowskiSolver::<[f64; 2]>::new(rect);
    assert!(matches!(
        replacement.difference(&a, &b),
        Err(MinkowskiError::ForeignOperand)
    ));
}

#[test]
fn sugar_selects_domain_for_sources_and_each_operation() {
    for (a, b) in [
        (
            rectangle::<f64>(8.0, 10.0, 3.0, 5.0),
            rectangle::<f64>(10.0, 12.0, 6.0, 8.0),
        ),
        (
            rectangle::<f64>(1e6, 1e6 + 4.0, 1e6, 1e6 + 4.0),
            rectangle::<f64>(-1e6, -1e6 + 0.25, -1e6, -1e6 + 0.25),
        ),
    ] {
        assert_rectangle(
            &minkowski_sum(a.as_slice(), &b, FillRule::NonZero).unwrap(),
            &rectangle::<f64>(
                a[0][0] + b[0][0],
                a[2][0] + b[2][0],
                a[0][1] + b[0][1],
                a[2][1] + b[2][1],
            ),
            1e-8,
        );
        assert_rectangle(
            &minkowski_difference(&a, &b, FillRule::NonZero).unwrap(),
            &rectangle::<f64>(
                a[0][0] - b[2][0],
                a[2][0] - b[0][0],
                a[0][1] - b[2][1],
                a[2][1] - b[0][1],
            ),
            1e-8,
        );
    }
    let a = rectangle::<f32>(-3.0, 1.0, 2.0, 4.0);
    assert_rectangle(
        &minkowski_sum(&a, &a, FillRule::NonZero).unwrap(),
        &rectangle::<f32>(-6.0, 2.0, 4.0, 8.0),
        1e-6,
    );
}

#[test]
fn sugar_returns_empty_for_empty_unfilled_and_collapsed_sources() {
    let square = rectangle::<f64>(0.0, 1.0, 0.0, 1.0);
    let empty: Vec<[f64; 2]> = Vec::new();
    let line = vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]];
    for source in [&empty, &line] {
        assert!(minkowski_sum(source, &square, FillRule::NonZero)
            .unwrap()
            .is_empty());
        assert!(minkowski_difference(&square, source, FillRule::NonZero)
            .unwrap()
            .is_empty());
    }
    assert!(minkowski_sum(&square, &square, FillRule::Negative)
        .unwrap()
        .is_empty());
    assert!(minkowski_difference(
        &[square.clone(), square.clone()],
        &square,
        FillRule::EvenOdd
    )
    .unwrap()
    .is_empty());
    let tiny = rectangle::<f64>(0.0, 1e-12, 0.0, 1e-12);
    assert!(minkowski_sum(&tiny, &square, FillRule::NonZero)
        .unwrap()
        .is_empty());
}

#[test]
fn sugar_reports_numeric_errors_and_prepare_rejects_invalid_points() {
    let limit = f64::MAX_COORDINATE;
    let large = rectangle::<f64>(limit * 0.75, limit, 0.0, limit * 0.25);
    assert!(matches!(
        minkowski_sum(&large, &large, FillRule::NonZero),
        Err(MinkowskiError::CoordinateOutOfRange)
    ));
    assert!(minkowski_sum(&large, &large, FillRule::Negative)
        .unwrap()
        .is_empty());
    let negative = rectangle::<f64>(-limit, -limit * 0.75, 0.0, limit * 0.25);
    assert!(matches!(
        minkowski_difference(&large, &negative, FillRule::NonZero),
        Err(MinkowskiError::CoordinateOutOfRange)
    ));
    let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain(-8.0, 8.0));
    for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, limit * 2.0] {
        let invalid = [[0.0, 0.0], [x, 0.0], [0.0, 1.0]];
        assert!(matches!(
            solver.prepare(&invalid, FillRule::NonZero),
            Err(MinkowskiError::CoordinateOutOfRange)
        ));
        assert!(matches!(
            minkowski_sum(&invalid, &invalid, FillRule::NonZero),
            Err(MinkowskiError::CoordinateOutOfRange)
        ));
    }
}

#[test]
fn exact_grid_extremes_do_not_overflow_integer_sum() {
    let mut solver = MinkowskiSolver::<[f64; 2], i16>::new(domain(-1.0, 1.0));
    let a = solver
        .prepare(&rectangle::<f64>(0.0, 0.5, 0.0, 0.5), FillRule::NonZero)
        .unwrap()
        .unwrap();
    assert_rectangle(
        &solver.sum(&a, &a).unwrap(),
        &rectangle::<f64>(0.0, 1.0, 0.0, 1.0),
        0.0,
    );
    let a = solver
        .prepare(&rectangle::<f64>(-1.0, 0.0, -1.0, 0.0), FillRule::NonZero)
        .unwrap()
        .unwrap();
    assert_rectangle(
        &solver.difference(&a, &a).unwrap(),
        &rectangle::<f64>(-1.0, 1.0, -1.0, 1.0),
        0.0,
    );
}
