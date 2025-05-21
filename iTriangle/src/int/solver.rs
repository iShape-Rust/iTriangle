use alloc::vec::Vec;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::int::meta::MeshMetaProvider;
use crate::int::monotone::builder::TrianglesBuilder;
use crate::int::triangulation::RawIntTriangulation;
use crate::int::unchecked::IntUncheckedTriangulatable;
use crate::int::validation::Validation;

pub(super) struct ShapesSolver;
pub(super) struct ShapeSolver;
pub(super) struct ContourSolver;

impl ShapesSolver {

    #[inline]
    pub(super) fn triangulate(validation: Validation, shapes: &IntShapes) -> RawIntTriangulation {
        let shapes = shapes.simplify(validation.fill_rule, validation.options);
        Self::uncheck_triangulate(&shapes)
    }

    pub(super) fn uncheck_triangulate(shapes: &IntShapes) -> RawIntTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                first.uncheck_triangulate()
            } else {
                RawIntTriangulation::empty()
            };
        }

        let mut triangles_count = 0;
        let mut points_count = 0;
        for shape in shapes.iter() {
            let meta = shape.meta(0);
            triangles_count += meta.triangles_count;
            points_count += meta.vertices_count;
        }

        let mut triangles = Vec::with_capacity(triangles_count);
        let mut points = Vec::with_capacity(points_count);

        let mut iter = shapes.iter();
        if let Some(first) = iter.next() {
            let mut raw_0 = first.uncheck_triangulate();
            triangles.append(&mut raw_0.triangles);
            points.append(&mut raw_0.points);

            for shape in iter {
                let points_offset = points.len();
                let triangle_offset = triangles.len();
                let mut raw_i = shape.uncheck_triangulate();
                raw_i.shift(points_offset, triangle_offset);

                triangles.append(&mut raw_i.triangles);
                points.append(&mut raw_i.points);
            }
        }

        RawIntTriangulation::new(triangles, points)
    }

    #[inline]
    pub(super) fn triangulate_with_steiner_points(
        validation: Validation,
        shapes: &IntShapes,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        shapes.simplify(validation.fill_rule, validation.options)
            .uncheck_triangulate_with_steiner_points(points)
    }

    pub(super) fn uncheck_triangulate_with_steiner_points(
        shapes: &IntShapes,
        groups: &[Vec<IntPoint>],
    ) -> RawIntTriangulation {
        if shapes.len() <= 1 {
            return if let Some(first) = shapes.first() {
                first.uncheck_triangulate_with_steiner_points(&groups[0])
            } else {
                RawIntTriangulation::empty()
            };
        }

        let mut triangles_count = 0;
        let mut points_count = 0;
        for (i, shape) in shapes.iter().enumerate() {
            let meta = shape.meta(groups[i].len());
            triangles_count += meta.triangles_count;
            points_count += meta.vertices_count;
        }

        let mut triangles = Vec::with_capacity(triangles_count);
        let mut points = Vec::with_capacity(points_count);

        let mut raw_0 = TrianglesBuilder::shape_triangulation(&shapes[0], Some(&groups[0]));
        triangles.append(&mut raw_0.triangles);
        points.append(&mut raw_0.points);

        let mut i = 1;
        while i < shapes.len() {
            let shape = &shapes[i];
            let steiner_points = &groups[i];
            i += 1;

            let points_offset = points.len();
            let triangle_offset = triangles.len();
            let mut raw_i = TrianglesBuilder::shape_triangulation(shape, Some(steiner_points));
            raw_i.shift(points_offset, triangle_offset);

            triangles.append(&mut raw_i.triangles);
            points.append(&mut raw_i.points);
        }

        RawIntTriangulation::new(triangles, points)
    }
}

impl ShapeSolver {

    #[inline]
    pub(super) fn triangulate(validation: Validation, shape: &IntShape) -> RawIntTriangulation {
        let shapes = shape.simplify(validation.fill_rule, validation.options);
        ShapesSolver::uncheck_triangulate(&shapes)
    }

    #[inline]
    pub(super) fn uncheck_triangulate(shape: &IntShape) -> RawIntTriangulation {
        TrianglesBuilder::shape_triangulation(shape, None)
    }

    #[inline]
    pub(super) fn triangulate_with_steiner_points(
        validation: Validation,
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        shape.simplify(validation.fill_rule, validation.options)
            .uncheck_triangulate_with_steiner_points(points)
    }

    #[inline]
    pub(super) fn uncheck_triangulate_with_steiner_points(
        shape: &IntShape,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        if shape.len() <= 1 {
            return if let Some(first) = shape.first() {
                first.uncheck_triangulate_with_steiner_points(points)
            } else {
                RawIntTriangulation::empty()
            };
        }

        TrianglesBuilder::shape_triangulation(shape, Some(points))
    }
}

impl ContourSolver {

    #[inline]
    pub(super) fn triangulate(validation: Validation, contour: &IntContour) -> RawIntTriangulation {
        contour.simplify(validation.fill_rule, validation.options).uncheck_triangulate()
    }

    #[inline]
    pub(super) fn uncheck_triangulate(contour: &IntContour) -> RawIntTriangulation {
        if contour.len() < 3 {
            RawIntTriangulation::empty()
        } else {
            TrianglesBuilder::contour_triangulation(contour, None)
        }
    }

    #[inline]
    pub(super) fn triangulate_with_steiner_points(
        validation: Validation,
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        contour.simplify(validation.fill_rule, validation.options)
            .uncheck_triangulate_with_steiner_points(points)
    }

    #[inline]
    pub(super) fn uncheck_triangulate_with_steiner_points(
        contour: &IntContour,
        points: &[IntPoint],
    ) -> RawIntTriangulation {
        if contour.len() < 3 {
            RawIntTriangulation::empty()
        } else {
            TrianglesBuilder::contour_triangulation(contour, Some(points))
        }
    }
}