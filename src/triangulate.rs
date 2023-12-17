use i_float::fix_vec::FixVec;
use i_overlay::bool::fill_rule::FillRule;
use i_overlay::ext::simplify::Simplify;
use i_shape::fix_path::FixPathExtension;
use i_shape::fix_shape::FixShape;
use crate::delaunay::convex::{ConvexPath, ConvexSide};
use crate::delaunay::delaunay::Delaunay;
use crate::flip_shape::Flip;

pub struct Triangulation {
    pub points: Vec<FixVec>,
    pub indices: Vec<usize>
}

pub trait Triangulate {

    fn to_triangulation(&self, validate_rule: Option<FillRule>) -> Triangulation;

    fn into_triangulation(self, validate_rule: Option<FillRule>) -> Triangulation;

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>) -> Vec<ConvexPath>;

    fn into_convex_polygons(self, validate_rule: Option<FillRule>) -> Vec<ConvexPath>;

    fn to_delaunay(&self) -> Option<Delaunay>;

    fn into_delaunay(self) -> Option<Delaunay>;
}

impl Triangulate for FixShape {

    fn to_triangulation(&self, validate_rule: Option<FillRule>) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            shape_to_triangulation(&self, fill_rule)
        } else {
            if let Some(delaunay) = self.to_flip().delaunay() {
                delaunay.to_triangulation(0)
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
    }

    fn into_triangulation(self, validate_rule: Option<FillRule>) -> Triangulation {
        if let Some(fill_rule) = validate_rule {
            shape_to_triangulation(&self, fill_rule)
        } else {
            if let Some(delaunay) = self.into_delaunay() {
                delaunay.to_triangulation(0)
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
    }

    fn to_convex_polygons(&self, validate_rule: Option<FillRule>) -> Vec<ConvexPath> {
        if let Some(fill_rule) = validate_rule {
            shape_to_convex_polygons(self, fill_rule)
        } else {
            if let Some(delaunay) = self.to_delaunay() {
                delaunay.to_convex_polygons()
            } else {
                Vec::new()
            }
        }
    }

    fn into_convex_polygons(self, validate_rule: Option<FillRule>) -> Vec<ConvexPath> {
        if let Some(fill_rule) = validate_rule {
            shape_into_convex_polygons(self, fill_rule)
        } else {
            if let Some(delaunay) = self.into_delaunay() {
                delaunay.to_convex_polygons()
            } else {
                Vec::new()
            }
        }
    }

    fn to_delaunay(&self) -> Option<Delaunay> {
        self.to_flip().delaunay()
    }

    fn into_delaunay(self) -> Option<Delaunay> {
        self.into_flip().delaunay()
    }

}

fn shape_to_triangulation(shape: &FixShape, fill_rule: FillRule) -> Triangulation {
    let shapes = shape.simplify(fill_rule);

    let mut points = Vec::new();
    let mut indices = Vec::new();

    for shape in shapes {
        if let Some(delaunay) = shape.into_delaunay() {
            let sub_triangulation = delaunay.to_triangulation(points.len());

            let mut sub_indices = sub_triangulation.indices;
            let mut sub_points = sub_triangulation.points;

            indices.append(&mut sub_indices);
            points.append(&mut sub_points);
        }
    }

    Triangulation { points, indices }
}


fn shape_into_convex_polygons(shape: FixShape, fill_rule: FillRule) -> Vec<ConvexPath> {
    let mut shapes = shape.simplify(fill_rule);
    if shapes.len() == 1 && shapes[0].is_convex_polygon() {
        let mut paths = shapes.pop().unwrap().paths;
        let mut path = paths.pop().unwrap();
        path.remove_degenerates();
        if path.area() < 0 {
            path.reverse()
        }

        let side = vec![ConvexSide::Outer; path.len()];

        let polygon = ConvexPath { path, side };

        [polygon].to_vec()
    } else {
        shapes_to_convex_polygons(shapes)
    }
}

fn shape_to_convex_polygons(shape: &FixShape, fill_rule: FillRule) -> Vec<ConvexPath> {
    shapes_to_convex_polygons(shape.simplify(fill_rule))
}

fn shapes_to_convex_polygons(shapes: Vec<FixShape>) -> Vec<ConvexPath> {
    let mut polygons = Vec::new();

    for shape in shapes {
        if let Some(delaunay) = shape.into_delaunay() {
            let mut sub_polygons = delaunay.to_convex_polygons();
            polygons.append(&mut sub_polygons);
        }
    }

    polygons
}