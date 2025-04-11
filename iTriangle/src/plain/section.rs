use crate::plain::vertex::IndexPoint;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use std::cmp::Ordering;
use i_tree::set::sort::KeyValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EdgeType {
    Regular(usize), // keep index to triangle
    Phantom(usize), // keep index to itself(edge) in phantom store
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VSegment {
    pub(super) a: IntPoint,
    pub(super) b: IntPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TriangleEdge {
    pub(super) a: IndexPoint,
    pub(super) b: IndexPoint,
    pub(super) kind: EdgeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Content {
    Point(IndexPoint),
    Edges(Vec<TriangleEdge>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Section {
    // Todo probably we can remove prev and next
    pub(super) prev: IntPoint,
    pub(super) next: IntPoint,
    pub(super) sort: VSegment,
    pub(super) content: Content,
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

impl Default for Section {
    #[inline]
    fn default() -> Self {
        Self {
            prev: IntPoint::ZERO,
            next: IntPoint::ZERO,
            sort: Default::default(),
            content: Content::Point(IndexPoint::empty()),
        }
    }
}

impl KeyValue<VSegment> for Section {
    #[inline]
    fn key(&self) -> &VSegment {
        &self.sort
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use crate::plain::section::{Content, Section, VSegment};
    use i_overlay::i_float::int::point::IntPoint;
    use i_tree::EMPTY_REF;
    use i_tree::set::sort::SetCollection;
    use i_tree::set::tree::SetTree;
    use crate::plain::vertex::IndexPoint;

    impl Section {
        fn with_sort(sort: VSegment) -> Section {
            Section {
                sort,
                prev: IntPoint::ZERO,
                next: IntPoint::ZERO,
                content: Content::Point(IndexPoint::empty()),
            }
        }
    }


    #[test]
    fn test_0() {
        let vs = VSegment { a: IntPoint::new(0, 10), b: IntPoint::new(10, 10 )};

        let ord0 = vs.is_under_point_order(IntPoint::new(0, 20));
        let ord1 = vs.is_under_point_order(IntPoint::new(5, 20));
        let ord2 = vs.is_under_point_order(IntPoint::new(10, 20));

        assert_eq!(ord0, Ordering::Less);
        assert_eq!(ord1, Ordering::Less);
        assert_eq!(ord2, Ordering::Less);

        let ord3 = vs.is_under_point_order(IntPoint::new(0, 10));
        let ord4 = vs.is_under_point_order(IntPoint::new(5, 10));
        let ord5 = vs.is_under_point_order(IntPoint::new(10, 10));

        assert_eq!(ord3, Ordering::Equal);
        assert_eq!(ord4, Ordering::Equal);
        assert_eq!(ord5, Ordering::Equal);

        let ord6 = vs.is_under_point_order(IntPoint::new(0, 0));
        let ord7 = vs.is_under_point_order(IntPoint::new(5, 0));
        let ord8 = vs.is_under_point_order(IntPoint::new(10, 0));

        assert_eq!(ord6, Ordering::Greater);
        assert_eq!(ord7, Ordering::Greater);
        assert_eq!(ord8, Ordering::Greater);
    }

    #[test]
    fn test_1() {
        let vs0 = VSegment { a: IntPoint::new(0, 9), b: IntPoint::new(10, 9 )};
        let vs1 = VSegment { a: IntPoint::new(0, 6), b: IntPoint::new(10, 6 )};
        let vs2 = VSegment { a: IntPoint::new(0, 3), b: IntPoint::new(10, 3 )};
        let vs3 = VSegment { a: IntPoint::new(0, 1), b: IntPoint::new(10, 1 )};

        let mut sections = SetTree::new(8);
        sections.insert(Section::with_sort(vs0));
        sections.insert(Section::with_sort(vs1));
        sections.insert(Section::with_sort(vs2));
        sections.insert(Section::with_sort(vs3));

        let i0 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 15)));
        let i1 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 7)));
        let i2 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 4)));
        let i3 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 3)));
        let i4 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 2)));
        let i5 = sections.first_index_less_by(|s| s.is_under_point_order(IntPoint::new(5, 0)));

        let r0 = sections.value_by_index(i0);
        let r1 = sections.value_by_index(i1);
        let r2 = sections.value_by_index(i2);
        let r3 = sections.value_by_index(i3);
        let r4 = sections.value_by_index(i4);

        assert!(r0.eq(&Section::with_sort(vs0)));
        assert!(r1.eq(&Section::with_sort(vs1)));
        assert!(r2.eq(&Section::with_sort(vs2)));
        assert!(r3.eq(&Section::with_sort(vs2)));
        assert!(r4.eq(&Section::with_sort(vs3)));

        assert_eq!(i5, EMPTY_REF);
    }
}
