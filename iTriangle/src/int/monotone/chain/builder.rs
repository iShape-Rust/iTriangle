use crate::int::monotone::chain::vertex::ChainVertex;
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_key_sort::sort::two_keys::TwoKeysSort;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};

pub(crate) struct ChainBuilder;

impl ChainBuilder {
    pub(crate) fn flat_to_vertices<I: IntNumber + SortKey>(
        flat: &FlatContoursBuffer<I>,
        vertices: &mut Vec<ChainVertex<I>>,
    ) {
        vertices.clear();
        for range in flat.ranges.iter() {
            let contour = &flat.points[range.clone()];
            vertices.add_contour(contour);
        }
        vertices.sort_by_swipe_line();
    }

    pub(crate) fn shape_to_vertices<I: IntNumber + SortKey>(
        shape: &IntShape<I>,
        points: Option<&[IntPoint<I>]>,
        vertices: &mut Vec<ChainVertex<I>>,
    ) {
        vertices.clear();
        for contour in shape.iter() {
            vertices.add_contour(contour);
        }
        if let Some(points) = points {
            vertices.add_steiner_points(points);
        }
        vertices.sort_by_swipe_line();
    }

    pub(crate) fn contour_to_vertices<I: IntNumber + SortKey>(
        contour: &IntContour<I>,
        points: Option<&[IntPoint<I>]>,
        vertices: &mut Vec<ChainVertex<I>>,
    ) {
        vertices.clear();
        vertices.add_contour(contour);
        if let Some(points) = points {
            vertices.add_steiner_points(points);
        }
        vertices.sort_by_swipe_line();
    }
}

#[derive(Debug, PartialEq)]
enum DirectionType {
    Next,
    Prev,
}

struct Direction<I: IntNumber> {
    point: IntPoint<I>,
    kind: DirectionType,
}

trait ChainVertexVec<I: IntNumber> {
    fn add_contour(&mut self, contour: &[IntPoint<I>]);
    fn add_steiner_points(&mut self, points: &[IntPoint<I>]);
}

impl<I: IntNumber> ChainVertexVec<I> for Vec<ChainVertex<I>> {
    #[inline]
    fn add_contour(&mut self, contour: &[IntPoint<I>]) {
        let mut prev = contour[contour.len() - 2];
        let mut this = contour[contour.len() - 1];

        for &next in contour.iter() {
            self.push(ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    fn add_steiner_points(&mut self, points: &[IntPoint<I>]) {
        for &this in points {
            self.push(ChainVertex::implant(this));
        }
    }
}

pub(crate) trait ChainVertexExport {
    type Int: IntNumber;
    fn feed_points(&self, points: &mut Vec<IntPoint<Self::Int>>);
}
impl<I: IntNumber> ChainVertexExport for [ChainVertex<I>] {
    type Int = I;

    #[inline]
    fn feed_points(&self, points: &mut Vec<IntPoint<I>>) {
        points.clear();
        points.reserve(self.len());
        let mut index = usize::MAX;
        for v in self.iter() {
            if v.index != index {
                index = v.index;
                points.push(v.this);
            }
        }
    }
}

trait ChainVertexSort {
    fn sort_by_swipe_line(&mut self);
    fn sort_node_in_clockwise_order(&mut self);
}

impl<I: IntNumber + SortKey> ChainVertexSort for [ChainVertex<I>] {
    #[inline]
    fn sort_by_swipe_line(&mut self) {
        self.sort_by_two_keys(false, |v| v.this.x, |v| v.this.y);

        debug_assert_eq!(self[0].index, 0); // must be 0 as default value
        let mut index = self[0].index;
        let mut p = self[0].this;
        let mut i = 0;
        while i < self.len() {
            let mut j = i + 1;
            while j < self.len() {
                let vj = &mut self[j];
                if vj.this != p {
                    index += 1;
                    vj.index = index;
                    p = vj.this;
                    break;
                }

                vj.index = index;
                j += 1;
            }

            if i + 1 < j {
                self[i..j].sort_node_in_clockwise_order();
            }

            i = j;
        }
    }

    fn sort_node_in_clockwise_order(&mut self) {
        let mut dirs = Vec::with_capacity(2 * self.len());
        for v in self.iter() {
            dirs.push(Direction {
                point: v.next,
                kind: DirectionType::Next,
            });
            dirs.push(Direction {
                point: v.prev,
                kind: DirectionType::Prev,
            });
        }

        let c = self.first().unwrap().this;

        dirs.sort_unstable_by(|d0, d1| {
            let a = d0.point;
            let b = d1.point;
            if (a.x < c.x || a.x == c.x && a.y < c.y) && (b.x < c.x || b.x == c.x && b.y < c.y)
                || (a.x > c.x || a.x == c.x && a.y > c.y) && (b.x > c.x || b.x == c.x && b.y > c.y)
            {
                Triangle::clock_order(a, b, c)
            } else if a.x == c.x && b.x == c.x {
                a.y.cmp(&b.y)
            } else {
                a.x.cmp(&b.x)
            }
        });

        if dirs[0].kind == DirectionType::Prev {
            let mut prev = 0;
            for vj in self.iter_mut() {
                let next = prev + 1;
                vj.prev = dirs[prev].point;
                vj.next = dirs[next].point;
                debug_assert_eq!(dirs[prev].kind, DirectionType::Prev);
                debug_assert_eq!(dirs[next].kind, DirectionType::Next);

                prev += 2;
            }
        } else {
            let last_dir = dirs.len() - 1;
            let last_prev = dirs[last_dir].point;

            if c.x < last_prev.x {
                // start with next
                let mut prev = last_dir;
                let mut next = 0;
                for vj in self.iter_mut() {
                    vj.prev = dirs[prev].point;
                    vj.next = dirs[next].point;
                    debug_assert_eq!(dirs[prev].kind, DirectionType::Prev);
                    debug_assert_eq!(dirs[next].kind, DirectionType::Next);

                    prev = next + 1;
                    next += 2;
                }
            } else {
                // skip first next
                let mut prev = 1;
                let last_vert = self.len() - 1;
                for vj in self.iter_mut().take(last_vert) {
                    let next = prev + 1;
                    vj.prev = dirs[prev].point;
                    vj.next = dirs[next].point;
                    debug_assert_eq!(dirs[prev].kind, DirectionType::Prev);
                    debug_assert_eq!(dirs[next].kind, DirectionType::Next);

                    prev += 2;
                }
                let vl = &mut self[last_vert];
                vl.prev = last_prev;
                vl.next = dirs[0].point;
            }
        };
    }
}
