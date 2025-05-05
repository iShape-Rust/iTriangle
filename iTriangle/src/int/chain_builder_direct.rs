use i_overlay::i_float::int::point::IntPoint;
use crate::int::chain_builder::sort_in_clockwise_order;
use crate::int::chain_vertex::ChainVertex;

pub(super) struct ChainVerticesDirectBuilder {
    vertices: Vec<ChainVertex>,
}

impl ChainVerticesDirectBuilder {
    #[inline]
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub(super) fn add_path(&mut self, path: &[IntPoint]) {
        let n = path.len();
        if n < 3 {
            return;
        }

        let mut prev = path[n - 2];
        let mut this = path[n - 1];

        for &next in path.iter() {
            self.vertices.push(ChainVertex::new(this, next, prev));
            prev = this;
            this = next;
        }
    }

    #[inline]
    pub(super) fn add_steiner_points(&mut self, points: &[IntPoint]) {
        for &this in points {
            self.vertices.push(ChainVertex::implant(this));
        }
    }

    pub(super) fn into_chain_vertices(self) -> Vec<ChainVertex> {
        let mut vertices = self.vertices;
        vertices.sort_unstable_by(|a, b| a.this.cmp(&b.this));

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

        vertices
    }
}