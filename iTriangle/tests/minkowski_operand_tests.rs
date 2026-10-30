use i_triangle::i_overlay::core::fill_rule::FillRule;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::core::overlay::Overlay;
use i_triangle::i_overlay::core::overlay_rule::OverlayRule;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::i_overlay::i_float::int::rect::IntRect;
use i_triangle::i_overlay::i_shape::int::path::ContourExtension;
use i_triangle::i_overlay::i_shape::int::shape::IntContour;
use i_triangle::i_overlay::i_shape::{int_path, int_shapes};
use i_triangle::int::minkowski::IntMinkowskiOperand;

#[test]
fn prepares_borrowed_contours_for_all_integer_types() {
    fn check<I: OverlayInt + From<i16>>() {
        let contour: IntContour<I> = [(0, 0), (10, 0), (10, 10), (0, 10)]
            .map(|(x, y)| IntPoint::new(I::from(x), I::from(y)))
            .into();
        let operand =
            IntMinkowskiOperand::with_source(contour.as_slice(), FillRule::NonZero).unwrap();

        assert!(operand.bbox() == IntRect::new(I::from(0), I::from(10), I::from(0), I::from(10)));
        assert_eq!(operand.convex_polygons().len(), 1);
        let polygon = &operand.convex_polygons()[0];
        assert_eq!(polygon.len(), contour.len());
        assert!(polygon.iter().all(|p| contour.contains(p)));
        assert!(polygon.is_convex());
        assert!(!polygon.is_clockwise_ordered());
    }

    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn empty_and_degenerate_sources_have_no_operand() {
    let contours: Vec<IntContour<i32>> = vec![
        vec![],
        int_path![[0, 0]],
        int_path![[0, 0], [10, 0]],
        int_path![[0, 0], [5, 0], [10, 0]],
    ];
    for contour in &contours {
        assert!(IntMinkowskiOperand::with_source(contour, FillRule::NonZero).is_none());
    }
    assert!(IntMinkowskiOperand::with_source(&contours, FillRule::NonZero).is_none());
}

#[test]
fn decomposition_preserves_concavity_holes_and_separate_components() {
    let shapes = int_shapes![
        [
            [[0, 0], [20, 0], [20, 10], [10, 10], [10, 20], [0, 20]],
            [[2, 2], [2, 6], [6, 6], [6, 2]]
        ],
        [[[30, 0], [40, 0], [40, 10], [30, 10]]]
    ];
    let operand = IntMinkowskiOperand::with_source(&shapes, FillRule::NonZero).unwrap();
    assert_eq!(operand.bbox(), IntRect::new(0, 40, 0, 20));
    for polygon in operand.convex_polygons() {
        assert!(polygon.len() >= 3);
        assert!(polygon.is_convex());
        assert!(!polygon.is_clockwise_ordered());
    }

    let difference = Overlay::from_subj_and_clip(&shapes, operand.convex_polygons())
        .overlay(OverlayRule::Xor, FillRule::NonZero);
    assert!(difference.is_empty());
}

#[test]
fn fill_rule_does_not_shrink_the_source_bbox() {
    let square = int_path![[0, 0], [10, 0], [10, 10], [0, 10]];
    let remote = int_path![[100, 100], [110, 100], [110, 110], [100, 110]];
    let paths = vec![square.clone(), remote.clone(), remote.clone()];
    let operand = IntMinkowskiOperand::with_source(&paths, FillRule::EvenOdd).unwrap();

    assert_eq!(operand.bbox(), IntRect::new(0, 110, 0, 110));
    assert!(
        Overlay::from_subj_and_clip(&square, operand.convex_polygons())
            .overlay(OverlayRule::Xor, FillRule::NonZero)
            .is_empty()
    );

    assert!(
        IntMinkowskiOperand::with_source(&[remote.clone(), remote], FillRule::EvenOdd).is_none()
    );
    assert!(IntMinkowskiOperand::with_source(&square, FillRule::Positive).is_some());
    assert!(IntMinkowskiOperand::with_source(&square, FillRule::Negative).is_none());
}
