use i_triangle::i_overlay::core::fill_rule::FillRule;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::core::overlay::Overlay;
use i_triangle::i_overlay::core::overlay_rule::OverlayRule;
use i_triangle::i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::i_overlay::i_shape::int::shape::{IntContour, IntShapes};
use i_triangle::i_overlay::i_shape::source::int::resource::IntShapeResource;
use i_triangle::i_overlay::i_shape::{int_path, int_shapes};
use i_triangle::int::minkowski::{IntMinkowskiSolver, MinkowskiError};

fn assert_geometry<I: OverlayInt, R: IntShapeResource<I> + ?Sized>(
    actual: &IntShapes<I>,
    expected: &R,
) {
    assert!(Overlay::from_subj_and_clip(actual, expected)
        .overlay(OverlayRule::Xor, FillRule::NonZero)
        .is_empty());
}

#[test]
fn subtracts_translated_rectangles_for_all_integer_types() {
    fn check<I: OverlayInt + From<i16>>() {
        let contour = |points: [(i16, i16); 4]| -> IntContour<I> {
            points
                .map(|(x, y)| IntPoint::new(I::from(x), I::from(y)))
                .into()
        };
        let a = contour([(-10, 3), (-4, 3), (-4, 8), (-10, 8)]);
        let b = contour([(2, -5), (6, -5), (6, -2), (2, -2)]);
        let expected = contour([(-16, 5), (-6, 5), (-6, 13), (-16, 13)]);
        let reversed = contour([(6, -13), (16, -13), (16, -5), (6, -5)]);
        let mut solver = IntMinkowskiSolver::default();
        let lhs = solver.prepare(&a, FillRule::NonZero).unwrap();
        let rhs = solver.prepare(&b, FillRule::NonZero).unwrap();
        assert_geometry(&solver.difference(&lhs, &rhs).unwrap(), &expected);
        assert_geometry(&solver.difference(&rhs, &lhs).unwrap(), &reversed);
    }

    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn triangle_difference_matches_sum_with_reflection() {
    let a = int_path![[0, 0], [4, 0], [0, 4]];
    let b = int_path![[0, 0], [2, 0], [0, 2]];
    let reflected = int_path![[0, 0], [-2, 0], [0, -2]];
    let expected = int_path![[-2, 0], [0, -2], [4, -2], [4, 0], [0, 4], [-2, 4]];
    let mut solver = IntMinkowskiSolver::default();
    let lhs = solver.prepare(&a, FillRule::NonZero).unwrap();
    let rhs = solver.prepare(&b, FillRule::NonZero).unwrap();
    let reflected = solver.prepare(&reflected, FillRule::NonZero).unwrap();
    let difference = solver.difference(&lhs, &rhs).unwrap();
    assert_geometry(&difference, &expected);
    assert_geometry(&difference, &solver.sum(&lhs, &reflected).unwrap());
}

#[test]
fn difference_is_not_the_inverse_of_sum() {
    let a = int_path![[0, 0], [10, 0], [10, 10], [0, 10]];
    let b = int_path![[0, 0], [2, 0], [2, 2], [0, 2]];
    let mut solver = IntMinkowskiSolver::default();
    let lhs = solver.prepare(&a, FillRule::NonZero).unwrap();
    let rhs = solver.prepare(&b, FillRule::NonZero).unwrap();
    let sum = solver.sum(&lhs, &rhs).unwrap();
    assert_geometry(&sum, &int_path![[0, 0], [12, 0], [12, 12], [0, 12]]);
    let sum = solver.prepare(&sum, FillRule::NonZero).unwrap();
    let difference = solver.difference(&sum, &rhs).unwrap();
    assert_geometry(
        &difference,
        &int_path![[-2, -2], [12, -2], [12, 12], [-2, 12]],
    );
    assert!(!Overlay::from_subj_and_clip(&difference, &a)
        .overlay(OverlayRule::Xor, FillRule::NonZero)
        .is_empty());
}

#[test]
fn preserves_concavity_holes_and_output_options_across_operations() {
    let concave = int_path![[0, 0], [4, 0], [4, 1], [1, 1], [1, 4], [0, 4]];
    let holed = int_shapes![[
        [[0, 0], [10, 0], [10, 10], [0, 10]],
        [[3, 3], [3, 7], [7, 7], [7, 3]]
    ]];
    let square = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
    let expected_concave = int_path![[-1, -1], [4, -1], [4, 1], [1, 1], [1, 4], [-1, 4]];
    let expected_holed = int_shapes![[
        [[-1, -1], [10, -1], [10, 10], [-1, 10]],
        [[3, 3], [3, 6], [6, 6], [6, 3]]
    ]];
    for preserve_output_collinear in [false, true] {
        let mut solver = IntMinkowskiSolver::with_options(0, preserve_output_collinear);
        let rhs = solver.prepare(&square, FillRule::NonZero).unwrap();
        for _ in 0..3 {
            let lhs = solver.prepare(&concave, FillRule::NonZero).unwrap();
            let result = solver.difference(&lhs, &rhs).unwrap();
            assert_geometry(&result, &expected_concave);
            if preserve_output_collinear {
                assert!(result[0][0].len() > expected_concave.len());
            } else {
                assert_eq!(result[0][0].len(), expected_concave.len());
            }
            let lhs = solver.prepare(&holed, FillRule::NonZero).unwrap();
            assert_geometry(&solver.difference(&lhs, &rhs).unwrap(), &expected_holed);
            assert_geometry(
                &solver.sum(&rhs, &rhs).unwrap(),
                &int_path![[0, 0], [2, 0], [2, 2], [0, 2]],
            );
        }
    }
}

#[test]
fn subtraction_checks_all_bbox_bounds_and_recovers_after_errors() {
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
                    rectangle(-one, I::Wide::ZERO)
                } else {
                    rectangle(I::Wide::ZERO, one)
                };
                let invalid = if positive {
                    rectangle(-two, -one)
                } else {
                    rectangle(one, two)
                };
                let lhs = solver.prepare(&a, FillRule::NonZero).unwrap();
                let valid = solver.prepare(&valid, FillRule::NonZero).unwrap();
                let invalid = solver.prepare(&invalid, FillRule::NonZero).unwrap();
                let before = solver.difference(&lhs, &valid).unwrap();
                assert!(matches!(
                    solver.difference(&lhs, &invalid),
                    Err(MinkowskiError::CoordinateOutOfRange)
                ));
                assert_geometry(&solver.difference(&lhs, &valid).unwrap(), &before);
                let self_difference = solver.difference(&lhs, &lhs).unwrap();
                let expected: IntContour<I> = [(-two, -two), (two, -two), (two, two), (-two, two)]
                    .map(|(x, y)| IntPoint::new(I::from_wide(x), I::from_wide(y)))
                    .into();
                assert_geometry(&self_difference, &expected);
            }
        }
    }

    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn rejects_original_bbox_even_when_retained_geometry_would_fit() {
    let limit = 1_i32 << 30;
    let square = int_path![[0, 0], [2, 0], [2, 2], [0, 2]];
    let remote = int_path![
        [limit - 4, 0],
        [limit - 2, 0],
        [limit - 2, 2],
        [limit - 4, 2]
    ];
    let source = vec![square.clone(), remote.clone(), remote];
    let kernel = int_path![[-5, -2], [-3, -2], [-3, 0], [-5, 0]];
    let mut solver = IntMinkowskiSolver::default();
    let lhs = solver.prepare(&source, FillRule::EvenOdd).unwrap();
    let rhs = solver.prepare(&kernel, FillRule::NonZero).unwrap();
    assert!(matches!(
        solver.difference(&lhs, &rhs),
        Err(MinkowskiError::CoordinateOutOfRange)
    ));
    let retained = solver.prepare(&square, FillRule::NonZero).unwrap();
    assert_geometry(
        &solver.difference(&retained, &rhs).unwrap(),
        &int_path![[3, 0], [7, 0], [7, 4], [3, 4]],
    );
}
