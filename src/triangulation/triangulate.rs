use i_float::fix_vec::FixVec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::simplify::Simplify;
use i_shape::fix_path::FixPath;
use i_shape::fix_shape::FixShape;
use crate::delaunay::triangulate::ShapeTriangulate;

#[derive(Debug)]
pub struct Triangulation {
    pub points: Vec<FixVec>,
    pub indices: Vec<usize>
}

pub trait Triangulate {

    fn to_triangulation(&self, validate_rule: Option<FillRule>) -> Triangulation;

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>) -> Vec<FixPath>;

}

trait UnsafeTriangulate {
    fn triangulation(&self) -> Triangulation;
    fn convex_polygons(&self) -> Vec<FixPath>;
}

impl UnsafeTriangulate for Vec<FixShape> {

    fn triangulation(&self) -> Triangulation {
        let mut points = Vec::new();
        let mut indices = Vec::new();

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

    fn convex_polygons(&self) -> Vec<FixPath> {
        if self.len() == 1 && self[0].is_convex_polygon() {
            [self[0].paths[0].clone()].to_vec()
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

impl Triangulate for FixShape {

    fn to_triangulation(&self, validate_rule: Option<FillRule>) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule).triangulation()
        } else {
            self.triangulation()
        }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>) -> Vec<FixPath> {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule).convex_polygons()
        } else if let Some(delaunay) = self.delaunay() {
            delaunay.to_convex_polygons()
        } else {
            Vec::new()
        }
    }
}

impl Triangulate for Vec<FixShape> {

    fn to_triangulation(&self, validate_rule: Option<FillRule>) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule).triangulation()
        } else {
            self.triangulation()
        }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>) -> Vec<FixPath> {
        if let Some(fill_rule) = validate_rule {
            self.simplify(fill_rule).convex_polygons()
        } else {
            self.convex_polygons()
        }
    }
}