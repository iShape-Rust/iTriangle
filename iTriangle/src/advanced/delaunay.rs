use crate::advanced::buffer::DelaunayBuffer;
use alloc::vec::Vec;
use crate::geom::triangle::IntTriangle;
use crate::int::triangulation::RawIntTriangulation;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::u128::UInt128;
use crate::advanced::bitset::IndexBitSet;

/// A 2D integer-based Delaunay triangulation.
/// Each triangle satisfies the Delaunay condition.
///
/// # Fields
/// - `triangles`: A list of `IntTriangle` elements (triangle vertex indices and neighbors)
/// - `points`: A list of `IntPoint` elements (original and inserted points)
///
pub struct IntDelaunay {
    pub triangles: Vec<IntTriangle>,
    pub points: Vec<IntPoint>,
}

impl RawIntTriangulation {

    /// Converts an int triangle mesh into a Delaunay triangulation by applying edge flips.
    ///
    /// The mesh is refined in-place by checking local angle conditions and
    /// flipping edges until the Delaunay criterion is satisfied.
    ///
    /// # Returns
    /// A new [`IntDelaunay`] structure with updated triangle connectivity.
    #[inline]
    pub fn into_delaunay(self) -> IntDelaunay {
        let mut delaunay = IntDelaunay {
            triangles: self.triangles,
            points: self.points,
        };

        delaunay.triangles.build();

        delaunay
    }
}

pub trait DelaunayRefine {
    fn build(&mut self);
    fn build_with_buffer(&mut self, buffer: &mut DelaunayBuffer);
    fn fix_triangles(&mut self, buffer: &mut Vec<usize>, bitset: &mut IndexBitSet);
    fn fix_triangle(&mut self, abc_index: usize, bitset: &mut IndexBitSet);
    fn update_neighbor(&mut self, neighbor_index: usize, old_index: usize, new_index: usize);
    fn swap_triangles(&mut self, abc_index: usize, pcb_index: usize) -> bool;
}

impl DelaunayRefine for [IntTriangle] {

    #[inline]
    fn build(&mut self) {
        let mut buffer = DelaunayBuffer::new();
        self.build_with_buffer(&mut buffer);
    }

    #[inline]
    fn build_with_buffer(&mut self, buffer: &mut DelaunayBuffer) {
        let mut bitset = buffer.bitset.take().unwrap_or_default();
        bitset.clear_and_resize(self.len());
        for abc_index in 0..self.len() {
            self.fix_triangle(abc_index, &mut bitset);
        }

        let mut indices = buffer.indices.take().unwrap_or_default();
        bitset.read_and_clean(&mut indices);

        if !indices.is_empty() {
            self.fix_triangles(&mut indices, &mut bitset);
        }

        buffer.bitset = Some(bitset);
        buffer.indices = Some(indices);
    }

    #[inline]
    fn fix_triangles(&mut self, indices: &mut Vec<usize>, bitset: &mut IndexBitSet) {
        debug_assert!(!indices.is_empty());
        debug_assert!(bitset.is_empty());
        while !indices.is_empty() {
            for &abc_index in indices.iter() {
                self.fix_triangle(abc_index, bitset);
            }
            bitset.read_and_clean(indices);
        }
    }

    #[inline]
    fn fix_triangle(&mut self, abc_index: usize, unchecked: &mut IndexBitSet) {
        // loop by same triangle increase cache locality
        let mut skip = usize::MAX;
        let mut perfect= false;
        while !perfect {
            perfect = true;
            let neighbors = unsafe { self.get_unchecked(abc_index) }.neighbors;
            for &pbc_index in neighbors.iter() {
                if pbc_index >= self.len() || pbc_index == skip {
                    continue;
                }

                if self.swap_triangles(abc_index, pbc_index) {
                    skip = pbc_index;
                    unchecked.insert(pbc_index);
                    perfect = false;
                    break;
                }
            }
        }
        unchecked.remove(abc_index);
    }

    #[inline]
    fn update_neighbor(&mut self, neighbor_index: usize, old_index: usize, new_index: usize) {
        if neighbor_index >= self.len() {
            return;
        }
        self[neighbor_index].update_neighbor(old_index, new_index);
    }

    #[inline]
    fn swap_triangles(&mut self, abc_index: usize, pcb_index: usize) -> bool {
        // abc_index & pcb_index can not be more self.triangles.len()
        let t_abc = unsafe { self.get_unchecked(abc_index) };
        let t_pcb = unsafe { self.get_unchecked(pcb_index) };
        let abc = t_abc.abc_by_neighbor(pcb_index);
        let pcb = t_pcb.abc_by_neighbor(abc_index);
        if DelaunayCondition::is_flip_not_required(
            pcb.v0.vertex.point, // p
            abc.v0.vertex.point, // a
            abc.v1.vertex.point, // b
            abc.v2.vertex.point, // c
        ) {
            return false;
        }

        // abc and pcb are clock-wised ordered triangles

        // abc -> abp
        // pcb -> pca

        self.update_neighbor(abc.v1.neighbor, abc_index, pcb_index);
        self.update_neighbor(pcb.v1.neighbor, pcb_index, abc_index);

        let abp = &mut self[abc_index];
        abp.neighbors[abc.v0.position] = pcb.v1.neighbor;
        abp.neighbors[abc.v1.position] = pcb_index;
        abp.neighbors[abc.v2.position] = abc.v2.neighbor;
        abp.vertices[abc.v2.position] = pcb.v0.vertex;

        let pca = &mut self[pcb_index];
        pca.neighbors[pcb.v0.position] = abc.v1.neighbor;
        pca.neighbors[pcb.v1.position] = abc_index;
        pca.neighbors[pcb.v2.position] = pcb.v2.neighbor;
        pca.vertices[pcb.v2.position] = abc.v0.vertex;

        true
    }
}

struct DelaunayCondition;

impl DelaunayCondition {
    // if p is inside circumscribe circle of a, b, c return false
    // if p is inside circumscribe A + B > 180
    // return true if triangle satisfied condition and do not need flip triangles
    // more detail explanation and demo https://ishape-rust.github.io/iShape-js/triangle/delaunay.html
    #[inline]
    fn is_flip_not_required(p: IntPoint, a: IntPoint, b: IntPoint, c: IntPoint) -> bool {
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

impl IntTriangle {
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
impl IntDelaunay {
    fn validate(&self) {
        use i_overlay::i_float::triangle::Triangle;

        for (i, t) in self.triangles.iter().enumerate() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;
            let area = Triangle::area_two_point(a, b, c);
            assert!(area <= 0);

            let n0 = t.neighbors[0];
            let n1 = t.neighbors[1];
            let n2 = t.neighbors[2];

            if n0 < self.triangles.len() {
                assert!(self.triangles[n0].neighbors.contains(&i));
            }
            if n1 < self.triangles.len() {
                assert!(self.triangles[n1].neighbors.contains(&i));
            }
            if n2 < self.triangles.len() {
                assert!(self.triangles[n2].neighbors.contains(&i));
            }
        }
    }

    fn area(&self) -> i64 {
        use i_overlay::i_float::triangle::Triangle;
        let mut s = 0;
        for t in self.triangles.iter() {
            let a = t.vertices[0].point;
            let b = t.vertices[1].point;
            let c = t.vertices[2].point;

            s += Triangle::area_two_point(a, b, c);
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use crate::advanced::delaunay::Vec;
use alloc::vec;
use crate::advanced::delaunay::DelaunayCondition;
use crate::advanced::delaunay::DelaunayRefine;
    use crate::advanced::delaunay::IntDelaunay;
    use crate::geom::point::IndexPoint;
    use crate::geom::triangle::IntTriangle;
    use crate::int::triangulatable::IntTriangulatable;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::core::simplify::Simplify;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;
    use rand::Rng;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    #[test]
    fn test_0() {
        let a = IntPoint::new(0, 4);
        let b = IntPoint::new(-2, 0);
        let c = IntPoint::new(2, 0);
        let p = IntPoint::new(0, -4);

        let is_flip_not_required = DelaunayCondition::is_flip_not_required(p, a, b, c);
        assert_eq!(is_flip_not_required, true);
    }

    #[test]
    fn test_1() {
        // border case
        let a = IntPoint::new(0, 2);
        let b = IntPoint::new(-2, 0);
        let c = IntPoint::new(2, 0);
        let p = IntPoint::new(0, -2);

        let is_flip_not_required = DelaunayCondition::is_flip_not_required(p, a, b, c);
        assert_eq!(is_flip_not_required, true);
    }

    #[test]
    fn test_2() {
        let a = IntPoint::new(0, 2);
        let b = IntPoint::new(-2, 0);
        let c = IntPoint::new(2, 0);
        let p = IntPoint::new(0, -1);

        let is_flip_not_required = DelaunayCondition::is_flip_not_required(p, a, b, c);
        assert_eq!(is_flip_not_required, false);
    }

    #[test]
    fn test_3() {
        let a = IntPoint::new(0, 1);
        let b = IntPoint::new(-2, 0);
        let c = IntPoint::new(2, 0);
        let p = IntPoint::new(0, -1);

        let is_flip_not_required = DelaunayCondition::is_flip_not_required(p, a, b, c);
        assert_eq!(is_flip_not_required, false);
    }

    #[test]
    fn test_4() {
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

        let mut delaunay = IntDelaunay {
            triangles: vec![
                IntTriangle {
                    vertices: [
                        IndexPoint::new(4, points[4]),
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(6, points[6]),
                    ],
                    neighbors: [1, 3, 2],
                },
                IntTriangle {
                    vertices: [
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(3, points[3]),
                        IndexPoint::new(6, points[6]),
                    ],
                    neighbors: [5, 0, 4],
                },
                IntTriangle {
                    vertices: [
                        IndexPoint::new(0, points[0]),
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(4, points[4]),
                    ],
                    neighbors: [0, usize::MAX, usize::MAX],
                },
                IntTriangle {
                    vertices: [
                        IndexPoint::new(4, points[4]),
                        IndexPoint::new(6, points[6]),
                        IndexPoint::new(7, points[7]),
                    ],
                    neighbors: [usize::MAX, usize::MAX, 0],
                },
                IntTriangle {
                    vertices: [
                        IndexPoint::new(2, points[2]),
                        IndexPoint::new(1, points[1]),
                        IndexPoint::new(3, points[3]),
                    ],
                    neighbors: [usize::MAX, 1, usize::MAX],
                },
                IntTriangle {
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

        let is_swapped = delaunay.triangles.swap_triangles(0, 1);
        assert!(is_swapped);
    }

    #[test]
    fn test_5() {
        let shape = vec![path(&[[4, 2], [-4, 4], [-1, 0], [0, -1], [4, -4]])];
        let shape_area = shape.area_two();

        let delaunay = shape.triangulate().into_delaunay();
        delaunay.validate();

        assert_eq!(delaunay.area(), shape_area);
    }

    #[test]
    fn test_random_0() {
        for _ in 0..100_000 {
            let shape = vec![random(8, 5)];

            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    IntOverlayOptions::keep_all_points(),
                )
                .first()
            {
                let shape_area = first.area_two();

                let delaunay = first.triangulate().into_delaunay();

                delaunay.validate();
                assert_eq!(delaunay.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_1() {
        for _ in 0..100_000 {
            let shape = vec![random(8, 12)];

            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    IntOverlayOptions::keep_all_points(),
                )
                .first()
            {
                let shape_area = first.area_two();

                let delaunay = first.triangulate().into_delaunay();

                delaunay.validate();
                assert_eq!(delaunay.area(), shape_area);
            };
        }
    }

    #[test]
    fn test_random_2() {
        for _ in 0..2_000 {
            let main = random(50, 20);
            let mut shape = vec![main];
            for _ in 0..10 {
                shape.push(random(30, 5));
            }

            if let Some(first) = shape
                .simplify(
                    FillRule::NonZero,
                    IntOverlayOptions::keep_all_points(),
                )
                .first()
            {
                let shape_area = first.area_two();

                let delaunay = first.triangulate().into_delaunay();

                delaunay.validate();
                assert_eq!(delaunay.area(), shape_area);
            };
        }
    }

    fn random(radius: i32, n: usize) -> IntPath {
        let a = radius / 2;
        let mut points = Vec::with_capacity(n);
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a);
            let y = rng.random_range(-a..=a);
            points.push(IntPoint { x, y })
        }

        points
    }
}
