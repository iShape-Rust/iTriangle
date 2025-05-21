use alloc::vec::Vec;
use crate::int::monotone::chain::vertex::ChainVertex;
use i_key_sort::bin_key::index::BinLayout;
use i_key_sort::sort::layout::BinStore;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};


pub(crate) struct ChainBuilder {
    pub(crate) vertices: Vec<ChainVertex>,
    bin_store: BinStore<i32>
}

impl ChainBuilder {

    #[inline]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(capacity),
            bin_store: BinStore::empty(0, 0)
        }
    }

    pub(crate) fn shape_to_vertices(&mut self, shape: &IntShape, points: Option<&[IntPoint]>) {
        if let Some(layout) = self.shape_layout_and_reserve(shape, 0) {
            self.bin_store.init(layout);

            for contour in shape.iter() {
                self.bin_store.reserve_bins_space(contour.iter().map(|p|&p.x));
            }
            if let Some(points) = points {
                self.bin_store.reserve_bins_space(points.iter().map(|p|&p.x));
            }

            let count = self.bin_store.prepare_bins();
            self.vertices.resize(count, ChainVertex::EMPTY);

            for contour in shape.iter() {
                self.add_contour_with_bins(contour);
            }

            if let Some(points) = points {
                for &p in points {
                    self.bin_store.feed_vec(&mut self.vertices, ChainVertex::implant(p));
                }
            }

            self.sort_vertices_with_bins();
        } else {
            self.vertices.clear();
            for contour in shape.iter() {
                self.add_contour(contour);
            }
            if let Some(points) = points {
                self.add_steiner_points(points);
            }
            self.sort_vertices();
        }
    }

    pub(crate) fn contour_to_vertices(&mut self, contour: &IntContour, points: Option<&[IntPoint]>) {
        if let Some(layout) = self.contour_layout_and_reserve(contour, 0) {
            self.bin_store.init(layout);
            self.bin_store.reserve_bins_space(contour.iter().map(|p|&p.x));
            if let Some(points) = points {
                self.bin_store.reserve_bins_space(points.iter().map(|p|&p.x));
            }
            let count = self.bin_store.prepare_bins();
            self.vertices.resize(count, ChainVertex::EMPTY);

            self.add_contour_with_bins(contour);
            if let Some(points) = points {
                for &p in points {
                    self.bin_store.feed_vec(&mut self.vertices, ChainVertex::implant(p));
                }
            }

            self.sort_vertices_with_bins();
        } else {
            self.vertices.clear();
            self.add_contour(contour);
            if let Some(points) = points {
                self.add_steiner_points(points);
            }
            self.sort_vertices();
        }
    }

    #[inline]
    pub(crate) fn feed_with_points(&self, points: &mut Vec<IntPoint>) {
        if points.capacity() < self.vertices.len() {
            let additional = self.vertices.len() - points.capacity();
            points.reserve(additional);
        }
        let mut index = usize::MAX;
        for v in self.vertices.iter() {
            if v.index != index {
                index = v.index;
                points.push(v.this);
            }
        }
    }

    #[inline]
    pub(crate) fn to_points(&self) -> Vec<IntPoint> {
        let mut points = Vec::with_capacity(self.vertices.len());
        self.feed_with_points(&mut points);
        points
    }

    #[inline]
    fn contour_layout_and_reserve(&mut self, contour: &IntContour, extra_count: usize) -> Option<BinLayout<i32>> {
        let count = contour.len() + extra_count;

        if count < 64 || count > 1000_000 {
            // direct approach work better for small and large data
            return None
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
    fn shape_layout_and_reserve(&mut self, shape: &IntShape, extra_count: usize) -> Option<BinLayout<i32>> {
        let main_count = shape.iter().fold(0, |s, path| s + path.len());
        let count = main_count + extra_count;

        if count < 64 || count > 1000_000 {
            // direct approach work better for small and large data
            return None
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
    fn add_contour(&mut self, contour: &[IntPoint]) {
        let mut prev = contour[contour.len() - 2];
        let mut this = contour[contour.len() - 1];

        for &next in contour.iter() {
            self.vertices.push(ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    fn add_contour_with_bins(&mut self, contour: &[IntPoint]) {
        let mut prev = contour[contour.len() - 2];
        let mut this = contour[contour.len() - 1];

        for &next in contour.iter() {
            self.bin_store.feed_vec(&mut self.vertices, ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    fn add_steiner_points(&mut self, points: &[IntPoint]) {
        for &this in points {
            self.vertices.push(ChainVertex::implant(this));
        }
    }

    #[inline]
    fn sort_vertices(&mut self) {
        self.vertices.sort_unstable_by(|a, b| a.this.cmp(&b.this));
        self.sort_vertices_clockwise_order()
    }

    fn sort_vertices_clockwise_order(&mut self) {
        debug_assert_eq!(self.vertices[0].index, 0); // must be 0 as default value
        let mut index = self.vertices[0].index;
        let mut p = self.vertices[0].this;
        let mut i = 0;
        while i < self.vertices.len() {
            let mut j = i + 1;
            while j < self.vertices.len() {
                let vj = &mut self.vertices[j];
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
                sort_in_clockwise_order(&mut self.vertices[i..j]);
            }

            i = j;
        }
    }

    #[inline]
    fn sort_vertices_with_bins(&mut self) {
        for bin in self.bin_store.bins.iter() {
            let start = bin.offset;
            let end = bin.data;
            if start < end {
                self.vertices[start..end].sort_by(|a, b| a.this.cmp(&b.this));
            }
        }

        self.sort_vertices_clockwise_order()
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

fn sort_in_clockwise_order(vertices: &mut [ChainVertex]) {
    let mut dirs = Vec::with_capacity(2 * vertices.len());
    for v in vertices.iter() {
        dirs.push(Direction {
            point: v.next,
            kind: DirectionType::Next,
        });
        dirs.push(Direction {
            point: v.prev,
            kind: DirectionType::Prev,
        });
    }

    let c = vertices.first().unwrap().this;

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
        for vj in vertices.iter_mut() {
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
            for vj in vertices.iter_mut() {
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
            let last_vert = vertices.len() - 1;
            for vj in vertices.iter_mut().take(last_vert) {
                let next = prev + 1;
                vj.prev = dirs[prev].point;
                vj.next = dirs[next].point;
                debug_assert_eq!(dirs[prev].kind, DirectionType::Prev);
                debug_assert_eq!(dirs[next].kind, DirectionType::Next);

                prev += 2;
            }
            let vl = &mut vertices[last_vert];
            vl.prev = last_prev;
            vl.next = dirs[0].point;
        }
    };
}