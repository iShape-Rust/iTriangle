use i_float::fix_vec::FixVec;
use i_shape::triangle::Triangle;
use crate::delaunay::index_buffer::IndexBuffer;
use crate::delaunay::triangle::DTriangle;
use crate::delaunay::vertex::DVertex;

pub struct Delaunay {
    points: Vec<FixVec>,
    triangles: Vec<DTriangle>,
}

impl Delaunay {
    pub(super) fn new(points: Vec<FixVec>, triangles: Vec<DTriangle>) -> Self {
        Self { points, triangles }
    }

    pub fn points(self) -> Vec<FixVec> {
        self.points
    }

    pub fn triangles_indices(&self) -> Vec<usize> {
        let mut result = vec![usize::MAX; 3 * self.triangles.len()];
        let mut j = 0;
        for triangle in self.triangles.iter() {
            result[j] = triangle.vertices[0].index;
            result[j + 1] = triangle.vertices[1].index;
            result[j + 2] = triangle.vertices[2].index;
            j += 3;
        }

        return result;
    }

    pub fn triangles_indices_shifted(&self, shifted: usize) -> Vec<usize> {
        let mut result = vec![usize::MAX; 3 * self.triangles.len()];
        let mut j = 0;
        for triangle in self.triangles.iter() {
            result[j] = triangle.vertices[0].index + shifted;
            result[j + 1] = triangle.vertices[1].index + shifted;
            result[j + 2] = triangle.vertices[2].index + shifted;
            j += 3;
        }

        return result;
    }

    pub(crate) fn build(&mut self) {
        let count = self.triangles.len();
        let mut visit_marks = vec![false; count];

        let mut visit_index = 0;

        let mut origin = Vec::with_capacity(64);
        origin.push(0);

        let mut buffer = Vec::with_capacity(64);


        while !origin.is_empty() {
            let mut j = 0;
            while j < origin.len() {
                let i = origin[j];
                j += 1;
                let mut triangle = self.triangles[i];
                visit_marks[i] = true;
                for k in 0..3 {
                    let neighbor_index = triangle.neighbors[k];
                    if neighbor_index == usize::MAX {
                        continue;
                    }
                    let mut neighbor = self.triangles[neighbor_index];
                    if self.swap(triangle, neighbor) {
                        triangle = self.triangles[triangle.index];
                        neighbor = self.triangles[neighbor.index];

                        let tna = triangle.na();
                        if tna != usize::MAX && tna != neighbor.index {
                            buffer.push(tna);
                        }

                        let tnb = triangle.nb();
                        if tnb != usize::MAX && tnb != neighbor.index {
                            buffer.push(tnb);
                        }

                        let tnc = triangle.nc();
                        if tnc != usize::MAX && tnc != neighbor.index {
                            buffer.push(tnc);
                        }

                        let nna = neighbor.na();
                        if nna != usize::MAX && nna != triangle.index {
                            buffer.push(nna);
                        }

                        let nnb = neighbor.nb();
                        if nnb != usize::MAX && nnb != triangle.index {
                            buffer.push(nnb);
                        }

                        let nnc = neighbor.nc();
                        if nnc != usize::MAX && nnc != triangle.index {
                            buffer.push(nnc);
                        }
                    }
                }
            }

            if buffer.len() == 0 && visit_index < count {
                visit_index += 1;
                while visit_index < count {
                    if visit_marks[visit_index] == false {
                        buffer.push(visit_index);
                        break;
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
                    if neighbor_index != usize::MAX {
                        let mut neighbor = self.triangles[neighbor_index];
                        if self.swap(triangle, neighbor) {
                            index_buffer.add(triangle.index);
                            index_buffer.add(neighbor.index);

                            triangle = self.triangles[triangle.index];
                            neighbor = self.triangles[neighbor.index];

                            let tna = triangle.na();
                            if tna != usize::MAX && tna != neighbor.index {
                                buffer.push(tna);
                            }

                            let tnb = triangle.nb();
                            if tnb != usize::MAX && tnb != neighbor.index {
                                buffer.push(tnb);
                            }

                            let tnc = triangle.nc();
                            if tnc != usize::MAX && tnc != neighbor.index {
                                buffer.push(tnc);
                            }

                            let nna = neighbor.na();
                            if nna != usize::MAX && nna != triangle.index {
                                buffer.push(nna);
                            }

                            let nnb = neighbor.nb();
                            if nnb != usize::MAX && nnb != triangle.index {
                                buffer.push(nnb);
                            }

                            let nnc = neighbor.nc();
                            if nnc != usize::MAX && nnc != triangle.index {
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
        let p = pbc.vertices[pi];

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
            let ab = abc.neighbors[ci];
            let ac = abc.neighbors[bi];

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
            let ac_index = abc.neighbors[bi]; // b - angle
            if ac_index != usize::MAX {
                let mut neighbor = self.triangles[ac_index];
                neighbor.update_opposite(abc.index, acp.index);
                self.triangles[ac_index] = neighbor;
            }

            // bp (pbc) is now edge of abp
            let bp_index = pbc.neighbor(c.index); // c - angle
            if bp_index != usize::MAX {
                let mut neighbor = self.triangles[bp_index];
                neighbor.update_opposite(pbc.index, abp.index);
                self.triangles[bp_index] = neighbor;
            }

            self.triangles[abc.index] = abp;
            self.triangles[pbc.index] = acp;

            true
        };
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