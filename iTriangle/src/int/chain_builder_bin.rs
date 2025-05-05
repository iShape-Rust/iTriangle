use crate::int::chain_builder::sort_in_clockwise_order;
use crate::int::chain_vertex::ChainVertex;
use i_key_sort::sort::layout::BinStore;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};

pub(super) struct ChainVerticesBinBuilder {
    pub(super) count: usize,
    store: BinStore<i32>,
}

impl ChainVerticesBinBuilder {
    #[inline]
    pub(super) fn with_contour(contour: &IntContour, extra_count: usize) -> Option<Self> {
        let mut count = extra_count;
        let mut min = i32::MAX;
        let mut max = i32::MIN;

        for p in contour.iter() {
            min = min.min(p.x);
            max = max.max(p.x);
        }
        count += contour.len();

        let store = BinStore::new(min, max, count)?;

        Some(Self { count, store })
    }

    #[inline]
    pub(super) fn with_shape(shape: &IntShape, extra_count: usize) -> Option<Self> {
        let mut count = extra_count;
        let mut min = i32::MAX;
        let mut max = i32::MIN;

        for contour in shape.iter() {
            for p in contour.iter() {
                min = min.min(p.x);
                max = max.max(p.x);
            }
            count += contour.len();
        }

        let store = BinStore::new(min, max, count)?;

        Some(Self { count, store })
    }

    #[inline]
    pub(super) fn add_contour_to_vertices(
        &mut self,
        contour: &[IntPoint],
        vertices: &mut Vec<ChainVertex>,
    ) {
        let n = contour.len();
        if n < 3 {
            return;
        }

        let mut prev = contour[n - 2];
        let mut this = contour[n - 1];

        for &next in contour.iter() {
            self.store
                .feed_vec(vertices, ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    pub(super) fn reserve_space_for_contour(&mut self, contour: &[IntPoint]) {
        let n = contour.len();
        if n < 3 {
            return;
        }
        self.store.reserve_bins_space(contour.iter());
    }

    #[inline]
    pub(super) fn add_steiner_points_to_vertices(
        &mut self,
        points: &[IntPoint],
        vertices: &mut Vec<ChainVertex>,
    ) {
        for &this in points {
            self.store.feed_vec(vertices, ChainVertex::implant(this));
        }
    }

    #[inline]
    pub(super) fn reserve_space_for_points(&mut self, points: &[IntPoint]) {
        self.store.reserve_bins_space(points.iter());
    }

    #[inline]
    pub(super) fn prepare_space(&mut self) {
        self.store.prepare_bins();
    }

    pub(super) fn sort_chain_vertices(&self, vertices: &mut Vec<ChainVertex>) {
        for bin in self.store.bins.iter() {
            let start = bin.offset;
            let end = bin.data;
            if start < end {
                vertices[start..end].sort_by(|a, b| a.this.cmp(&b.this));
            }
        }

        debug_assert_eq!(vertices[0].index, 0); // must be 0 as default value
        let mut index = 0;
        let mut p = vertices[0].this;
        let mut i = 0;
        while i < vertices.len() {
            let mut j = i + 1;
            while j < vertices.len() {
                let vj = &mut vertices[j];
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
                sort_in_clockwise_order(&mut vertices[i..j]);
            }

            i = j;
        }
    }
}
