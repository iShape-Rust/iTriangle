use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::shape::IntShape;

pub(super) enum VertexType {
    Start,
    Merge,
    Other
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PathVertex {
    pub(super) index: usize,
    pub(super) this: IntPoint,
    pub(super) next: IntPoint,
    pub(super) prev: IntPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexPoint {
    pub index: usize,
    pub point: IntPoint,
}

impl IndexPoint {

    #[inline]
    pub(super) fn new(index: usize, point: IntPoint) -> Self {
        Self { index, point }
    }

    #[inline]
    pub(super) const fn empty() -> Self {
        Self {
            index: usize::MAX,
            point: IntPoint::ZERO,
        }
    }
}

impl Default for IndexPoint {
    #[inline]
    fn default() -> Self {
        IndexPoint::empty()
    }
}

impl PathVertex {

    #[inline]
    pub(super) fn index_point(&self) -> IndexPoint {
        IndexPoint::new(self.index, self.this)
    }

    #[inline]
    pub(super) fn vert_type(&self) -> VertexType {
        let clock_wise = Triangle::is_clockwise_point(self.prev, self.this, self.next);
        if clock_wise {
            if self.prev.x < self.this.x && self.next.x <= self.this.x {
                VertexType::Merge
            } else {
                VertexType::Other
            }
        } else {
            if self.this.x <= self.prev.x && self.this.x < self.next.x {
                VertexType::Start
            } else {
                VertexType::Other
            }
        }
    }

}

pub(super) trait ShapeToVertices {
    fn to_vertices(&self, inner_points: &[IntPoint]) -> Vec<PathVertex>;
}

impl ShapeToVertices for IntShape {
    fn to_vertices(&self, inner_points: &[IntPoint]) -> Vec<PathVertex> {
        let capacity = self.iter()
            .fold(0, |s, path| s + path.len()) + inner_points.len();

        let mut vertices = Vec::with_capacity(capacity);
        for path in self.iter() {
            let n = path.len();

            let mut prev = path[n - 2];
            let mut this = path[n - 1];

            for &next in path.iter() {
                vertices.push(PathVertex {
                    index: 0,
                    this,
                    next,
                    prev,
                });
                prev = this;
                this = next;
            }
        }

        for &this in inner_points {
            vertices.push(PathVertex {
                index: 0,
                this,
                next: IntPoint::new(i32::MIN, i32::MIN),
                prev: IntPoint::new(i32::MIN, i32::MIN),
            });
        }

        vertices.sort_unstable_by(|a, b| a.this.cmp(&b.this));

        let mut p = unsafe { vertices.get_unchecked(0) }.this;
        p.x = p.x.wrapping_add(1);
        let mut index = 0;
        for v in vertices.iter_mut() {
            v.index = index;
            if p != v.this {
                index += 1;
            }
            p = v.this;
        }

        vertices
    }
}

#[cfg(test)]
mod tests {
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::shape::IntShape;
    use crate::plain::vertex::ShapeToVertices;

    #[test]
    fn test_0() {
        let shape: IntShape = vec![
            vec![
                IntPoint::new(-10, -10),
                IntPoint::new(10, -10),
                IntPoint::new(10, 10),
                IntPoint::new(-10, 10),
            ],
            vec![
                IntPoint::new(-5, -5),
                IntPoint::new(0, 0),
                IntPoint::new(-5, 5),
                IntPoint::new(5, 5),
                IntPoint::new(0, 0),
                IntPoint::new(5, -5),
            ],
        ];
        let vertices = shape.to_vertices(&vec![]);

        assert_eq!(vertices.len(), 10);
    }
}
