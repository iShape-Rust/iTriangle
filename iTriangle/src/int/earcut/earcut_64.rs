use crate::int::meta::TrianglesCount;
use crate::int::triangulation::{IndexType, IntTriangulation};
use core::cmp::Ordering;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::IntContour;
use i_overlay::i_shape::util::reserve::Reserve;

pub(crate) trait Earcut64 {
    fn earcut_triangulate_into<I: IndexType>(
        &self,
        triangulation: &mut IntTriangulation<I>,
    ) -> bool;
}

impl Earcut64 for IntContour {
    fn earcut_triangulate_into<I: IndexType>(
        &self,
        triangulation: &mut IntTriangulation<I>,
    ) -> bool {
        debug_assert!(self.len() < 64);
        triangulation
            .indices
            .reserve_capacity(self.triangles_count(0));
        triangulation.indices.clear();

        let success = EarcutSolver::new(self).triangulate_into(triangulation);

        if success {
            triangulation.points.clear();
            triangulation.points.extend_from_slice(self);
        }

        success
    }
}

enum ConvexSearchResult {
    Circle,
    Index(usize),
    None,
}

struct EarcutSolver<'a> {
    contour: &'a IntContour,
    available: u64,
}

impl<'a> EarcutSolver<'a> {
    fn new(contour: &'a IntContour) -> Self {
        Self {
            contour,
            available: u64::ones_start_to_index(contour.len()),
        }
    }

    fn triangulate_into<I: IndexType>(&mut self, triangulation: &mut IntTriangulation<I>) -> bool {
        let mut i = 0;
        while self.available.count_ones() >= 3 {
            match self.find_convex_part(i) {
                ConvexSearchResult::None => {}
                ConvexSearchResult::Circle => {
                    self.collect_last_ear_triangles(i, triangulation);
                    return true;
                }
                ConvexSearchResult::Index(convex_end) => {
                    if let Some(ear_end) = self.validate_ear_and_trim_if_needed(i, convex_end) {
                        self.collect_ear_triangles(i, ear_end, triangulation);
                    }
                }
            }
            i = self.available.next_wrapped_index(i);
        }

        true
    }

    #[inline]
    fn collect_ear_triangles<I: IndexType>(
        &mut self,
        start: usize,
        end: usize,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let bits = self.available & u64::ones_in_range_include(start, end);

        let mut i = start;
        let a = unsafe { I::try_from(i).unwrap_unchecked() };
        i = bits.next_wrapped_index(i);
        let mut b = unsafe { I::try_from(i).unwrap_unchecked() };

        while i != end {
            // clear only inner vertices
            self.available &= !(1 << i);

            i = bits.next_wrapped_index(i);
            let c = unsafe { I::try_from(i).unwrap_unchecked() };
            triangulation.indices.push(a);
            triangulation.indices.push(b);
            triangulation.indices.push(c);

            b = c;
        }
    }

    #[inline]
    fn collect_last_ear_triangles<I: IndexType>(
        &self,
        start: usize,
        triangulation: &mut IntTriangulation<I>,
    ) {
        let bits = self.available;
        let mut i = start;
        let a = unsafe { I::try_from(i).unwrap_unchecked() };
        i = bits.next_wrapped_index(i);
        let mut b = unsafe { I::try_from(i).unwrap_unchecked() };
        i = bits.next_wrapped_index(i);
        while i != start {
            let c = unsafe { I::try_from(i).unwrap_unchecked() };
            triangulation.indices.push(a);
            triangulation.indices.push(b);
            triangulation.indices.push(c);

            b = c;
            i = bits.next_wrapped_index(i);
        }
    }

    fn find_convex_part(&self, i0: usize) -> ConvexSearchResult {
        let a = self.contour[i0];
        let i1 = self.available.next_wrapped_index(i0);
        let mut b = self.contour[i1];

        let mut i = i1;
        let mut v0 = b.subtract(a); // prev edge
        while i != i0 {
            let j = self.available.next_wrapped_index(i);
            let c = self.contour[j];

            // cb - next edge
            let cb = c.subtract(b);

            // ac - cut ear edge
            let ac = c.subtract(a);

            // ac must be inside ear
            let cross_0 = cb.cross_product(ac);

            // must not go in clock wise direction
            let cross_1 = cb.cross_product(v0);

            if cross_1 > 0 || cross_0 >= 0 && j != i0 {
                // j != i0 skip last full ear edge
                if i == i1 {
                    return ConvexSearchResult::None;
                }

                if cross_0 == 0 {
                    let prev = self.available.prev_wrapped_index(i);
                    return if prev == i1 {
                        ConvexSearchResult::None
                    } else {
                        ConvexSearchResult::Index(prev)
                    };
                }

                return ConvexSearchResult::Index(i);
            }
            b = c;
            i = j;
            v0 = cb;
        }
        // if we here then we did a full round
        ConvexSearchResult::Circle
    }

    #[inline]
    fn validate_ear_and_trim_if_needed(&self, start: usize, end: usize) -> Option<usize> {
        let mut candidates = self.fast_ear_check(start, end);
        if candidates == 0 {
            //
            return Some(end);
        }
        let ear_indices = self.available & u64::ones_in_range_include(start, end);
        let second_index = ear_indices.next_wrapped_index(start);
        let mut range_end = end;
        while range_end != second_index {
            candidates = self.candidates_ear_check(start, range_end, candidates);
            if candidates == 0 {
                return Some(range_end);
            }
            range_end = ear_indices.prev_wrapped_index(range_end);
        }
        None
    }

    #[inline]
    fn fast_ear_check(&self, start: usize, end: usize) -> u64 {
        let ear_indices = u64::ones_in_range_include(start, end);
        let other_indices = self.available & !ear_indices;

        // fast check find all vertices which are from ear side
        let a = self.contour[start];
        let b = self.contour[end];
        let ab = a.subtract(b);
        let mut candidates = 0;
        let mut bits = other_indices;
        while bits != 0 {
            let index = bits.trailing_zeros() as usize;
            let bit_val = 1 << index;

            let c = self.contour[index];
            let cb = c.subtract(b);

            let cross = ab.cross_product(cb);
            if cross >= 0 {
                candidates |= bit_val;
            }

            bits &= !bit_val;
        }
        candidates
    }

    #[inline]
    fn candidates_ear_check(&self, start: usize, end: usize, candidates: u64) -> u64 {
        let ear_indices = self.available & u64::ones_in_range_include(start, end);

        let a = self.contour[start];
        let b = self.contour[end];
        let ab = a.subtract(b);

        let mut bits = candidates;
        let mut i = bits.trailing_zeros() as usize;

        // by all candidates
        while bits != 0 {
            let c = self.contour[i];
            let ac = a.subtract(c);
            if ac.cross_product(ab) == 0 {
                // if same line as last cut edge
                let bc = b.subtract(c);

                // must be opposite for inner point
                if bc.dot_product(ac) < 0 {
                    return bits;
                }
            } else if self.is_not_valid_candidate(ear_indices, c) {
                return bits;
            }

            bits &= !(1 << i);
            i = bits.trailing_zeros() as usize;
        }

        0
    }

    #[inline]
    fn is_not_valid_candidate(&self, ear_indices: u64, c: IntPoint) -> bool {
        let n = ear_indices.count_ones();

        let mut i = ear_indices.trailing_zeros() as usize;
        let mut p0 = self.contour[i];

        let mut count = 0;
        for _ in 0..n {
            i = ear_indices.next_wrapped_index(i);
            let pi = self.contour[i];
            let (a, b) = match p0.x.cmp(&pi.x) {
                Ordering::Equal => {
                    p0 = pi;
                    continue;
                }
                Ordering::Less => (p0, pi),
                Ordering::Greater => (pi, p0),
            };

            if a.x <= c.x && c.x < b.x {
                let ab = a.subtract(b);
                let ac = a.subtract(c);

                let cross = ab.cross_product(ac);
                if cross >= 0 {
                    count += 1;
                }
            }
            p0 = pi;
        }

        count & 1 == 1
    }
}

trait Bit {
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
    use crate::int::earcut::earcut_64::{Bit, Earcut64};
    use crate::int::triangulation::IntTriangulation;
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
    fn test_prev_wrapped_index_0() {
        assert_eq!(0b011_u64.prev_wrapped_index(0), 1);
        assert_eq!(0b011_u64.prev_wrapped_index(1), 0);
        assert_eq!(0b111_u64.prev_wrapped_index(2), 1);

        assert_eq!(0b0110_u64.prev_wrapped_index(1), 2);
        assert_eq!(0b0110_u64.prev_wrapped_index(2), 1);
        assert_eq!(0b1110_u64.prev_wrapped_index(3), 2);
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
    fn test_earcut_3() {
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
    fn test_earcut_4() {
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
    fn test_earcut_5() {
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
    fn test_earcut_6() {
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
    fn test_earcut_7() {
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
    fn test_earcut_8() {
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
    fn test_earcut_9() {
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
    fn test_earcut_10() {
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
    fn test_earcut_11() {
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
    fn test_earcut_12() {
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
    fn test_earcut_13() {
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
    fn test_earcut_14() {
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
    fn test_earcut_15() {
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
    fn test_random_0() {
        for _ in 0..1000_000 {
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

    fn single_test(contour: &IntContour) {
        let mut triangulation = IntTriangulation::<u8>::default();
        contour.earcut_triangulate_into(&mut triangulation);

        triangulation.validate(contour.area_two());
        assert_eq!(triangulation.indices.len() / 3, contour.len() - 2);
    }

    fn roll_test(contour: &IntContour) {
        let mut triangulation = IntTriangulation::<u8>::default();

        let mut path = contour.to_vec();
        for _ in 0..path.len() {
            contour.earcut_triangulate_into(&mut triangulation);

            triangulation.validate(contour.area_two());
            assert_eq!(triangulation.indices.len() / 3, contour.len() - 2);

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
