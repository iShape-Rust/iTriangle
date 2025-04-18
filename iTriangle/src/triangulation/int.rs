use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ContourDirection;
use i_overlay::core::simplify::Simplify;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::count::PointsCount;
use i_overlay::i_shape::int::path::{IntPath, PointPathExtension};
use i_overlay::i_shape::int::shape::{IntShape};
use crate::delaunay::triangulate::ShapeTriangulate;

#[derive(Debug)]
pub struct Triangulation {
    pub points: Vec<IntPoint>,
    pub indices: Vec<usize>,
}

pub trait IntTriangulate {
    fn to_triangulation(&self, validate_rule: Option<FillRule>, min_area: usize) -> Triangulation;

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>, min_area: usize) -> Vec<IntPath>;
}

trait UnsafeTriangulate {
    fn triangulation(&self) -> Triangulation;
    fn convex_polygons(&self) -> Vec<IntPath>;
}

impl UnsafeTriangulate for Vec<IntShape> {
    fn triangulation(&self) -> Triangulation {
        let count = self.points_count();
        if count < 3 {
            return Triangulation { points: vec![], indices: vec![] };
        }

        let mut points = Vec::with_capacity(count);
        let mut indices = Vec::with_capacity(3 * (count - 2));

        for shape in self.iter() {
            if let Some(delaunay) = shape.delaunay() {
                let sub_triangulation = delaunay.to_triangulation(points.len());

                let mut sub_indices = sub_triangulation.indices;
                let mut sub_points = sub_triangulation.points;

                indices.append(&mut sub_indices);
                points.append(&mut sub_points);
            }
        }

        Triangulation { points, indices }
    }

    fn convex_polygons(&self) -> Vec<IntPath> {
        if self.len() == 1 && self[0].len() == 1 && self[0][0].is_convex() {
            self[0].clone()
        } else {
            let mut polygons = Vec::new();

            for shape in self.iter() {
                if let Some(delaunay) = shape.delaunay() {
                    let mut sub_polygons = delaunay.to_convex_polygons();
                    polygons.append(&mut sub_polygons);
                }
            }

            polygons
        }
    }
}

impl IntTriangulate for IntShape {
    fn to_triangulation(&self, validate_rule: Option<FillRule>, min_area: usize) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule, ContourDirection::CounterClockwise, min_area).triangulation()
        } else {
            self.triangulation()
        }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>, min_area: usize) -> Vec<IntPath> {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule, ContourDirection::CounterClockwise, min_area).convex_polygons()
        } else if let Some(delaunay) = self.delaunay() {
            delaunay.to_convex_polygons()
        } else {
            Vec::new()
        }
    }
}

impl IntTriangulate for Vec<IntShape> {
    fn to_triangulation(&self, validate_rule: Option<FillRule>, min_area: usize) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule, ContourDirection::CounterClockwise, min_area).triangulation()
        } else {
            self.triangulation()
        }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>, min_area: usize) -> Vec<IntPath> {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule, ContourDirection::CounterClockwise, min_area).convex_polygons()
        } else {
            self.convex_polygons()
        }
    }
}