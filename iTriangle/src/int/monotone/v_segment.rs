use core::cmp::Ordering;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VSegment {
    pub(crate) a: IntPoint,
    pub(crate) b: IntPoint,
}

impl VSegment {
    #[inline]
    fn is_under_segment_order(&self, other: &VSegment) -> Ordering {
        match self.b.cmp(&other.b) {
            Ordering::Less => Triangle::clock_order_point(self.b, other.a, other.b),
            Ordering::Equal => Triangle::clock_order_point(self.b, self.a, other.a),
            Ordering::Greater => Triangle::clock_order_point(other.b, self.b, self.a),
        }
    }

    #[inline]
    pub(crate) fn is_under_point_order(&self, p: IntPoint) -> Ordering {
        debug_assert!(self.a.x <= p.x && p.x <= self.b.x);
        Triangle::clock_order_point(self.a, p, self.b)
    }
}

impl PartialOrd<Self> for VSegment {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VSegment {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.is_under_segment_order(other)
    }
}

impl Default for VSegment {
    #[inline]
    fn default() -> Self {
        Self {
            a: IntPoint::ZERO,
            b: IntPoint::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use i_overlay::i_float::int::point::IntPoint;
    use crate::int::monotone::v_segment::VSegment;

    #[test]
    fn test_0() {
        let v0 = VSegment { a: IntPoint::new(0, 0), b: IntPoint::new(5, 0) };
        let v1 = VSegment { a: IntPoint::new(0, 0), b: IntPoint::new(5, 5) };

        assert_eq!(v0.is_under_segment_order(&v1), Ordering::Less);
    }

    #[test]
    fn test_1() {
        let v0 = VSegment { a: IntPoint::new(-2, -2), b: IntPoint::new(5, -2) };
        let v1 = VSegment { a: IntPoint::new(0, 0), b: IntPoint::new(5, 0) };

        assert_eq!(v0.is_under_segment_order(&v1), Ordering::Less);
    }

    #[test]
    fn test_2() {
        let v0 = VSegment { a: IntPoint::new(-2, -5), b: IntPoint::new(5, 0) };
        let v1 = VSegment { a: IntPoint::new(0, 0), b: IntPoint::new(5, 0) };

        assert_eq!(v0.is_under_segment_order(&v1), Ordering::Less);
    }

    #[test]
    fn test_3() {
        let v0 = VSegment { a: IntPoint::new(0, -5), b: IntPoint::new(5, 5) };
        let v1 = VSegment { a: IntPoint::new(0, 0), b: IntPoint::new(5, 0) };

        assert_eq!(v0.is_under_segment_order(&v1), Ordering::Greater);
    }
}