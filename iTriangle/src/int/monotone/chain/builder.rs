use crate::int::monotone::chain::vertex::ChainVertex;
use alloc::vec::Vec;
use i_key_sort::bin_key::index::BinLayout;
use i_key_sort::sort::layout::BinStore;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};
use i_overlay::i_shape::util::reserve::Reserve;

pub(crate) struct ChainBuilder {
    bin_store: BinStore<i32>,
}

impl Default for ChainBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            bin_store: BinStore::empty(0, 0),
        }
    }
}

impl ChainBuilder {
    pub(crate) fn flat_to_vertices(
        &mut self,
        flat: &FlatContoursBuffer,
        vertices: &mut Vec<ChainVertex>,
    ) {
        if let Some(layout) = self.flat_layout_and_reserve(flat, 0) {
            self.bin_store.init(layout);

            self.bin_store
                .reserve_bins_space(flat.points.iter().map(|p| &p.x));

            let count = self.bin_store.prepare_bins();
            vertices.resize(count, ChainVertex::EMPTY);

            for range in flat.ranges.iter() {
                let contour = &flat.points[range.clone()];
                vertices.add_contour_with_bins(contour, &mut self.bin_store);
            }
            vertices.sort_by_swipe_line_with_bins(&mut self.bin_store);
        } else {
            vertices.clear();
            for range in flat.ranges.iter() {
                let contour = &flat.points[range.clone()];
                vertices.add_contour(contour);
            }
            vertices.sort_by_swipe_line();
        }
    }

    pub(crate) fn shape_to_vertices(
        &mut self,
        shape: &IntShape,
        points: Option<&[IntPoint]>,
        vertices: &mut Vec<ChainVertex>,
    ) {
        if let Some(layout) = self.shape_layout_and_reserve(shape, 0) {
            self.bin_store.init(layout);

            for contour in shape.iter() {
                self.bin_store
                    .reserve_bins_space(contour.iter().map(|p| &p.x));
            }
            if let Some(points) = points {
                self.bin_store
                    .reserve_bins_space(points.iter().map(|p| &p.x));
            }

            let count = self.bin_store.prepare_bins();
            vertices.resize(count, ChainVertex::EMPTY);

            for contour in shape.iter() {
                vertices.add_contour_with_bins(contour, &mut self.bin_store);
            }

            if let Some(points) = points {
                for &p in points {
                    self.bin_store.feed_vec(vertices, ChainVertex::implant(p));
                }
            }

            vertices.sort_by_swipe_line_with_bins(&mut self.bin_store);
        } else {
            vertices.clear();
            for contour in shape.iter() {
                vertices.add_contour(contour);
            }
            if let Some(points) = points {
                vertices.add_steiner_points(points);
            }
            vertices.sort_by_swipe_line();
        }
    }

    pub(crate) fn contour_to_vertices(
        &mut self,
        contour: &IntContour,
        points: Option<&[IntPoint]>,
        vertices: &mut Vec<ChainVertex>,
    ) {
        if let Some(layout) = self.contour_layout_and_reserve(contour, 0) {
            self.bin_store.init(layout);
            self.bin_store
                .reserve_bins_space(contour.iter().map(|p| &p.x));
            if let Some(points) = points {
                self.bin_store
                    .reserve_bins_space(points.iter().map(|p| &p.x));
            }
            let count = self.bin_store.prepare_bins();
            vertices.resize(count, ChainVertex::EMPTY);

            vertices.add_contour_with_bins(contour, &mut self.bin_store);
            if let Some(points) = points {
                for &p in points {
                    self.bin_store.feed_vec(vertices, ChainVertex::implant(p));
                }
            }

            vertices.sort_by_swipe_line_with_bins(&mut self.bin_store);
        } else {
            vertices.clear();
            vertices.add_contour(contour);
            if let Some(points) = points {
                vertices.add_steiner_points(points);
            }
            vertices.sort_by_swipe_line();
        }
    }

    #[inline]
    fn contour_layout_and_reserve(
        &mut self,
        contour: &[IntPoint],
        extra_count: usize,
    ) -> Option<BinLayout<i32>> {
        let count = contour.len() + extra_count;

        if !(64..=1_000_000).contains(&count) {
            // direct approach work better for small and large data
            return None;
        }

        let mut min = i32::MAX;
        let mut max = i32::MIN;

        for p in contour.iter() {
            min = min.min(p.x);
            max = max.max(p.x);
        }

        BinLayout::new(min..max, count)
    }

    #[inline]
    fn shape_layout_and_reserve(
        &mut self,
        shape: &IntShape,
        extra_count: usize,
    ) -> Option<BinLayout<i32>> {
        let main_count = shape.iter().fold(0, |s, path| s + path.len());
        let count = main_count + extra_count;

        if !(64..=1_000_000).contains(&count) {
            // direct approach work better for small and large data
            return None;
        }

        let mut min = i32::MAX;
        let mut max = i32::MIN;

        for contour in shape.iter() {
            for p in contour.iter() {
                min = min.min(p.x);
                max = max.max(p.x);
            }
        }

        BinLayout::new(min..max, count)
    }

    #[inline]
    fn flat_layout_and_reserve(
        &mut self,
        flat: &FlatContoursBuffer,
        extra_count: usize,
    ) -> Option<BinLayout<i32>> {
        let main_count = flat.points.len();
        let count = main_count + extra_count;

        if !(64..=1_000_000).contains(&count) {
            // direct approach work better for small and large data
            return None;
        }

        let mut min = i32::MAX;
        let mut max = i32::MIN;

        for p in flat.points.iter() {
            min = min.min(p.x);
            max = max.max(p.x);
        }

        BinLayout::new(min..max, count)
    }
}

#[derive(Debug, PartialEq)]
enum DirectionType {
    Next,
    Prev,
}

struct Direction {
    point: IntPoint,
    kind: DirectionType,
}

trait ChainVertexVec {
    fn add_contour(&mut self, contour: &[IntPoint]);
    fn add_contour_with_bins(&mut self, contour: &[IntPoint], bin_store: &mut BinStore<i32>);
    fn add_steiner_points(&mut self, points: &[IntPoint]);
}

impl ChainVertexVec for Vec<ChainVertex> {
    #[inline]
    fn add_contour(&mut self, contour: &[IntPoint]) {
        let mut prev = contour[contour.len() - 2];
        let mut this = contour[contour.len() - 1];

        for &next in contour.iter() {
            self.push(ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    fn add_contour_with_bins(&mut self, contour: &[IntPoint], bin_store: &mut BinStore<i32>) {
        let mut prev = contour[contour.len() - 2];
        let mut this = contour[contour.len() - 1];

        for &next in contour.iter() {
            bin_store.feed_vec(self, ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }
    #[inline]
    fn add_steiner_points(&mut self, points: &[IntPoint]) {
        for &this in points {
            self.push(ChainVertex::implant(this));
        }
    }
}

pub(crate) trait ChainVertexExport {
    fn feed_points(&self, points: &mut Vec<IntPoint>);
}
impl ChainVertexExport for [ChainVertex] {
    #[inline]
    fn feed_points(&self, points: &mut Vec<IntPoint>) {
        points.reserve_capacity(self.len());
        points.clear();
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
    fn sort_by_swipe_line_with_bins(&mut self, bin_store: &mut BinStore<i32>);
    fn sort_possible_nodes(&mut self);
    fn sort_node_in_clockwise_order(&mut self);
}

impl ChainVertexSort for [ChainVertex] {

    #[inline]
    fn sort_by_swipe_line(&mut self) {
        self.sort_unstable_by(|a, b| a.this.cmp(&b.this));
        self.sort_possible_nodes();
    }

    fn sort_by_swipe_line_with_bins(&mut self, bin_store: &mut BinStore<i32>) {
        for bin in bin_store.bins.iter() {
            let start = bin.offset;
            let end = bin.data;
            if start < end {
                self[start..end].sort_by(|a, b| a.this.cmp(&b.this));
            }
        }
        self.sort_possible_nodes();
    }

    fn sort_possible_nodes(&mut self) {
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
                Triangle::clock_order_point(a, b, c)
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
