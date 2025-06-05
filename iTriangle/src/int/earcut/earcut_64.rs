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

enum ShrinkResult {
    Impossible,
    Full,
    NotMore(FixVec),
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

    #[inline(always)]
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
        let candidates = self.filter_by_bounding_box(ear_indices);
        if candidates == 0 {
            return Some(end);
        }

        // we should shrink the ear from the end
        // ear is a convex polygon
        // we should find minimal vector which exclude all candidates
        match self.find_shrink_vector(start, ear_indices, candidates, same_point) {
            ShrinkResult::Impossible => None,
            ShrinkResult::Full => Some(end),
            ShrinkResult::NotMore(max_vec) => {
                self.find_max_possible_ear(start, ear_indices, max_vec)
            }
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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
    fn find_shrink_vector(
        &self,
        start: usize,
        ear_indices: u64,
        candidates: u64,
        same_point: bool,
    ) -> ShrinkResult {
        let a = *self.point(start);
        let end = ear_indices.prev_wrapped_index(start);
        let (b, count) = if same_point {
            let count = ear_indices.count_ones() as usize - 1;
            let bi = ear_indices.prev_wrapped_index(end);
            (*self.point(bi), count)
        } else {
            let count = ear_indices.count_ones() as usize;
            (*self.point(end), count)
        };

        let mut ba = b.subtract(a);

        let mut bits = candidates;
        let n = bits.count_ones();

        let mut any = false;

        // compare with candidates
        for _ in 0..n {
            let i = bits.trailing_zeros() as usize;
            bits &= !(1 << i);

            let c = *self.point(i);
            let ca = c.subtract(a);

            let cross = ca.cross_product(ba);

            match cross.cmp(&0) {
                Ordering::Less => {}
                Ordering::Equal => {
                    if !(any || same_point) {
                        // if C inside AB, ear is not possible
                        if Math::ab_contains_c(a.subtract(c), c, b) {
                            return ShrinkResult::Impossible;
                        }
                    }
                }
                Ordering::Greater => {
                    if self.is_point_inside_ear(c, start, ear_indices, count) {
                        ba = ca;
                        any = true;
                    }
                }
            }
        }

        if any {
            ShrinkResult::NotMore(ba)
        } else {
            ShrinkResult::Full
        }
    }

    #[inline(always)]
    fn is_point_inside_ear(
        &self,
        c: IntPoint,
        start: usize,
        ear_indices: u64,
        count: usize,
    ) -> bool {
        let mut a = *self.point(start);
        let mut j = ear_indices.next_wrapped_index(start);

        for _ in 0..count {
            let b = *self.point(j);
            let ab = b.subtract(a);
            let ac = c.subtract(a);
            let cross = ab.cross_product(ac);

            if cross <= 0 {
                // true only if C inside last slice edge
                return cross != 0 && j == start && Math::ab_contains_c(ac, c, b);
            }

            a = b;
            j = ear_indices.next_wrapped_index(j)
        }

        true
    }

    #[inline]
    fn find_max_possible_ear(
        &self,
        start: usize,
        ear_indices: u64,
        min_vec: FixVec,
    ) -> Option<usize> {
        let a = *self.point(start);

        let b0 = ear_indices.next_wrapped_index(start);
        let mut i = start;
        let mut j = start;
        let mut bi = b0;

        let mut cross = i64::MIN;
        while cross < 0 {
            let b = *self.point(bi);
            let ba = b.subtract(a);

            cross = min_vec.cross_product(ba);

            j = i; // last success
            i = bi; // keep prev
            bi = ear_indices.next_wrapped_index(bi);
        }

        if j == b0 {
            None
        } else {
            Some(j)
        }
    }
}

struct Math;

impl Math {
    #[inline(always)]
    fn ab_contains_c(ac: FixVec, c: IntPoint, b: IntPoint) -> bool {
        // a, b, c already on the same line
        let bc = b.subtract(c);
        let dot = ac.dot_product(bc);
        dot < 0
    }
}

pub(super) trait Bit {
    fn ones_start_to_index(count: usize) -> Self;
    fn ones_index_to_end(count: usize) -> Self;
    fn ones_in_sorted_closed_range(start: usize, end: usize) -> Self;
    fn ones_in_range_include(start: usize, end: usize) -> Self;
    fn next_wrapped_index(&self, after: usize) -> usize;
    fn prev_wrapped_index(&self, before: usize) -> usize;
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

    #[inline(always)]
    fn prev_wrapped_index(&self, before: usize) -> usize {
        debug_assert!(before <= 63);
        let back = *self & u64::ones_start_to_index(before);
        if back != 0 {
            63 - back.leading_zeros() as usize
        } else {
            63 - self.leading_zeros() as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::int::earcut::earcut_64::{Bit, Earcut64, EarcutSolver, Math};
    use crate::int::earcut::flat::FlatEarcutStore;
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
    fn test_ab_contains_0() {
        let a = IntPoint::new(0, 0);
        let b = IntPoint::new(10, 0);
        let c0 = IntPoint::new(-5, 0);
        let c1 = IntPoint::new(0, 0);
        let c2 = IntPoint::new(5, 0);
        let c3 = IntPoint::new(10, 0);
        let c4 = IntPoint::new(15, 0);

        assert_eq!(Math::ab_contains_c(a.subtract(c0), c0, b), false);
        assert_eq!(Math::ab_contains_c(a.subtract(c1), c1, b), false);
        assert_eq!(Math::ab_contains_c(a.subtract(c2), c2, b), true);
        assert_eq!(Math::ab_contains_c(a.subtract(c3), c3, b), false);
        assert_eq!(Math::ab_contains_c(a.subtract(c4), c4, b), false);
    }

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
    fn test_prev_wrapped_index_0() {
        assert_eq!(0b011_u64.prev_wrapped_index(0), 1);
        assert_eq!(0b011_u64.prev_wrapped_index(1), 0);
        assert_eq!(0b111_u64.prev_wrapped_index(2), 1);

        assert_eq!(0b0110_u64.prev_wrapped_index(1), 2);
        assert_eq!(0b0110_u64.prev_wrapped_index(2), 1);
        assert_eq!(0b1110_u64.prev_wrapped_index(3), 2);
    }

    #[test]
    fn test_next_contains_0() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let mut triangulation = IntTriangulation::<u32>::empty();
        let solver = EarcutSolver::new(&contour, FlatEarcutStore::new(&mut triangulation));

        let p0 = IntPoint::new(5, 5);
        let r0 = solver.is_point_inside_ear(p0, 0, 0b1111, 3);
        assert_eq!(r0, true);

        let p1 = IntPoint::new(15, 15);
        let r1 = solver.is_point_inside_ear(p1, 0, 0b1111, 3);
        assert_eq!(r1, false);

        let p2 = IntPoint::new(10, 0);
        let r2 = solver.is_point_inside_ear(p2, 0, 0b1111, 3);
        assert_eq!(r2, false);

        let p3 = IntPoint::new(5, 0);
        let r3 = solver.is_point_inside_ear(p3, 0, 0b1111, 3);
        assert_eq!(r3, false);

        let p4 = IntPoint::new(10, 5);
        let r4 = solver.is_point_inside_ear(p4, 0, 0b1111, 3);
        assert_eq!(r4, false);

        let p5 = IntPoint::new(5, 10);
        let r5 = solver.is_point_inside_ear(p5, 0, 0b1111, 3);
        assert_eq!(r5, false);

        // slice edge
        let p6 = IntPoint::new(0, 5);
        let r6 = solver.is_point_inside_ear(p6, 0, 0b1111, 3);
        assert_eq!(r6, true);
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
    fn test_earcut_7b() {
        let contour = vec![
            IntPoint::new(15, 10),
            IntPoint::new(0, 10),
            IntPoint::new(0, 0),
            IntPoint::new(5, 5),
            IntPoint::new(10, 5),
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
