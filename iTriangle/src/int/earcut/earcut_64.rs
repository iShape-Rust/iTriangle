use crate::int::earcut::flat::FlatEarcutStore;
use crate::int::earcut::net::NetEarcutStore;
use crate::int::meta::TrianglesCount;
use crate::int::triangulation::{IndexType, IntTriangulation, RawIntTriangulation};
use core::cmp::Ordering;
use i_overlay::i_float::fix_vec::FixVec;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::util::reserve::Reserve;

pub(super) trait EarcutStore {
    fn collect_triangles(&mut self, contour: &[IntPoint], start: usize, bits: u64, count: u32);
}

pub trait Earcut64 {
    fn earcut_flat_triangulate_into<I: IndexType>(&self, triangulation: &mut IntTriangulation<I>);

    fn earcut_net_triangulate_into(&self, triangulation: &mut RawIntTriangulation);
}

impl Earcut64 for [IntPoint] {
    fn earcut_flat_triangulate_into<I: IndexType>(&self, triangulation: &mut IntTriangulation<I>) {
        debug_assert!(self.len() <= 64);

        triangulation
            .indices
            .reserve_capacity(self.triangles_count(0));
        triangulation.indices.clear();

        EarcutSolver::new(self, FlatEarcutStore::new(triangulation)).triangulate();

        triangulation.points.clear();
        triangulation.points.extend_from_slice(self);
    }

    fn earcut_net_triangulate_into(&self, triangulation: &mut RawIntTriangulation) {
        debug_assert!(self.len() <= 64);

        triangulation
            .triangles
            .reserve_capacity(self.triangles_count(0));
        triangulation.triangles.clear();

        EarcutSolver::new(self, NetEarcutStore::new(self.len(), triangulation)).triangulate();

        triangulation.points.clear();
        triangulation.points.extend_from_slice(self);
    }
}

enum ConvexSearchResult {
    Circle,
    Index(usize, bool),
    None,
}

struct EarcutSolver<'a, S> {
    store: S,
    contour: &'a [IntPoint],
    available: u64,
}

impl<'a, S: EarcutStore> EarcutSolver<'a, S> {
    fn new(contour: &'a [IntPoint], store: S) -> Self {
        Self {
            store,
            contour,
            available: u64::ones_start_to_index(contour.len()),
        }
    }

    #[inline(always)]
    fn point(&self, index: usize) -> &IntPoint {
        unsafe { self.contour.get_unchecked(index) }
    }

    fn triangulate(&mut self) {
        let mut i = 0;
        while self.available.count_ones() >= 3 {
            match self.find_convex_part(i) {
                ConvexSearchResult::None => {}
                ConvexSearchResult::Circle => {
                    self.collect_last_ear_triangles(i);
                    return;
                }
                ConvexSearchResult::Index(convex_end, same_point) => {
                    if let Some(ear_end) = self.validate_and_shrink_ear(i, convex_end, same_point) {
                        let same_point_after_shrink = same_point && ear_end == convex_end;
                        self.collect_ear_triangles(i, ear_end, same_point_after_shrink);
                    }
                }
            }
            i = self.available.next_wrapped_index(i);
        }
    }

    #[inline]
    fn collect_ear_triangles(&mut self, start: usize, end: usize, same_point: bool) {
        // ear indices
        let bits = self.available & u64::ones_in_range_include(start, end);

        // indices to remove
        let mut invert = !bits;
        // we keep end points
        invert |= 1 << start;
        invert |= 1 << end;

        let mut n = bits.count_ones() - 2;

        // Handles degenerate case where start and end share the same position,
        // like self-touches contours (e.g. sand clock).
        if same_point {
            // remove same point
            invert &= !(1 << start);
            // no need last zero triangle
            n -= 1;
        }

        self.available &= invert;
        self.store.collect_triangles(self.contour, start, bits, n);
    }

    #[inline]
    fn collect_last_ear_triangles(&mut self, start: usize) {
        let bits = self.available;
        self.store
            .collect_triangles(self.contour, start, bits, bits.count_ones() - 2);
    }

    fn find_convex_part(&self, i0: usize) -> ConvexSearchResult {
        // the ear must be a convex polygon

        let a = *self.point(i0);
        let i1 = self.available.next_wrapped_index(i0);
        let b = *self.point(i1);

        let mut i = i1;
        let ab = b.subtract(a);
        let mut ce = ab; // the prev vector
        let mut cj = *self.point(i);

        while i != i0 {
            let j = self.available.next_wrapped_index(i);
            let ci = cj;
            cj = *self.point(j);

            // appended edge
            let cc = cj.subtract(ci);

            // ca - slice edge
            let ca = a.subtract(cj);

            // cab < 180
            let cross_a = ab.cross_product(ca);

            // cca < 180
            let cross_c = cc.cross_product(ca);

            // cce <= 180
            let cross_v = cc.cross_product(ce);

            if cross_a >= 0 || cross_c <= 0 || cross_v > 0 {
                if i == i1 {
                    // empty ear
                    return ConvexSearchResult::None;
                } else if j == i0 {
                    return ConvexSearchResult::Circle;
                }

                if cross_a == 0 && a == cj {
                    return ConvexSearchResult::Index(j, true);
                }

                return ConvexSearchResult::Index(i, false);
            }
            i = j;
            ce = cc;
        }
        // if we here then we did a full round
        ConvexSearchResult::Circle
    }

    #[inline]
    fn validate_and_shrink_ear(&self, start: usize, end: usize, same_point: bool) -> Option<usize> {
        let ear_indices = self.available & u64::ones_in_range_include(start, end);
        let mut candidates = self.filter_by_bounding_box(ear_indices);
        if candidates == 0 {
            return Some(end);
        }

        let ear_points_count = ear_indices.count_ones() as usize;

        let (on_slice, contour_len) = if same_point {
            (0, ear_points_count - 1)
        } else {
            let on_slice = self.find_on_edge(start, end, candidates);
            (on_slice, ear_points_count)
        };

        // if not exclude slice candidates the xor logic can remove them
        let mut inner_candidates = candidates & !on_slice;
        if inner_candidates > 0 {
            inner_candidates = self.filter_by_even_odd(start, ear_indices, inner_candidates, contour_len);
        }

        candidates = inner_candidates | on_slice;

        if candidates == 0 {
            return Some(end);
        }

        // from this moment we should shrink the ear from the end
        // ear is a convex polygon, all candidates strong inside it
        // first we should find minimal vector which exclude all candidates
        // this vector the star of this vector in the point[start]
        let (v, indices) = self.find_shrink_vector(start, candidates);

        self.find_max_possible_ear(start, ear_indices, v, indices)
    }

    #[inline]
    fn filter_by_bounding_box(&self, ear_indices: u64) -> u64 {
        let rect = self.bounding_box(ear_indices);

        let mut candidates = 0;
        let mut bits = self.available & !ear_indices;
        while bits != 0 {
            let index = bits.trailing_zeros() as usize;
            let bit_val = 1 << index;
            bits &= !bit_val;

            if rect.contains(*self.point(index)) {
                candidates |= bit_val;
            }
        }

        candidates
    }

    fn bounding_box(&self, indices: u64) -> IntRect {
        let mut bits = indices;
        let i0 = bits.trailing_zeros() as usize;
        bits &= !(1 << i0);

        let mut rect = IntRect::with_point(*self.point(i0));

        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= !(1 << i);
            rect.unsafe_add_point(self.point(i));
        }

        rect
    }

    #[inline]
    fn filter_by_even_odd(&self, start: usize, ear_indices: u64, candidates: u64, contour_len: usize) -> u64 {
        let n = candidates.count_ones();

        let j0 = candidates.trailing_zeros() as usize;
        let mut j;

        let mut p0 = *self.point(start);
        let mut i = ear_indices.next_wrapped_index(start);

        // if even then a point is outside ear
        let mut count = 0u64; // count even odd

        // we will skip points that only touch the contour
        let mut skip = 0u64; // detect touch

        for _ in 0..contour_len {
            let pi = *self.point(i);
            let (a, b) = match p0.x.cmp(&pi.x) {
                Ordering::Equal => {
                    p0 = pi;
                    i = ear_indices.next_wrapped_index(i);
                    continue;
                }
                Ordering::Less => (p0, pi),
                Ordering::Greater => (pi, p0),
            };

            let ab = a.subtract(b);

            j = j0;
            for _ in 0..n {
                let c = *self.point(j);
                if a.x <= c.x && c.x < b.x {
                    let ac = a.subtract(c);

                    let cross = ab.cross_product(ac);
                    match cross.cmp(&0) {
                        // the edge must be under the point
                        Ordering::Greater => count ^= 1 << j,
                        // touch contour
                        Ordering::Equal => skip |= 1 << j,
                        Ordering::Less => {}
                    }
                }

                j = candidates.next_wrapped_index(j);
            }

            p0 = pi;
            i = ear_indices.next_wrapped_index(i);
        }

        // we remove that one which touch contour
        // we keep that one which count number is odd
        let mut filtered = 0;
        j = j0;
        for _ in 0..n {
            let bit_val = 1 << j;
            let xor = count & bit_val;
            let off = skip & bit_val;
            filtered |= xor & !off;

            j = candidates.next_wrapped_index(j);
        }

        filtered
    }

    #[inline]
    fn find_on_edge(&self, start: usize, end: usize, candidates: u64) -> u64 {
        let a = *self.point(start);
        let b = *self.point(end);

        let ab = a.subtract(b);

        let mut result = 0;

        let mut bits = candidates;
        while bits > 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= !(1 << i);

            let c = *self.point(i);

            let ac = a.subtract(c);

            if ab.cross_product(ac) == 0 {
                // we can skip C if it not inside AB
                // C is inside AB if AC and BC opposite
                let bc = b.subtract(c);

                // must be same opposite to add
                if bc.dot_product(ac) < 0 {
                    // remove
                    result |= 1 << i;
                }
            }
        }

        result
    }


    #[inline]
    fn find_shrink_vector(&self, start: usize, candidates: u64) -> (FixVec, u64) {
        let a = *self.point(start);

        let mut bits = candidates;
        let i = bits.trailing_zeros() as usize;
        let b = *self.point(i);
        bits &= !(1 << i);
        let mut indices = 1 << i;

        let mut ba = b.subtract(a);

        // compare with rest candidates
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            let c = *self.point(i);
            bits &= !(1 << i);

            let ca = c.subtract(a);

            match ca.cross_product(ba).cmp(&0) {
                Ordering::Equal => indices |= 1 << i,
                Ordering::Greater => {
                    indices = 1 << i;
                    ba = ca;
                }
                _ => {}
            }
        }

        (ba, indices)
    }

    #[inline]
    fn find_max_possible_ear(&self, start: usize, ear_indices: u64, limit: FixVec, vec_indices: u64) -> Option<usize> {
        let a = *self.point(start);

        let b0 = ear_indices.next_wrapped_index(start);
        let mut i = start;
        let mut j = start;
        let mut bi = b0;

        let mut cross = i64::MIN;
        let mut dot = i64::MAX;
        while cross < 0 || cross == 0 && dot < 0 {
            let b = *self.point(bi);
            let ba = b.subtract(a);

            cross = limit.cross_product(ba);
            dot = limit.dot_product(ba);

            j = i;  // last success
            i = bi; // keep prev
            bi = ear_indices.next_wrapped_index(bi);
        }

        // we need check are all indices inside AB
        if cross == 0 && !self.is_any_on_edge(a, *self.point(i), vec_indices) {
            // all candidates outside AB(slice edge)
            j = i;
        }

        if j == b0 {
            None
        } else {
            Some(j)
        }
    }

    #[inline]
    fn is_any_on_edge(&self, a: IntPoint, b: IntPoint, vec_indices: u64) -> bool {
        // we need check are all indices inside AB
        let mut bits = vec_indices;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            let c = *self.point(i);
            bits &= !(1 << i);

            let ac = a.subtract(c);

            // we can skip C if it not inside AB
            // C is inside AB if AC and BC opposite
            let bc = b.subtract(c);

            // must be collinear to skip
            let dot = bc.dot_product(ac);
            if dot < 0 {
                // C is inside AB
                return true
            }
        }

        false
    }

}

pub(super) trait Bit {
    fn ones_start_to_index(count: usize) -> Self;
    fn ones_index_to_end(count: usize) -> Self;
    fn ones_in_sorted_closed_range(start: usize, end: usize) -> Self;
    fn ones_in_range_include(start: usize, end: usize) -> Self;
    fn next_wrapped_index(&self, after: usize) -> usize;
}

impl Bit for u64 {
    #[inline(always)]
    fn ones_start_to_index(index: usize) -> Self {
        // index is excluded
        debug_assert!(index <= 64);
        let zeros_count = (64 - index) & 63;
        u64::MAX >> zeros_count
    }

    #[inline(always)]
    fn ones_index_to_end(index: usize) -> Self {
        // index is included
        debug_assert!(index <= 64);
        let zeros_count = index & 63;
        u64::MAX << zeros_count
    }

    #[inline(always)]
    fn ones_in_sorted_closed_range(min: usize, max: usize) -> Self {
        debug_assert!(min < max);
        debug_assert!(max <= 64);
        let up = 63usize.saturating_sub(max);
        (u64::MAX << min) & (u64::MAX >> up)
    }

    #[inline(always)]
    fn ones_in_range_include(start: usize, end: usize) -> Self {
        if start < end {
            Self::ones_in_sorted_closed_range(start, end)
        } else {
            let mut inv = !Self::ones_in_sorted_closed_range(end, start);
            inv |= 1 << start;
            inv |= 1 << end;
            inv
        }
    }

    #[inline(always)]
    fn next_wrapped_index(&self, after: usize) -> usize {
        debug_assert!(after <= 63);
        let front = self & u64::ones_index_to_end(after + 1);
        if front != 0 {
            front.trailing_zeros() as usize
        } else {
            self.trailing_zeros() as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::int::earcut::earcut_64::{Bit, Earcut64};
    use crate::int::triangulation::{IntTriangulation, RawIntTriangulation};
    use alloc::vec;
    use alloc::vec::Vec;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::core::simplify::Simplify;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;
    use i_overlay::i_shape::int::shape::IntContour;
    use rand::Rng;

    #[test]
    fn test_ones_start_to_index_0() {
        assert_eq!(u64::ones_start_to_index(1), 1);
        assert_eq!(u64::ones_start_to_index(64), u64::MAX);
    }

    #[test]
    fn test_ones_index_to_end_0() {
        assert_eq!(u64::ones_index_to_end(0), u64::MAX);
        assert_eq!(u64::ones_index_to_end(1), u64::MAX - 1);
    }

    #[test]
    fn test_ones_in_sorted_closed_range_0() {
        assert_eq!(u64::ones_in_sorted_closed_range(0, 63), u64::MAX);
        assert_eq!(u64::ones_in_sorted_closed_range(1, 63), u64::MAX << 1);
        assert_eq!(u64::ones_in_sorted_closed_range(2, 63), u64::MAX << 2);

        assert_eq!(u64::ones_in_sorted_closed_range(0, 62), u64::MAX >> 1);
        assert_eq!(u64::ones_in_sorted_closed_range(0, 61), u64::MAX >> 2);
        assert_eq!(u64::ones_in_sorted_closed_range(0, 60), u64::MAX >> 3);

        // overflow
        assert_eq!(u64::ones_in_sorted_closed_range(0, 64), u64::MAX);
        assert_eq!(u64::ones_in_sorted_closed_range(1, 64), u64::MAX << 1);
        assert_eq!(u64::ones_in_sorted_closed_range(2, 64), u64::MAX << 2);
    }

    #[test]
    fn test_ones_in_range_include_0() {
        // sorted order
        assert_eq!(u64::ones_in_range_include(0, 63), u64::MAX);
        assert_eq!(u64::ones_in_range_include(1, 63), u64::MAX << 1);
        assert_eq!(u64::ones_in_range_include(2, 63), u64::MAX << 2);

        assert_eq!(u64::ones_in_range_include(0, 62), u64::MAX >> 1);
        assert_eq!(u64::ones_in_range_include(0, 61), u64::MAX >> 2);
        assert_eq!(u64::ones_in_range_include(0, 60), u64::MAX >> 3);

        // overflow
        assert_eq!(u64::ones_in_range_include(0, 64), u64::MAX);
        assert_eq!(u64::ones_in_range_include(1, 64), u64::MAX << 1);
        assert_eq!(u64::ones_in_range_include(2, 64), u64::MAX << 2);

        // reversed order
        assert_eq!(
            u64::ones_in_range_include(63, 0),
            !u64::ones_in_range_include(1, 62)
        );
        assert_eq!(
            u64::ones_in_range_include(63, 1),
            !u64::ones_in_range_include(2, 62)
        );
        assert_eq!(
            u64::ones_in_range_include(63, 2),
            !u64::ones_in_range_include(3, 62)
        );

        assert_eq!(
            u64::ones_in_range_include(62, 0),
            !u64::ones_in_range_include(1, 61)
        );
        assert_eq!(
            u64::ones_in_range_include(61, 0),
            !u64::ones_in_range_include(1, 60)
        );
        assert_eq!(
            u64::ones_in_range_include(60, 0),
            !u64::ones_in_range_include(1, 59)
        );
    }

    #[test]
    fn test_next_wrapped_index_0() {
        assert_eq!(0b11_u64.next_wrapped_index(0), 1);
        assert_eq!(0b11_u64.next_wrapped_index(1), 0);

        assert_eq!(0b110_u64.next_wrapped_index(1), 2);
        assert_eq!(0b110_u64.next_wrapped_index(2), 1);
    }

    #[test]
    fn test_earcut_0() {
        let square = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        single_test(&square);
        roll_test(&square);
    }

    #[test]
    fn test_earcut_1() {
        let square = vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 5),
            IntPoint::new(10, 10),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
            IntPoint::new(0, 5),
        ];

        single_test(&square);
        roll_test(&square);
    }

    #[test]
    fn test_earcut_2() {
        let rhombus = vec![
            IntPoint::new(-5, 0),
            IntPoint::new(0, -5),
            IntPoint::new(5, 0),
            IntPoint::new(0, 5),
        ];

        single_test(&rhombus);
        roll_test(&rhombus);
    }

    #[test]
    fn test_earcut_3() {
        let sand_clock = vec![
            IntPoint::new(0, 0),
            IntPoint::new(-5, -10),
            IntPoint::new(5, -10),
            IntPoint::new(0, 0),
            IntPoint::new(5, 10),
            IntPoint::new(-5, 10),
        ];

        single_test(&sand_clock);
        roll_test(&sand_clock);
    }

    #[test]
    fn test_earcut_4() {
        let hz_sand_clock = vec![
            IntPoint::new(0, 0),
            IntPoint::new(-10, 5),
            IntPoint::new(-10, -5),
            IntPoint::new(0, 0),
            IntPoint::new(10, -5),
            IntPoint::new(10, 5),
        ];

        single_test(&hz_sand_clock);
        roll_test(&hz_sand_clock);
    }

    #[test]
    fn test_earcut_5() {
        let cross = vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 10),
            IntPoint::new(-5, 10),
            IntPoint::new(0, 0),
            IntPoint::new(-10, 5),
            IntPoint::new(-10, -5),
            IntPoint::new(0, 0),
            IntPoint::new(-5, -10),
            IntPoint::new(5, -10),
            IntPoint::new(0, 0),
            IntPoint::new(10, -5),
            IntPoint::new(10, 5),
        ];

        single_test(&cross);
        roll_test(&cross);
    }

    #[test]
    fn test_earcut_6() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 5),
            IntPoint::new(5, 5),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_7() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(5, 5),
            IntPoint::new(10, 5),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
            IntPoint::new(15, 10),
            IntPoint::new(0, 10),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_8() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(5, 5),
            IntPoint::new(10, 5),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
            IntPoint::new(15, 15),
            IntPoint::new(10, 15),
            IntPoint::new(10, 10),
            IntPoint::new(5, 10),
            IntPoint::new(5, 15),
            IntPoint::new(0, 15),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_9() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 20),
            IntPoint::new(0, 20),
            IntPoint::new(-5, 15),
            IntPoint::new(-5, 10),
            IntPoint::new(0, 5),
            IntPoint::new(5, 5),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_10() {
        let contour = vec![
            IntPoint::new(3, -3),
            IntPoint::new(2, 4),
            IntPoint::new(-1, 1),
            IntPoint::new(-2, 2),
            IntPoint::new(-4, -2),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_11() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(2, -1),
            IntPoint::new(2, 0),
            IntPoint::new(2, 1),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_12() {
        let contour = vec![
            IntPoint::new(-3, -2),
            IntPoint::new(2, 2),
            IntPoint::new(2, 3),
            IntPoint::new(2, 4),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_13() {
        let contour = vec![
            IntPoint::new(-1, 0),
            IntPoint::new(-3, -3),
            IntPoint::new(-1, -4),
            IntPoint::new(-1, -2),
            IntPoint::new(2, -3),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_14() {
        let contour = vec![
            IntPoint::new(0, 2),
            IntPoint::new(-1, 4),
            IntPoint::new(0, -3),
            IntPoint::new(3, -4),
            IntPoint::new(0, -2),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_15() {
        let contour = vec![
            IntPoint::new(3, 3),
            IntPoint::new(-1, 3),
            IntPoint::new(-2, 4),
            IntPoint::new(0, -2),
            IntPoint::new(-1, 2),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_16() {
        let contour = vec![
            IntPoint::new(-2, 2),
            IntPoint::new(-1, -2),
            IntPoint::new(-2, -4),
            IntPoint::new(3, 4),
            IntPoint::new(-3, 3),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_17() {
        let contour = vec![
            IntPoint::new(-3, 3),
            IntPoint::new(-3, -1),
            IntPoint::new(3, -1),
            IntPoint::new(-2, 2),
            IntPoint::new(2, 0),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_18() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(-4, 3),
            IntPoint::new(-2, -4),
            IntPoint::new(2, 0),
            IntPoint::new(0, -1),
            IntPoint::new(-3, 2),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_19() {
        let contour = vec![
            IntPoint::new(-3, 1),
            IntPoint::new(-3, 0),
            IntPoint::new(-2, -1),
            IntPoint::new(1, -1),
            IntPoint::new(2, -2),
            IntPoint::new(0, 1),
            IntPoint::new(-1, 3),
            IntPoint::new(-1, 1),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_20() {
        let contour = vec![
            IntPoint::new(-3, -1),
            IntPoint::new(-4, -2),
            IntPoint::new(3, -1),
            IntPoint::new(-4, 4),
            IntPoint::new(-1, -1),
            IntPoint::new(1, 0),
            IntPoint::new(1, -1),
            IntPoint::new(-1, -1),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_21() {
        let contour = vec![
            IntPoint::new(3, 2),
            IntPoint::new(-4, 0),
            IntPoint::new(1, -4),
            IntPoint::new(0, 1),
            IntPoint::new(-1, 0),
            IntPoint::new(-3, 0),
            IntPoint::new(0, 1),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_22() {
        let contour = vec![
            IntPoint::new(-2, 4),
            IntPoint::new(-2, -4),
            IntPoint::new(2, -3),
            IntPoint::new(2, 0),
            IntPoint::new(0, -1),
            IntPoint::new(1, -2),
            IntPoint::new(-1, -2),
            IntPoint::new(0, -1),
            IntPoint::new(-1, -1),
            IntPoint::new(1, 2),
            IntPoint::new(2, 0),
            IntPoint::new(3, -2),
            IntPoint::new(2, 2),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_23() {
        let contour = vec![
            IntPoint::new(4, 4),
            IntPoint::new(-4, 4),
            IntPoint::new(2, 2),
            IntPoint::new(4, 1),
            IntPoint::new(3, 3),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_24() {
        let contour = vec![
            IntPoint::new(1, 2),
            IntPoint::new(0, 3),
            IntPoint::new(0, 0),
            IntPoint::new(1, 1),
            IntPoint::new(3, 1),
            IntPoint::new(1, 3),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_25() {
        let contour = vec![
            IntPoint::new(1, 4),
            IntPoint::new(0, 5),
            IntPoint::new(0, 0),
            IntPoint::new(1, 1),
            IntPoint::new(3, 1),
            IntPoint::new(1, 3),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_26() {
        let contour = vec![
            IntPoint::new(-4, 0),
            IntPoint::new(2, -2),
            IntPoint::new(0, -1),
            IntPoint::new(1, 0),
            IntPoint::new(2, -2),
            IntPoint::new(3, -4),
            IntPoint::new(3, 0),
            IntPoint::new(4, 0),
            IntPoint::new(-2, 4),
            IntPoint::new(0, 0),
            IntPoint::new(-1, 0),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_earcut_27() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(1, 0),
            IntPoint::new(2, 2),
            IntPoint::new(1, 1),
            IntPoint::new(0, 1),
            IntPoint::new(-1, -1),
        ];

        single_test(&contour);
        roll_test(&contour);
    }

    #[test]
    fn test_random_0() {
        for _ in 0..100_000 {
            if let Some(first) = random(8, 5)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    if !contour.is_empty() {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_1() {
        for _ in 0..100_000 {
            if let Some(first) = random(8, 7)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    if !contour.is_empty() {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_2() {
        for _ in 0..100_000 {
            if let Some(first) = random(8, 10)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    if !contour.is_empty() {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_3() {
        for _ in 0..50_000 {
            if let Some(first) = random(8, 12)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    if !contour.is_empty() {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_4() {
        for _ in 0..40_000 {
            if let Some(first) = random(16, 32)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    let n = contour.len();
                    if 3 <= n && n <= 64 {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_5() {
        for _ in 0..20_000 {
            if let Some(first) = random(16, 48)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    let n = contour.len();
                    if 3 <= n && n <= 64 {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_6() {
        for _ in 0..10_000 {
            if let Some(first) = random(16, 64)
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_output_points())
                .first()
            {
                if let Some(contour) = first.first() {
                    let n = contour.len();
                    if 3 <= n && n <= 64 {
                        single_test(&contour);
                    }
                }
            }
        }
    }

    fn single_test(contour: &IntContour) {
        // flat
        let mut flat = IntTriangulation::<u8>::default();
        contour.earcut_flat_triangulate_into(&mut flat);

        flat.validate(contour.area_two());
        assert!(flat.indices.len() / 3 <= contour.len() - 2);

        // net
        let mut net = RawIntTriangulation::default();
        contour.earcut_net_triangulate_into(&mut net);

        net.validate();
        assert_eq!(net.area_two(), contour.area_two());
        assert!(net.triangles.len() / 3 <= contour.len() - 2);
    }

    fn roll_test(contour: &IntContour) {
        let mut triangulation = IntTriangulation::<u8>::default();

        let mut path = contour.to_vec();
        for _ in 0..path.len() {
            contour.earcut_flat_triangulate_into(&mut triangulation);

            triangulation.validate(contour.area_two());
            assert!(triangulation.indices.len() / 3 <= contour.len() - 2);

            roll(&mut path);
        }
    }

    fn roll(points: &mut Vec<IntPoint>) {
        if let Some(last) = points.pop() {
            points.insert(0, last);
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
