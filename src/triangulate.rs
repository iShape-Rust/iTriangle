use i_float::fix_vec::FixVec;
use i_overlay::bool::self_intersection::SelfIntersection;
use i_shape::fix_shape::FixShape;
use crate::delaunay::convex::ConvexPath;
use crate::delaunay::delaunay::Delaunay;
use crate::flip_shape::Flip;

pub struct Triangulation {
    pub points: Vec<FixVec>,
    pub indices: Vec<usize>
}

pub trait Triangulate {

    fn to_triangulation(&self, validate: bool) -> Triangulation;

    fn into_triangulation(self, validate: bool) -> Triangulation;

    fn to_convex_polygons(&self, validate: bool) -> Vec<ConvexPath>;

    fn into_convex_polygons(self, validate: bool) -> Vec<ConvexPath>;

    fn to_delaunay(&self) -> Option<Delaunay>;

    fn into_delaunay(self) -> Option<Delaunay>;
}

impl Triangulate for FixShape {

    fn to_triangulation(&self, validate: bool) -> Triangulation {
        if !validate {
            return if let Some(delaunay) = self.to_flip().delaunay() {
                delaunay.to_triangulation(0)
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
        shape_to_triangulation(&self)
    }

    fn into_triangulation(self, validate: bool) -> Triangulation {
        if !validate {
            return if let Some(delaunay) = self.into_delaunay() {
                delaunay.to_triangulation(0)
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
        shape_to_triangulation(&self)
    }

    fn to_convex_polygons(&self, validate: bool) -> Vec<ConvexPath> {
        if !validate {
            return if let Some(delaunay) = self.to_delaunay() {
                delaunay.to_convex_polygons()
            } else {
                Vec::new()
            }
        }
        shape_to_convex_polygons(&self)
    }

    fn into_convex_polygons(self, validate: bool) -> Vec<ConvexPath> {
        if !validate {
            return if let Some(delaunay) = self.into_delaunay() {
                delaunay.to_convex_polygons()
            } else {
                Vec::new()
            }
        }

        shape_to_convex_polygons(&self)
    }

    fn to_delaunay(&self) -> Option<Delaunay> {
        self.to_flip().delaunay()
    }

    fn into_delaunay(self) -> Option<Delaunay> {
        self.into_flip().delaunay()
    }

}

fn shape_to_triangulation(shape: &FixShape) -> Triangulation {
    let shapes = shape.resolve_self_intersection();

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

fn shape_to_convex_polygons(shape: &FixShape) -> Vec<ConvexPath> {
    let shapes = shape.resolve_self_intersection();

    let mut polygons = Vec::new();

    for shape in shapes {
        if let Some(delaunay) = shape.into_delaunay() {
            let mut sub_polygons = delaunay.to_convex_polygons();
            polygons.append(&mut sub_polygons);
        }
    }

    polygons
}