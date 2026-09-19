use i_triangle::float::custom::CustomTriangulatable;
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use i_triangle::float::unchecked::UncheckedTriangulatable;
use i_triangle::float::uniform::UniformTriangulatable;
use i_triangle::i_overlay::core::integer::OverlayInt;
use i_triangle::i_overlay::i_float::float::number::FloatNumber;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn check_coordinate_budget<F: FloatNumber, I: OverlayInt>(radius: F) {
    let contour = vec![
        [-radius, -radius],
        [radius, -radius],
        [radius, radius],
        [-radius, radius],
    ];
    let direct = contour.triangulate_as::<I>();
    let limit = I::ONE << (I::BITS - 3);
    for point in direct.raw.points() {
        assert!(point.x >= -limit && point.x <= limit);
        assert!(point.y >= -limit && point.y <= limit);
    }
    let expected = direct.to_triangulation::<u32>();
    assert_eq!(expected.indices.len(), 6);

    let mut triangulator = Triangulator::<u32, I>::default();
    let buffered = triangulator.triangulate(&contour);
    let unchecked = triangulator.uncheck_triangulate(&contour);
    let custom = contour
        .custom_triangulate_as::<I>(Default::default())
        .to_triangulation::<u32>();
    let unchecked_direct = contour
        .unchecked_triangulate_as::<I>()
        .to_triangulation::<u32>();
    for mesh in [buffered, unchecked, custom, unchecked_direct] {
        assert_eq!(mesh.indices.len(), expected.indices.len());
        assert_eq!(mesh.points.len(), expected.points.len());
        for point in mesh.points {
            assert!(point[0].is_finite() && point[1].is_finite());
            assert!(expected.points.contains(&point));
        }
    }
}

#[test]
fn all_engines_share_the_coordinate_budget_at_rounding_and_scalar_limits() {
    // Just below a power of two: the default adapter can round beyond the budget.
    for radius in [1.999_999_f32, f32::MAX_COORDINATE, f32::MIN_POSITIVE] {
        check_coordinate_budget::<_, i16>(radius);
        check_coordinate_budget::<_, i32>(radius);
        check_coordinate_budget::<_, i64>(radius);
    }
    for radius in [
        1.999_999_999_999_999_f64,
        f64::MAX_COORDINATE,
        f64::MIN_POSITIVE,
    ] {
        check_coordinate_budget::<_, i16>(radius);
        check_coordinate_budget::<_, i32>(radius);
        check_coordinate_budget::<_, i64>(radius);
    }
}

#[test]
fn invalid_coordinates_panic_in_all_float_entry_points() {
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX_COORDINATE * 2.0,
    ] {
        // Interior NaN must not disappear during min/max bounds accumulation.
        let contour = vec![[0.0, 0.0], [invalid, 1.0], [2.0, 2.0], [0.0, 2.0]];
        let shape = vec![contour.clone()];
        let shapes = [shape.clone()];
        assert!(catch_unwind(|| shape.triangulate()).is_err());
        assert!(catch_unwind(|| shape.custom_triangulate(Default::default())).is_err());
        assert!(catch_unwind(|| shape.unchecked_triangulate()).is_err());
        assert!(catch_unwind(|| shapes.triangulate()).is_err());
        assert!(catch_unwind(|| shapes.custom_triangulate(Default::default())).is_err());
        assert!(catch_unwind(|| shapes.unchecked_triangulate()).is_err());
        assert!(catch_unwind(|| contour.triangulate()).is_err());
        assert!(catch_unwind(|| contour.custom_triangulate(Default::default())).is_err());
        assert!(catch_unwind(|| contour.unchecked_triangulate()).is_err());
        assert!(catch_unwind(|| contour.uniform_triangulate(1.0)).is_err());
        let mesh = Triangulation {
            points: vec![[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]],
            indices: vec![0u32, 1, 2],
        };
        assert!(catch_unwind(|| mesh.locate_points::<f64, i32>(&[[invalid, 1.0]])).is_err());
    }
}

#[test]
fn reusable_triangulator_preserves_output_on_invalid_input_and_recovers() {
    let valid = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    let invalid = vec![[0.0, 0.0], [f64::MAX_COORDINATE * 2.0, 1.0], [2.0, 2.0]];
    let mut triangulator = Triangulator::<u32>::default();
    let mut output = triangulator.triangulate(&valid);
    let previous = output.clone();
    for unchecked in [false, true] {
        assert!(catch_unwind(AssertUnwindSafe(|| {
            if unchecked {
                triangulator.uncheck_triangulate_into(&invalid, &mut output);
            } else {
                triangulator.triangulate_into(&invalid, &mut output);
            }
        }))
        .is_err());
        assert_eq!(output.points, previous.points);
        assert_eq!(output.indices, previous.indices);
        if unchecked {
            triangulator.uncheck_triangulate_into(&valid, &mut output);
        } else {
            triangulator.triangulate_into(&valid, &mut output);
        }
        output.validate(16.0, 1e-10);
    }
}

#[test]
fn empty_input_clears_reused_output() {
    let empty: Vec<[f64; 2]> = Vec::new();
    assert!(empty.triangulate().raw.points().is_empty());
    assert!(empty
        .custom_triangulate(Default::default())
        .raw
        .points()
        .is_empty());
    assert!(empty.unchecked_triangulate().raw.points().is_empty());
    assert!(empty
        .uniform_triangulate(2.0)
        .to_triangulation::<u32>()
        .indices
        .is_empty());
    let mut triangulator = Triangulator::<u32>::default();
    let valid = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
    let mut output = triangulator.triangulate(&valid);
    triangulator.triangulate_into(&empty, &mut output);
    assert!(output.points.is_empty() && output.indices.is_empty());
    triangulator.uncheck_triangulate_into(&valid, &mut output);
    triangulator.uncheck_triangulate_into(&empty, &mut output);
    assert!(output.points.is_empty() && output.indices.is_empty());
}

#[test]
fn self_intersections_match_overlay_on_the_same_integer_grid() {
    use i_triangle::i_overlay::core::fill_rule::FillRule;
    use i_triangle::i_overlay::float::simplify::SimplifyShape;
    use i_triangle::i_overlay::i_shape::float::area::Area;

    // With i16 the intersection rounding changes area by more than the old
    // test tolerance relative to i32. The reference must use the selected engine.
    fn check<I: OverlayInt>() {
        let contour = [
            [-5.0f32, -2.0],
            [4.0, 5.0],
            [-3.0, 1.0],
            [0.0, 3.0],
            [0.0, -3.0],
            [0.0, 4.0],
        ];
        let area = contour.simplify_shape_as::<I>(FillRule::NonZero).area();
        let mut triangulator = Triangulator::<u32, I>::default();
        for delaunay in [false, true] {
            triangulator.delaunay(delaunay);
            triangulator.triangulate(&contour).validate(area, 0.000_01);
        }
        contour
            .triangulate_as::<I>()
            .to_triangulation::<u32>()
            .validate(area, 0.000_01);
    }
    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn nested_empty_contours_clear_buffers_without_leaving_stale_geometry() {
    let empty: Vec<Vec<Vec<[f64; 2]>>> = vec![vec![vec![]], vec![vec![], vec![]]];
    let mut triangulator = Triangulator::<u32>::default();
    let valid = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
    let mut output = triangulator.triangulate(&valid);
    triangulator.triangulate_into(&empty, &mut output);
    assert!(output.points.is_empty() && output.indices.is_empty());
    triangulator.uncheck_triangulate_into(&valid, &mut output);
    triangulator.uncheck_triangulate_into(&empty, &mut output);
    assert!(output.points.is_empty() && output.indices.is_empty());
}
