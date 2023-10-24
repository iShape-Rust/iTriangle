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
                delaunay.into_triangulation()
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
                let sub_indices = delaunay.triangles_indices_shifted(offset);
                indices.extend(sub_indices);

                let sub_points = delaunay.points();
                offset += sub_points.len();

                points.extend(sub_points);
            }
        }

        return Triangulation { points, indices }
    }

    fn into_triangulation(self, validate: bool) -> Triangulation {
        if !validate {
            return if let Some(delaunay) = self.into_delaunay() {
                delaunay.into_triangulation()
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
        let shapes = self.resolve_self_intersection();

        let mut points = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        for shape in shapes {
            if let Some(delaunay) = shape.into_delaunay() {
                let sub_triangulation = delaunay.into_shifted_triangulation(offset);

                let mut sub_indices = sub_triangulation.indices;
                let mut sub_points = sub_triangulation.points;

                offset += sub_points.len();

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