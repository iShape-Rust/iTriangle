use i_float::fix_vec::FixVec;
use i_shape::triangle::Triangle;
use crate::delaunay::index_buffer::IndexBuffer;
use crate::delaunay::triangle::DTriangle;
use crate::delaunay::vertex::DVertex;
use crate::index::{NIL_INDEX, Index};
use crate::triangulate::Triangulation;

pub struct Delaunay {
    points: Vec<FixVec>,
    triangles: Vec<DTriangle>,
}

impl Delaunay {
    pub fn into_triangulation(self) -> Triangulation {
        let indices = self.triangles_indices();
        Triangulation { points: self.points, indices }
    }

    pub fn into_shifted_triangulation(self, shifted: usize) -> Triangulation {
        let indices = self.triangles_indices_shifted(shifted);
        Triangulation { points: self.points, indices }
    }

    pub fn points(&self) -> &Vec<FixVec> {
        &self.points
    }

    pub(super) fn new(points: Vec<FixVec>, triangles: Vec<DTriangle>) -> Self {
        Self { points, triangles }
    }

    pub fn triangles_indices(&self) -> Vec<usize> {
        let mut result = vec![NIL_INDEX; 3 * self.triangles.len()];
        let mut j = 0;
        let pointer = result.as_mut_ptr();
        for triangle in self.triangles.iter() {
            unsafe {
                *pointer.add(j) = triangle.vertices[0].index;
                *pointer.add(j + 1) = triangle.vertices[1].index;
                *pointer.add(j + 2) = triangle.vertices[2].index;
            }
            j += 3;
        }

        return result;
    }

    pub fn triangles_indices_shifted(&self, shifted: usize) -> Vec<usize> {
        let mut result = vec![NIL_INDEX; 3 * self.triangles.len()];
        let mut j = 0;
        let pointer = result.as_mut_ptr();
        for triangle in self.triangles.iter() {
            unsafe {
                *pointer.add(j) = triangle.vertices[0].index + shifted;
                *pointer.add(j + 1) = triangle.vertices[1].index + shifted;
                *pointer.add(j + 2) = triangle.vertices[2].index + shifted;
            }
            j += 3;
        }

        return result;
    }

    pub(crate) fn build(&mut self) {
        let count = self.triangles.len();
        let mut visit_marks = vec![false; count];
        let visit_marks_ptr = visit_marks.as_mut_ptr();

        let mut visit_index = 0;

        let mut origin = Vec::with_capacity(64);
        origin.push(0);

        let mut buffer = Vec::with_capacity(64);

        while !origin.is_empty() {
            let mut j = 0;
            while j < origin.len() {
                let i = origin[j];
                j += 1;
                unsafe {
                    let mut triangle = *self.triangles.get_unchecked(i);
                    *visit_marks_ptr.add(i) = true;
                    for k in 0..3 {
                        let neighbor_index = triangle.neighbors[k];
                        if neighbor_index.is_nil() {
                            continue;
                        }
                        let mut neighbor = *self.triangles.get_unchecked(neighbor_index);
                        if self.swap(triangle, neighbor) {
                            triangle = *self.triangles.get_unchecked(triangle.index);
                            neighbor = *self.triangles.get_unchecked(neighbor.index);

                            let tna = triangle.na();
                            if tna.is_not_nil() && tna != neighbor.index {
                                buffer.push(tna);
                            }

                            let tnb = triangle.nb();
                            if tnb.is_not_nil() && tnb != neighbor.index {
                                buffer.push(tnb);
                            }

                            let tnc = triangle.nc();
                            if tnc.is_not_nil() && tnc != neighbor.index {
                                buffer.push(tnc);
                            }

                            let nna = neighbor.na();
                            if nna.is_not_nil() && nna != triangle.index {
                                buffer.push(nna);
                            }

                            let nnb = neighbor.nb();
                            if nnb.is_not_nil() && nnb != triangle.index {
                                buffer.push(nnb);
                            }

                            let nnc = neighbor.nc();
                            if nnc.is_not_nil() && nnc != triangle.index {
                                buffer.push(nnc);
                            }
                        }
                    }
                }
            }

            if buffer.len() == 0 && visit_index < count {
                visit_index += 1;
                while visit_index < count {
                    unsafe {
                        if *visit_marks.get_unchecked(visit_index) == false {
                            buffer.push(visit_index);
                            break;
                        }
                    }
                    visit_index += 1;
                }
            }

            buffer.clear();

            let temp = origin;
            origin = buffer;
            buffer = temp;
        }
    }

    fn fix(&mut self, indices: Vec<usize>, index_buffer: &mut IndexBuffer) {
        let mut origin = indices.clone();
        let mut buffer = Vec::with_capacity(64);

        while !origin.is_empty() {
            let mut j = 0;
            while j < origin.len() {
                let i = origin[j];
                j += 1;
                let mut triangle = self.triangles[i];
                for k in 0..3 {
                    let neighbor_index = triangle.neighbors[k];
                    if neighbor_index.is_not_nil() {
                        let mut neighbor = self.triangles[neighbor_index];
                        if self.swap(triangle, neighbor) {
                            index_buffer.add(triangle.index);
                            index_buffer.add(neighbor.index);

                            triangle = self.triangles[triangle.index];
                            neighbor = self.triangles[neighbor.index];

                            let tna = triangle.na();
                            if tna.is_not_nil() && tna != neighbor.index {
                                buffer.push(tna);
                            }

                            let tnb = triangle.nb();
                            if tnb.is_not_nil() && tnb != neighbor.index {
                                buffer.push(tnb);
                            }

                            let tnc = triangle.nc();
                            if tnc.is_not_nil() && tnc != neighbor.index {
                                buffer.push(tnc);
                            }

                            let nna = neighbor.na();
                            if nna.is_not_nil() && nna != triangle.index {
                                buffer.push(nna);
                            }

                            let nnb = neighbor.nb();
                            if nnb.is_not_nil() && nnb != triangle.index {
                                buffer.push(nnb);
                            }

                            let nnc = neighbor.nc();
                            if nnc.is_not_nil() && nnc != triangle.index {
                                buffer.push(nnc);
                            }
                        }
                    }
                }
            }
            origin.clear();

            let temp = origin;
            origin = buffer;
            buffer = temp;
        }
    }

    fn swap(&mut self, abc: DTriangle, pbc: DTriangle) -> bool {
        let pi = pbc.opposite(abc.index);
        unsafe {
            let p = *pbc.vertices.get_unchecked(pi);


            let ai: usize;
            let bi: usize;
            let ci: usize;
            let a: DVertex;  // opposite a-p
            let b: DVertex;  // edge bc
            let c: DVertex;

            ai = abc.opposite(pbc.index);
            match ai {
                0 => {
                    bi = 1;
                    ci = 2;
                    a = abc.va();
                    b = abc.vb();
                    c = abc.vc();
                }
                1 => {
                    bi = 2;
                    ci = 0;
                    a = abc.vb();
                    b = abc.vc();
                    c = abc.va();
                }
                _ => {
                    bi = 0;
                    ci = 1;
                    a = abc.vc();
                    b = abc.va();
                    c = abc.vb();
                }
            }

            let is_pass = Self::condition(p.point, c.point, a.point, b.point);

            return if is_pass {
                false
            } else {
                let is_abp_cw = Triangle::is_clockwise(a.point, b.point, p.point);

                let bp = pbc.neighbor(c.index);
                let cp = pbc.neighbor(b.index);
                let ab = *abc.neighbors.get_unchecked(ci);
                let ac = *abc.neighbors.get_unchecked(bi);

                // abc -> abp
                let abp: DTriangle;

                // pbc -> acp
                let acp: DTriangle;

                if is_abp_cw {
                    abp = DTriangle::abc_bc_ac_ab(
                        abc.index,
                        a,
                        b,
                        p,
                        bp,                 // a - bp
                        pbc.index,          // p - ap
                        ab,                     // b - ab
                    );

                    acp = DTriangle::abc_bc_ac_ab(
                        pbc.index,
                        a,
                        p,
                        c,
                        cp,                 // a - cp
                        ac,                     // p - ac
                        abc.index,          // b - ap
                    );
                } else {
                    abp = DTriangle::abc_bc_ac_ab(
                        abc.index,
                        a,
                        p,
                        b,
                        bp,                 // a - bp
                        ab,                 // p - ab
                        pbc.index,          // b - ap
                    );

                    acp = DTriangle::abc_bc_ac_ab(
                        pbc.index,
                        a,
                        c,
                        p,
                        cp,                 // a - cp
                        abc.index,          // p - ap
                        ac,                 // b - ac
                    )
                }

                // fix neighbor's link
                // ab, cp didn't change neighbor
                // bc -> ap, so no changes

                // ac (abc) is now edge of acp
                let ac_index = *abc.neighbors.get_unchecked(bi); // b - angle
                self.update_neighbor_index(ac_index, abc.index, acp.index);

                // bp (pbc) is now edge of abp
                let bp_index = pbc.neighbor(c.index); // c - angle
                self.update_neighbor_index(bp_index, pbc.index, abp.index);

                *self.triangles.get_unchecked_mut(abc.index) = abp;
                *self.triangles.get_unchecked_mut(pbc.index) = acp;

                true
            };
        }
    }

    fn update_neighbor_index(&mut self, index: usize, old_neighbor: usize, new_neighbor: usize) {
        if index.is_not_nil() {
            unsafe {
                let mut neighbor = *self.triangles.get_unchecked_mut(index);
                neighbor.update_opposite(old_neighbor, new_neighbor);
            }
        }
    }

    // if p0 is inside circumscribe circle of p1, p2, p3 return false
    // if p0 is inside circumscribe A + B > 180
    // return true if triangle satisfied condition and do not need flip triangles
    fn condition(p0: FixVec, p1: FixVec, p2: FixVec, p3: FixVec) -> bool {
        // x, y of all coordinates must be in range of i32
        // p1, p2, p3 points of current triangle
        // p0 is a test point
        // p1 and p3 common points of triangle p1, p2, p3 and p1, p0, p2
        // alpha (A) is an angle of p1, p0, p3
        // beta (B) is an angle of p1, p2, p3

        let v10 = p1 - p0;
        let v30 = p3 - p0;

        let v12 = p1 - p2;
        let v32 = p3 - p2;

        let cos_a = v10.unsafe_dot_product(v30);
        let cos_b = v12.unsafe_dot_product(v32);

        if cos_a < 0 && cos_b < 0 {
            // A > 90 and B > 90
            // A + B > 180
            return false;
        }

        if cos_a >= 0 && cos_b >= 0 {
            // A <= 90 and B <= 90
            // A + B <= 180
            return true;
        }

        let sn_a = v10.unsafe_cross_product(v30).abs() as u128; // A <= 180
        let sn_b = v12.unsafe_cross_product(v32).abs() as u128; // B <= 180

        if cos_a < 0 {
            // cosA < 0
            // cosB >= 0
            let cs_a = (-cos_a) as u128;
            let cs_b = cos_b as u128;

            let sin_a_cos_b = sn_a * cs_b;    // positive
            let cos_a_sin_b = cs_a * sn_b;    // negative

            sin_a_cos_b >= cos_a_sin_b
        } else {
            // cosA >= 0
            // cosB < 0
            let cs_a = cos_a as u128;
            let cs_b = (-cos_b) as u128;

            let sin_a_cos_b = sn_a * cs_b;    // negative
            let cos_a_sin_b = cs_a * sn_b;    // positive

            cos_a_sin_b >= sin_a_cos_b
        }
    }
}