use core::cmp::Ordering;
use core::ptr;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

const CLOCK_ORDER_HEAP_LEN: usize = 3;

pub(super) type ClockOrderHeap = FixedHeap<IntPoint, ClockOrderCompare, CLOCK_ORDER_HEAP_LEN>;

impl ClockOrderHeap {
    #[inline(always)]
    pub(super) fn with_center(center: IntPoint) -> Self {
        Self::new(ClockOrderCompare { center })
    }
}

pub(super) trait Compare<T> {
    fn compare(&self, a: &T, b: &T) -> Ordering;
}

pub(super) struct ClockOrderCompare {
    center: IntPoint,
}

impl Compare<IntPoint> for ClockOrderCompare {
    #[inline(always)]
    fn compare(&self, a: &IntPoint, b: &IntPoint) -> Ordering {
        Triangle::clock_order_point(self.center, *a, *b)
    }
}

pub(super) struct FixedHeap<T, C: Compare<T>, const N: usize> {
    overflow: bool,
    comparator: C,
    count: usize,
    buffer: [T; N],
}

impl<T: Default + Copy, C: Compare<T>, const N: usize> FixedHeap<T, C, N> {
    #[inline(always)]
    pub(super) fn new(comparator: C) -> Self {
        Self {
            count: 0,
            overflow: false,
            buffer: [T::default(); N],
            comparator,
        }
    }

    #[inline(always)]
    pub(super) fn as_slice(&self) -> &[T] {
        &self.buffer[0..self.count]
    }

    #[inline(always)]
    pub(super) fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    pub(super) fn is_overflow(&self) -> bool {
        self.overflow
    }

    #[inline(always)]
    pub(super) fn add(&mut self, v: T) {
        if self.count < N {
            unsafe { *self.buffer.get_unchecked_mut(self.count) = v };
            let mut i = self.count;
            self.count += 1;
            while i > 0 {
                let parent = (i - 1) >> 1;
                if self.comparator.compare(&self.val(parent), &self.val(i)) <= Ordering::Equal {
                    break;
                }
                self.swap(i, parent);
                i = parent;
            }
            return;
        }
        self.overflow = true;
        let root = unsafe { self.buffer.get_unchecked_mut(0) };
        if self.comparator.compare(&v, root) <= Ordering::Equal {
            return;
        }
        *root = v;
        self.down(0);
    }

    #[inline(always)]
    fn down(&mut self, mut i: usize) {
        loop {
            let ilt = i * 2 + 1;
            let irt = ilt + 1;

            if ilt >= self.count {
                break;
            }

            let (j, min) = if irt < self.count {
                let lt = self.val(ilt);
                let rt = self.val(irt);
                if self.comparator.compare(&lt, &rt) <= Ordering::Equal {
                    (ilt, lt)
                } else {
                    (irt, rt)
                }
            } else {
                (ilt, self.val(ilt))
            };

            let order = self.comparator.compare(&self.val(i), &min);
            match order {
                Ordering::Greater => {
                    self.swap(i, j);
                    i = j;
                }
                Ordering::Less | Ordering::Equal => break
            }
        }
    }

    #[inline(always)]
    fn val(&self, index: usize) -> T {
        *unsafe { self.buffer.get_unchecked(index) }
    }

    #[inline(always)]
    fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let p = self.buffer.as_mut_ptr();
            ptr::swap(p.add(a), p.add(b));
        }
    }

    #[inline(always)]
    pub(super) fn sort_in_place(&mut self) {
        let original_count = self.count;
        for end in (1..self.count).rev() {
            self.swap(0, end);
            self.count -= 1;
            self.down(0);
        }
        self.count = original_count;
    }
}

#[cfg(test)]
mod tests {
    use crate::int::earcut::heap::{ClockOrderHeap, Compare, FixedHeap};
    use core::cmp::Ordering;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_float::triangle::Triangle;

    const CAP: usize = 15;

    struct Min;

    impl<T: Ord> Compare<T> for Min {
        #[inline(always)]
        fn compare(&self, a: &T, b: &T) -> Ordering {
            a.cmp(b)
        }
    }

    struct Max;

    impl<T: Ord> Compare<T> for Max {
        #[inline(always)]
        fn compare(&self, a: &T, b: &T) -> Ordering {
            b.cmp(a)
        }
    }

    #[test]
    fn test_min_heap_0() {
        let mut heap = FixedHeap::<i32, Min, CAP>::new(Min);
        for &v in &[30, 10, 50, 20] {
            heap.add(v);
        }
        heap.sort_in_place();

        assert_eq!(heap.buffer[0..4], [50, 30, 20, 10]);
    }

    #[test]
    fn test_min_heap_1() {
        let mut heap = FixedHeap::<i32, Min, 7>::new(Min);
        for &v in &[5, 5, 4, 4, 3, 2, 1, 0, 0, 0] {
            heap.add(v);
        }
        heap.sort_in_place();

        assert_eq!(heap.buffer[0..7], [5, 5, 4, 4, 3, 2, 1]);
    }

    #[test]
    fn test_max_heap() {
        let mut heap = FixedHeap::<i32, Max, CAP>::new(Max);
        for &v in &[30, 10, 50, 20] {
            heap.add(v);
        }
        heap.sort_in_place();

        assert_eq!(heap.buffer[0..4], [10, 20, 30, 50]);
    }

    #[test]
    fn test_clock_0() {
        let mut points = [
            IntPoint::new(-3, 0),
            IntPoint::new(-2, 0),
            IntPoint::new(-1, 0),
            IntPoint::new(0, 0),
            IntPoint::new(1, 0),
            IntPoint::new(2, 0),
            IntPoint::new(3, 0),
        ];
        let c = IntPoint::new(0, 10);
        let mut heap = ClockOrderHeap::with_center(c);
        for &p in points.iter() {
            heap.add(p);
        }

        assert_eq!(heap.count, 7);

        heap.sort_in_place();

        points.sort_unstable_by(|a, b| Triangle::clock_order_point(c, *b, *a));

        assert_eq!(heap.buffer[0..7], points[0..7]);
    }

    #[test]
    fn test_1() {
        let mut points = [
            IntPoint::new(-3, 0),
            IntPoint::new(-2, 0),
            IntPoint::new(-1, 0),
            IntPoint::new(0, 0),
            IntPoint::new(1, 0),
            IntPoint::new(2, 0),
            IntPoint::new(3, 0),
        ];
        let c = IntPoint::new(0, -10);
        let mut heap = ClockOrderHeap::with_center(c);
        for &p in points.iter() {
            heap.add(p);
        }

        heap.sort_in_place();

        points.sort_unstable_by(|a, b| Triangle::clock_order_point(c, *b, *a));

        assert_eq!(heap.buffer[0..7], points[0..7]);
    }

    #[test]
    fn test_2() {
        let mut points = [
            IntPoint::new(0, 2),
            IntPoint::new(1, -1),
        ];
        let c = IntPoint::new(3, 3);
        let mut heap = ClockOrderHeap::with_center(c);
        heap.add(points[0]);
        heap.add(points[1]);

        heap.sort_in_place();
        points.sort_unstable_by(|a, b| Triangle::clock_order_point(c, *b, *a));

        assert_eq!(heap.buffer[0..2], points);

    }

    #[test]
    fn test_3() {
        let mut points = [
            IntPoint::new(2, -1),
            IntPoint::new(1, -3),
            IntPoint::new(1, -1),
            IntPoint::new(2, -1),
            IntPoint::new(1, 0),
            IntPoint::new(1, 1),
            IntPoint::new(0, 1),
            IntPoint::new(0, 2),
            IntPoint::new(-1, 1),
        ];
        let c = IntPoint::new(-4, 2);
        let mut heap = ClockOrderHeap::with_center(c);
        for &p in points.iter() {
            heap.add(p);
        }

        heap.sort_in_place();

        points.sort_unstable_by(|a, b| Triangle::clock_order_point(c, *b, *a));

        assert_eq!(heap.buffer[0..7], points[0..7]);
    }
}
