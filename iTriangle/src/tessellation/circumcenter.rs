use crate::advanced::bitset::IndexBitSet;
use crate::advanced::delaunay::{DelaunayRefine, IntDelaunay};
use crate::geom::point::IndexPoint;
use crate::geom::triangle::{Abc, IntTriangle};
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

impl<I: IntNumber> IntDelaunay<I> {
    #[inline]
    pub fn refine_with_circumcenters(mut self, min_area: I::WideUInt) -> Self {
        self.refine_with_circumcenters_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_mut(&mut self, min_area: I::WideUInt) {
        self.refine_with_circumcenters_and_selector::<SelectBiggerAngle>(min_area);
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle(mut self, min_area: I::WideUInt) -> Self {
        self.refine_with_circumcenters_by_obtuse_angle_mut(min_area);
        self
    }

    #[inline]
    pub fn refine_with_circumcenters_by_obtuse_angle_mut(&mut self, min_area: I::WideUInt) {
        self.refine_with_circumcenters_and_selector::<SelectObtuseAngle>(min_area)
    }

    fn refine_with_circumcenters_and_selector<S: EdgeSelector<I>>(
        &mut self,
        min_area: I::WideUInt,
    ) {
        let two_area = min_area << 1;

        let mut bitset = IndexBitSet::with_size(self.triangles.len());
        let mut buffer = Vec::with_capacity(self.triangles.len().max(16));

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
                    self.triangles.fix_triangles(&mut buffer, &mut bitset);
                    debug_assert!(buffer.is_empty());
                    split_counter += 1;
                }
                abc_index += 1;
            }

            iter_counter += 1;
        }
    }

    #[inline]
    fn select_edge_for_refinement<S: EdgeSelector<I>>(
        &self,
        min_area: I::WideUInt,
        abc: &IntTriangle<I>,
    ) -> Option<Abc<I>> {
        let a = abc.vertices[0].point;
        let b = abc.vertices[1].point;
        let c = abc.vertices[2].point;

        let area = Triangle::area_two(a, b, c).unsigned_abs();
        if area <= min_area {
            return None;
        }

        S::select(abc)
    }

    #[inline]
    fn split_triangle(&mut self, abc_index: usize, abc: Abc<I>, buffer: &mut Vec<usize>) {
        let pcb_index = abc.v0.neighbor;
        if pcb_index < self.triangles.len() {
            buffer.extend_from_slice(&self.split_triangle_with_neighbor(abc_index, abc, pcb_index));
        } else {
            buffer.extend_from_slice(&self.split_alone_triangle(abc_index, abc));
        }
    }

    fn split_triangle_with_neighbor(
        &mut self,
        abc_index: usize,
        abc: Abc<I>,
        pcb_index: usize,
    ) -> [usize; 4] {
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

        self.triangles
            .update_neighbor(abc.v1.neighbor, abc_index, amc_index);
        self.triangles
            .update_neighbor(abc.v2.neighbor, abc_index, abm_index);

        self.triangles
            .update_neighbor(pcb.v1.neighbor, pcb_index, pmb_index);
        self.triangles
            .update_neighbor(pcb.v2.neighbor, pcb_index, pcm_index);

        self.triangles[abm_index] = abm;
        self.triangles[pcm_index] = pcm;
        self.triangles.push(amc);
        self.triangles.push(pmb);

        [abm_index, pcm_index, amc_index, pmb_index]
    }

    fn split_alone_triangle(&mut self, abc_index: usize, abc: Abc<I>) -> [usize; 2] {
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

        self.triangles
            .update_neighbor(abc.v1.neighbor, abc_index, amc_index);
        self.triangles
            .update_neighbor(abc.v2.neighbor, abc_index, abm_index);

        self.triangles[abm_index] = abm;
        self.triangles.push(amc);

        [abm_index, amc_index]
    }
}

impl<I: IntNumber> Abc<I> {
    #[inline]
    fn circumscribed_center(&self) -> IntPoint<I> {
        let a = self.v0.vertex.point;
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;
        let ax = a.x.to_f64();
        let ay = a.y.to_f64();
        let bx = b.x.to_f64();
        let by = b.y.to_f64();
        let cx = c.x.to_f64();
        let cy = c.y.to_f64();

        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        let aa = ax * ax + ay * ay;
        let bb = bx * bx + by * by;
        let cc = cx * cx + cy * cy;
        let id = 1.0 / d;

        let fx = (aa * (by - cy) + bb * (cy - ay) + cc * (ay - by)) * id;
        let fy = (aa * (cx - bx) + bb * (ax - cx) + cc * (bx - ax)) * id;

        let x = I::from_float(fx);
        let y = I::from_float(fy);

        IntPoint::new(x, y)
    }

    #[inline]
    fn is_contain(&self, p: IntPoint<I>) -> bool {
        let a = self.v0.vertex.point;
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;

        Triangle::is_contain_exclude_borders(p, a, b, c)
    }

    #[inline]
    fn edge_mid_point(&self) -> IntPoint<I> {
        let b = self.v1.vertex.point;
        let c = self.v2.vertex.point;

        let x = (b.x.to_wide() + c.x.to_wide()) >> 1;
        let y = (b.y.to_wide() + c.y.to_wide()) >> 1;

        IntPoint::new(I::from_wide(x), I::from_wide(y))
    }
}

trait EdgeSelector<I: IntNumber> {
    fn select(abc: &IntTriangle<I>) -> Option<Abc<I>>;
}

struct SelectBiggerAngle {}
struct SelectObtuseAngle {}

impl<I: IntNumber> EdgeSelector<I> for SelectObtuseAngle {
    #[inline]
    fn select(abc: &IntTriangle<I>) -> Option<Abc<I>> {
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

impl<I: IntNumber> EdgeSelector<I> for SelectBiggerAngle {
    #[inline]
    fn select(abc: &IntTriangle<I>) -> Option<Abc<I>> {
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
