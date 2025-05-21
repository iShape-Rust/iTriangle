use alloc::vec::Vec;
use crate::geom::point::IndexPoint;
use crate::int::monotone::v_segment::VSegment;
use i_tree::set::sort::KeyValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EdgeType {
    Regular(usize), // keep index to triangle
    Phantom(usize), // keep index to itself(edge) in phantom store
}
#[derive(Debug, Clone, Copy)]
pub(super) struct TriangleEdge {
    pub(super) a: IndexPoint,
    pub(super) b: IndexPoint,
    pub(super) kind: EdgeType,
}

#[derive(Debug, Clone)]
pub(super) enum Content {
    Point(IndexPoint),
    Edges(Vec<TriangleEdge>),
}

#[derive(Debug, Clone)]
pub(super) struct Section {
    pub(super) sort: VSegment,
    pub(super) content: Content,
}

impl Default for Section {
    #[inline]
    fn default() -> Self {
        Self {
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

impl TriangleEdge {
    #[inline]
    pub(super) fn border(a: IndexPoint, b: IndexPoint) -> Self {
        Self {
            a,
            b,
            kind: EdgeType::Regular(usize::MAX),
        }
    }

    #[inline]
    pub(super) fn phantom(a: IndexPoint, b: IndexPoint, index: usize) -> Self {
        Self {
            a,
            b,
            kind: EdgeType::Phantom(index),
        }
    }

    #[inline]
    pub(super) fn regular(a: IndexPoint, b: IndexPoint, index: usize) -> Self {
        Self {
            a,
            b,
            kind: EdgeType::Regular(index),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::point::IndexPoint;
    use crate::int::monotone::section::{Content, Section, VSegment};
    use i_overlay::i_float::int::point::IntPoint;
    use i_tree::set::sort::SetCollection;
    use i_tree::set::tree::SetTree;
    use i_tree::EMPTY_REF;
    use core::cmp::Ordering;

    impl Section {
        fn with_sort(sort: VSegment) -> Section {
            Section {
                sort,
                content: Content::Point(IndexPoint::empty()),
            }
        }
    }

    #[test]
    fn test_0() {
        let vs = VSegment {
            a: IntPoint::new(0, 10),
            b: IntPoint::new(10, 10),
        };

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
        let vs0 = VSegment {
            a: IntPoint::new(0, 9),
            b: IntPoint::new(10, 9),
        };
        let vs1 = VSegment {
            a: IntPoint::new(0, 6),
            b: IntPoint::new(10, 6),
        };
        let vs2 = VSegment {
            a: IntPoint::new(0, 3),
            b: IntPoint::new(10, 3),
        };
        let vs3 = VSegment {
            a: IntPoint::new(0, 1),
            b: IntPoint::new(10, 1),
        };

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

        assert!(r0.sort.eq(&Section::with_sort(vs0).sort));
        assert!(r1.sort.eq(&Section::with_sort(vs1).sort));
        assert!(r2.sort.eq(&Section::with_sort(vs2).sort));
        assert!(r3.sort.eq(&Section::with_sort(vs2).sort));
        assert!(r4.sort.eq(&Section::with_sort(vs3).sort));

        assert_eq!(i5, EMPTY_REF);
    }
}
