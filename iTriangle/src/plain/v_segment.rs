use std::cmp::Ordering;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VSegment {
    pub(super) a: IntPoint,
    pub(super) b: IntPoint,
}

impl VSegment {
    #[inline]
    fn is_under_segment_order(&self, other: &VSegment) -> Ordering {
        match self.a.cmp(&other.a) {
            Ordering::Less => Triangle::clock_order_point(self.a, other.a, self.b),
            Ordering::Equal => Triangle::clock_order_point(self.a, other.b, self.b),
            Ordering::Greater => Triangle::clock_order_point(other.a, other.b, self.a),
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