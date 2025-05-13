use crate::geom::point::IndexPoint;
use i_key_sort::bin_key::index::BinKey;
use i_key_sort::bin_key::index::BinLayout;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;

#[derive(Debug, Clone, Copy)]
pub(super) enum VertexType {
    Start,
    End,
    Merge,
    Split,
    Join,
    Steiner,
}

#[derive(Debug, Clone)]
pub(crate) struct ChainVertex {
    pub(super) index: usize,
    pub(super) this: IntPoint,
    pub(super) next: IntPoint,
    pub(super) prev: IntPoint,
}

impl ChainVertex {
    pub(super) const EMPTY: ChainVertex = ChainVertex {
        index: 0,
        this: IntPoint::EMPTY,
        next: IntPoint::EMPTY,
        prev: IntPoint::EMPTY,
    };

    #[inline]
    pub(super) fn new(this: IntPoint, next: IntPoint, prev: IntPoint) -> Self {
        Self {
            index: 0,
            this,
            next,
            prev,
        }
    }

    #[inline]
    pub(super) fn implant(this: IntPoint) -> Self {
        Self {
            index: 0,
            this,
            next: IntPoint::EMPTY,
            prev: IntPoint::EMPTY,
        }
    }

    #[inline]
    pub(super) fn get_type(&self) -> VertexType {
        let clock_wise = Triangle::is_clockwise_point(self.prev, self.this, self.next);
        if self.prev == IntPoint::EMPTY && self.next == IntPoint::EMPTY {
            VertexType::Steiner
        } else if self.prev < self.this && self.next < self.this {
            if clock_wise {
                VertexType::Merge
            } else {
                VertexType::End
            }
        } else if self.this < self.next && self.this < self.prev {
            if clock_wise {
                VertexType::Split
            } else {
                VertexType::Start
            }
        } else {
            VertexType::Join
        }
    }

    #[inline]
    pub(super) fn index_point(&self) -> IndexPoint {
        IndexPoint::new(self.index, self.this)
    }
}

impl BinKey<i32> for ChainVertex {
    #[inline]
    fn bin_key(&self) -> i32 {
        self.this.x
    }

    #[inline]
    fn bin_index(&self, layout: &BinLayout<i32>) -> usize {
        layout.index(self.this.x)
    }
}

pub(crate) trait IntoPoints {
    fn into_points(self) -> Vec<IntPoint>;
}

impl IntoPoints for Vec<ChainVertex> {
    #[inline]
    fn into_points(self) -> Vec<IntPoint> {
        let mut points = Vec::with_capacity(self.len());
        let mut index = usize::MAX;
        for v in self.iter() {
            if v.index != index {
                index = v.index;
                points.push(v.this);
            }
        }
        points
    }
}