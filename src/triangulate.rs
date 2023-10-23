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

    fn triangulate(&self, validate: bool) -> Triangulation;

    fn delaunay(&self) -> Option<Delaunay>;

}

impl Triangulate for FixShape {

    fn triangulate(&self, validate: bool) -> Triangulation {
        if !validate {
            return if let Some(delaunay) = self.flip().delaunay() {
                let indices = delaunay.triangles_indices();
                let points = delaunay.points();

                Triangulation { points, indices }
            } else {
                Triangulation { points: Vec::new(), indices: Vec::new() }
            }
        }
        let shapes = self.resolve_self_intersection();

        let mut points = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        for shape in shapes.iter() {
            if let Some(delaunay) = shape.delaunay() {
                let sub_indices = delaunay.triangles_indices_shifted(offset);
                indices.extend(sub_indices);

                let sub_points = delaunay.points();
                offset += sub_points.len();

                points.extend(sub_points);
            }
        }

        return Triangulation { points, indices }
    }

    fn delaunay(&self) -> Option<Delaunay> {
        self.flip().delaunay()
    }
}