use std::collections::HashSet;
use crate::advanced::delaunay::IntDelaunay;
use crate::geom::point::IndexPoint;
use crate::geom::triangle::{Abc, IntTriangle};
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

impl IntDelaunay {

    #[inline]
    pub fn refine_with_circumcenters(self, min_area: u64) -> Self {
        self.refine_with_circumcenters_and_selector::<SelectBiggerAngle>(min_area)
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle(self, min_area: u64) -> Self {
        self.refine_with_circumcenters_and_selector::<SelectObtuseAngle>(min_area)
    }
    
    fn refine_with_circumcenters_and_selector<S: EdgeSelector>(mut self, min_area: u64) -> Self {
        let two_area = min_area << 1;
        let mut unchecked = HashSet::with_capacity(self.triangles.len());
        let mut buffer = Vec::with_capacity(16);

        let mut iter_counter = 0;
        let mut split_counter = self.triangles.len();

        // do 8 times or leave if last time we found only 25%
        while iter_counter < 8 && 4 * split_counter > self.triangles.len() {
            let mut abc_index = 0;
            split_counter = 0;
            while abc_index < self.triangles.len() {
                let abc = &self.triangles[abc_index];
                if let Some(t) = self.select_edge_for_refinement::<S>(two_area, abc) {
                    self.split_triangle(abc_index, t, &mut buffer);
                    self.fix_triangles(&mut buffer, &mut unchecked);
                    debug_assert!(buffer.is_empty());
                    split_counter += 1;
                }
                abc_index += 1;
            }

            iter_counter += 1;
        }

        self
    }

    #[inline]
    fn select_edge_for_refinement<S: EdgeSelector>(&self, min_area: u64, abc: &IntTriangle) -> Option<Abc> {
        let a = abc.vertices[0].point;
        let b = abc.vertices[1].point;
        let c = abc.vertices[2].point;

        let area = Triangle::area_two_point(a, b, c).unsigned_abs();
        if area <= min_area {
            return None;
        }

        S::select(abc)
    }

    #[inline]
    fn split_triangle(&mut self, abc_index: usize, abc: Abc, buffer: &mut Vec<usize>) {
        let pcb_index = abc.v0.neighbor;
        if pcb_index < self.triangles.len() {
            buffer.extend_from_slice(&self.split_triangle_with_neighbor(abc_index, abc, pcb_index));
        } else {
            buffer.extend_from_slice(&self.split_alone_triangle(abc_index, abc));
        }
    }

    fn split_triangle_with_neighbor(&mut self, abc_index: usize, abc: Abc, pcb_index: usize) -> [usize; 4] {
        let p = abc.circumscribed_center();
        let pcb = &self.triangles[pcb_index].abc_by_neighbor(abc_index);

        let m = if pcb.is_contain(p) {
            p
        } else {
            abc.edge_mid_point()
        };

        let m_index = self.points.len();
        self.points.push(m);
        let vm = IndexPoint {
            index: m_index,
            point: m,
        };

        // abc -> abm
        // pcb -> pcm
        let abm_index = abc_index;
        let pcm_index = pcb_index;

        let amc_index = self.triangles.len();
        let pmb_index = amc_index + 1;

        let abm = IntTriangle {
            vertices: [abc.v0.vertex, abc.v1.vertex, vm],
            neighbors: [pmb_index, amc_index, abc.v2.neighbor],
        };

        let amc = IntTriangle {
            vertices: [abc.v0.vertex, vm, abc.v2.vertex],
            neighbors: [pcm_index, abc.v1.neighbor, abm_index],
        };

        let pmb = IntTriangle {
            vertices: [pcb.v0.vertex, vm, pcb.v2.vertex],
            neighbors: [abm_index, pcb.v1.neighbor, pcm_index],
        };

        let pcm = IntTriangle {
            vertices: [pcb.v0.vertex, pcb.v1.vertex, vm],
            neighbors: [amc_index, pmb_index, pcb.v2.neighbor],
        };

        self.update_neighbor(abc.v1.neighbor, abc_index, amc_index);
        self.update_neighbor(abc.v2.neighbor, abc_index, abm_index);

        self.update_neighbor(pcb.v1.neighbor, pcb_index, pmb_index);
        self.update_neighbor(pcb.v2.neighbor, pcb_index, pcm_index);

        self.triangles[abm_index] = abm;
        self.triangles[pcm_index] = pcm;
        self.triangles.push(amc);
        self.triangles.push(pmb);


        [abm_index, pcm_index, amc_index, pmb_index]
    }

    fn split_alone_triangle(&mut self, abc_index: usize, abc: Abc) -> [usize; 2] {
        let m = abc.edge_mid_point();
        let m_index = self.points.len();
        self.points.push(m);
        let vm = IndexPoint {
            index: m_index,
            point: m,
        };

        let abm_index = abc_index;
        let amc_index = self.triangles.len();

        let abm = IntTriangle {
            vertices: [abc.v0.vertex, abc.v1.vertex, vm],
            neighbors: [usize::MAX, amc_index, abc.v2.neighbor],
        };

        let amc = IntTriangle {
            vertices: [abc.v0.vertex, vm, abc.v2.vertex],
            neighbors: [usize::MAX, abc.v1.neighbor, abm_index],
        };

        self.update_neighbor(abc.v1.neighbor, abc_index, amc_index);
        self.update_neighbor(abc.v2.neighbor, abc_index, abm_index);

        self.triangles[abm_index] = abm;
        self.triangles.push(amc);

        [abm_index, amc_index]
    }
}

impl Abc {
    #[inline]
    fn circumscribed_center(&self) -> IntPoint {
        let a = self.v0.vertex.point;
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;
        let ax = a.x as f64;
        let ay = a.y as f64;
        let bx = b.x as f64;
        let by = b.y as f64;
        let cx = c.x as f64;
        let cy = c.y as f64;

        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        let aa = ax * ax + ay * ay;
        let bb = bx * bx + by * by;
        let cc = cx * cx + cy * cy;
        let id = 1.0 / d;

        let fx = (aa * (by - cy) + bb * (cy - ay) + cc * (ay - by)) * id;
        let fy = (aa * (cx - bx) + bb * (ax - cx) + cc * (bx - ax)) * id;

        let x = fx.round() as i32;
        let y = fy.round() as i32;

        IntPoint::new(x, y)
    }

    #[inline]
    fn is_contain(&self, p: IntPoint) -> bool {
        let a = self.v0.vertex.point;
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;

        Triangle::is_contain_point_exclude_borders(p, a, b, c)
    }

    #[inline]
    fn edge_mid_point(&self) -> IntPoint {
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;

        let x = (b.x as i64 + c.x as i64) >> 1;
        let y = (b.y as i64 + c.y as i64) >> 1;

        IntPoint::new(x as i32, y as i32)
    }
}

trait EdgeSelector {
    fn select(abc: &IntTriangle) -> Option<Abc>;
}

struct SelectBiggerAngle {}
struct SelectObtuseAngle {}

impl EdgeSelector for SelectObtuseAngle {

    #[inline]
    fn select(abc: &IntTriangle) -> Option<Abc> {
        let a = abc.vertices[0].point;
        let b = abc.vertices[1].point;
        let c = abc.vertices[2].point;

        let sqr_c = a.sqr_distance(b);
        let sqr_a = b.sqr_distance(c);
        let sqr_b = c.sqr_distance(a);

        if sqr_c > sqr_a + sqr_b {
            Some(abc.abc_by_c())
        } else if sqr_b > sqr_a + sqr_c {
            Some(abc.abc_by_b())
        } else if sqr_a > sqr_b + sqr_c {
            Some(abc.abc_by_a())
        } else {
            None
        }
    }
}

impl EdgeSelector for SelectBiggerAngle {
    #[inline]
    fn select(abc: &IntTriangle) -> Option<Abc> {
        let a = abc.vertices[0].point;
        let b = abc.vertices[1].point;
        let c = abc.vertices[2].point;

        let sqr_c = a.sqr_distance(b);
        let sqr_a = b.sqr_distance(c);
        let sqr_b = c.sqr_distance(a);

        if sqr_c >= sqr_a && sqr_c >= sqr_b {
            Some(abc.abc_by_c())
        } else if sqr_b >= sqr_a && sqr_b >= sqr_c {
            Some(abc.abc_by_b())
        } else {
            Some(abc.abc_by_a())
        }
    }
}