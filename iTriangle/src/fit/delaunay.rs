use crate::geom::triangle::ABCTriangle;
use crate::raw::triangulation::RawTriangulation;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::u128::UInt128;

pub struct Delaunay {
    pub triangles: Vec<ABCTriangle>,
    pub points: Vec<IntPoint>,
}

impl RawTriangulation {
    #[inline]
    fn into_delaunay(self) -> Delaunay {
        let mut delaunay = Delaunay {
            triangles: self.triangles,
            points: self.points,
        };

        delaunay.build(usize::MAX);

        delaunay
    }
}

impl Delaunay {
    fn build(&mut self, max_round_count: usize) {
        let mut dirty = vec![false; self.triangles.len()];
        let mut unprocessed: Vec<usize> = (0..self.triangles.len()).collect();
        let mut buffer = Vec::with_capacity(self.triangles.len());
        let mut neighbors = [usize::MAX; 3];
        for _ in 0..max_round_count {
            for &abc_index in unprocessed.iter() {
                if dirty[abc_index] {
                    continue;
                }
                neighbors = unsafe { self.triangles.get_unchecked(abc_index) }.neighbors;
                for &pbc_index in neighbors.iter() {
                    if pbc_index >= self.triangles.len() {
                        continue;
                    }

                    if self.swap_triangles(abc_index, pbc_index) {
                        if !dirty[abc_index] {
                            dirty[abc_index] = true;
                            buffer.push(abc_index);
                        }
                        if !dirty[pbc_index] {
                            dirty[pbc_index] = true;
                            buffer.push(pbc_index);
                        }
                    }
                }
            }

            if buffer.is_empty() {
                return;
            }

            for &i in buffer.iter() {
                dirty[i] = false;
            }

            std::mem::swap(&mut unprocessed, &mut buffer);
            buffer.clear();
        }
    }

    #[inline]
    fn swap_triangles(&mut self, abc_index: usize, pcb_index: usize) -> bool {
        let abc = self.triangles[abc_index].abc_by_neighbor(pcb_index);
        let pcb = self.triangles[pcb_index].abc_by_neighbor(abc_index);
        let is_pass = Self::condition(
            pcb.v0.vertex.point, // p
            abc.v0.vertex.point,
            abc.v1.vertex.point,
            abc.v2.vertex.point,
        );

        if is_pass {
            return false;
        }

        // abc and pcb are clock-wised triangles

        // abc -> abp
        // pcb -> pca
        
        self.update_neighbor(abc.v1.neighbor, abc_index, pcb_index);
        self.update_neighbor(pcb.v1.neighbor, pcb_index, abc_index);

        let abp = &mut self.triangles[abc_index];
        abp.neighbors[abc.v0.position] = pcb.v1.neighbor;
        abp.neighbors[abc.v1.position] = pcb_index;
        abp.neighbors[abc.v2.position] = abc.v2.neighbor;
        abp.vertices[abc.v2.position] = pcb.v0.vertex;

        let pca = &mut self.triangles[pcb_index];
        pca.neighbors[pcb.v0.position] = abc.v1.neighbor;
        pca.neighbors[pcb.v1.position] = abc_index;
        pca.neighbors[pcb.v2.position] = pcb.v2.neighbor;
        pca.vertices[pcb.v2.position] = abc.v0.vertex;
        true
    }

    #[inline]
    fn update_neighbor(&mut self, index: usize, old_index: usize, new_index: usize) {
        if index >= self.triangles.len() {
            return;
        }
        self.triangles[index].update_neighbor(old_index, new_index);
    }

    // if p is inside circumscribe circle of a, b, c return false
    // if p is inside circumscribe A + B > 180
    // return true if triangle satisfied condition and do not need flip triangles
    fn condition(p: IntPoint, a: IntPoint, b: IntPoint, c: IntPoint) -> bool {
        // x, y of all coordinates must be in range of i32
        // p is a test point
        // b and c common points of triangle abc and pcb
        // alpha (A) is an angle of bpc
        // beta (B) is an angle of cab

        let vbp = b.subtract(p);
        let vcp = c.subtract(p);

        let vba = b.subtract(a);
        let vca = c.subtract(a);

        let cos_a = vbp.dot_product(vcp);
        let cos_b = vba.dot_product(vca);

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

        let sn_a = vbp.cross_product(vcp).unsigned_abs(); // A <= 180
        let sn_b = vba.cross_product(vca).unsigned_abs(); // B <= 180

        if cos_a < 0 {
            // cosA < 0
            // cosB >= 0
            let sin_a_cos_b = UInt128::multiply(sn_a, cos_b as u64); // positive
            let cos_a_sin_b = UInt128::multiply(cos_a.unsigned_abs(), sn_b); // negative

            sin_a_cos_b >= cos_a_sin_b
        } else {
            // cosA >= 0
            // cosB < 0
            let sin_a_cos_b = UInt128::multiply(sn_a, cos_b.unsigned_abs()); // negative
            let cos_a_sin_b = UInt128::multiply(cos_a as u64, sn_b); // positive

            cos_a_sin_b >= sin_a_cos_b
        }
    }
}

impl ABCTriangle {
    #[inline]
    fn update_neighbor(&mut self, old_index: usize, new_index: usize) {
        if self.neighbors[0] == old_index {
            self.neighbors[0] = new_index;
        } else if self.neighbors[1] == old_index {
            self.neighbors[1] = new_index;
        } else {
            debug_assert_eq!(self.neighbors[2], old_index);
            self.neighbors[2] = new_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fit::delaunay::Delaunay;
    use crate::geom::point::IndexPoint;
    use crate::geom::triangle::ABCTriangle;
    use i_overlay::i_float::int::point::IntPoint;

    #[test]
    fn test_0() {
        let points = vec![
            IntPoint::new(-3, 3),
            IntPoint::new(-2, -3),
            IntPoint::new(-2, 0),
            IntPoint::new(0, -1),
            IntPoint::new(0, 3),
            IntPoint::new(2, -3),
            IntPoint::new(2, 0),
            IntPoint::new(3, 3),
        ];

        let mut delaunay = Delaunay {
            triangles: vec![
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(4, points[4]),
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(6, points[6]),
                    ],
                    neighbors: [1, 3, 2],
                },
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(3, points[3]),
                        IndexPoint::new(6, points[6]),
                    ],
                    neighbors: [5, 0, 4],
                },
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(0, points[0]),
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(4, points[4]),
                    ],
                    neighbors: [0, usize::MAX, usize::MAX],
                },
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(4, points[4]),
                        IndexPoint::new(6, points[6]),
                        IndexPoint::new(7, points[7]),
                    ],
                    neighbors: [usize::MAX, usize::MAX, 0],
                },
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(1, points[1]),
                        IndexPoint::new(3, points[3]),
                    ],
                    neighbors: [usize::MAX, 1, usize::MAX],
                },
                ABCTriangle {
                    vertices: [
                        IndexPoint::new(3, points[3]),
                        IndexPoint::new(5, points[5]),
                        IndexPoint::new(6, points[6]),
                    ],
                    neighbors: [usize::MAX, 1, usize::MAX],
                },
            ],
            points,
        };

        let is_swapped = delaunay.swap_triangles(0, 1);
        assert!(is_swapped);
    }
}
