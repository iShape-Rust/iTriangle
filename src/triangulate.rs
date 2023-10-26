use i_float::fix_vec::FixVec;
use i_overlay::bool::self_intersection::SelfIntersection;
use i_shape::fix_shape::FixShape;
use crate::delaunay::delaunay::Delaunay;
use crate::flip_shape::Flip;

pub struct Triangulation {
    pub points: Vec<FixVec>,
    pub indices: Vec<usize>
}

pub trait Triangulate {

    fn to_triangulation(&self, validate: bool) -> Triangulation;

    fn into_triangulation(self, validate: bool) -> Triangulation;

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
        let shapes = self.resolve_self_intersection();

        let mut points = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        for shape in shapes.iter() {
            if let Some(delaunay) = shape.to_delaunay() {
                let sub_triangulation = delaunay.to_triangulation(offset);

                let mut sub_indices = sub_triangulation.indices;
                let mut sub_points = sub_triangulation.points;

                offset += sub_points.len();

                indices.append(&mut sub_indices);
                points.append(&mut sub_points);
            }
        }

        return Triangulation { points, indices }
    }

    fn into_triangulation(self, validate: bool) -> Triangulation {
        if !validate {
            return if let Some(delaunay) = self.into_delaunay() {
                delaunay.to_triangulation(0)
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
        let shapes = self.resolve_self_intersection();

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

        return Triangulation { points, indices }
    }

    fn to_delaunay(&self) -> Option<Delaunay> {
        self.to_flip().delaunay()
    }

    fn into_delaunay(self) -> Option<Delaunay> {
        self.into_flip().delaunay()
    }

}