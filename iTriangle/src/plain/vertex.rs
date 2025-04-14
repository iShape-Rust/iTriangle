use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::shape::IntShape;

#[derive(Debug, Clone, Copy)]
pub(super) enum VertexType {
    Start,
    End,
    Merge,
    Split,
    Join,
    Implant,
}

#[derive(Debug, Clone)]
pub(super) struct ChainVertex {
    pub(super) index: usize,
    pub(super) this: IntPoint,
    pub(super) next: IntPoint,
    pub(super) prev: IntPoint
}

#[derive(Debug, Clone, Copy)]
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

impl ChainVertex {

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
            VertexType::Implant
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

pub(super) trait ShapeToVertices {
    fn to_chain_vertices(&self, implants: &[IntPoint]) -> Vec<ChainVertex>;
}

impl ShapeToVertices for IntShape {
    fn to_chain_vertices(&self, implants: &[IntPoint]) -> Vec<ChainVertex> {
        let capacity = self.iter()
            .fold(0, |s, path| s + path.len()) + implants.len();

        let mut vertices = Vec::with_capacity(capacity);
        for path in self.iter() {
            let n = path.len();

            let mut prev = path[n - 2];
            let mut this = path[n - 1];

            for &next in path.iter() {
                vertices.push(ChainVertex::new(this, next, prev));
                prev = this;
                this = next;
            }
        }

        for &this in implants {
            vertices.push(ChainVertex::implant(this));
        }

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

#[derive(Debug, PartialEq)]
enum DirectionType {
    Next,
    Prev
}

struct Direction {
    point: IntPoint,
    kind: DirectionType,
}

fn sort_in_clockwise_order(vertices: &mut [ChainVertex]) {
    let mut dirs = Vec::with_capacity(2 * vertices.len());
    for v in vertices.iter() {
        dirs.push(Direction { point: v.next, kind: DirectionType::Next });
        dirs.push(Direction { point: v.prev, kind: DirectionType::Prev });
    }

    let c = vertices.first().unwrap().this;

    dirs.sort_unstable_by(|d0, d1| {
        let a = d0.point;
        let b = d1.point;
        if (a.x < c.x || a.x == c.x && a.y < c.y) && (b.x < c.x || b.x == c.x && b.y < c.y)
            || (a.x > c.x || a.x == c.x && a.y > c.y) && (b.x > c.x || b.x == c.x && b.y > c.y) {
            Triangle::clock_order_point(a, b, c)
        } else if a.x == c.x && b.x == c.x {
            a.y.cmp(&b.y)
        } else {
            a.x.cmp(&b.x)
        }
    });

    let (mut prev, mut next) = if dirs[0].kind == DirectionType::Prev {
        (0, 1)
    } else {
        (dirs.len() - 1, 0)
    };

    for vj in vertices.iter_mut() {
        vj.prev = dirs[prev].point;
        vj.next = dirs[next].point;
        debug_assert_eq!(dirs[prev].kind, DirectionType::Prev);
        debug_assert_eq!(dirs[next].kind, DirectionType::Next);

        prev = next + 1;
        next += 2;
    }
}


#[cfg(test)]
mod tests {
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::shape::IntShape;
    use crate::plain::vertex::{sort_in_clockwise_order, ChainVertex, ShapeToVertices};

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
        let vertices = shape.to_chain_vertices(&vec![]);

        assert_eq!(vertices.len(), 10);
    }

    #[test]
    fn test_1() {
        let v = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(10, 20),
            IntPoint::new(10, 0),
        );

        let mut vv = vec![v];
        sort_in_clockwise_order(&mut vv);

        assert_eq!(vv[0].next, IntPoint::new(10, 20));
        assert_eq!(vv[0].prev, IntPoint::new(10, 0));
    }

    #[test]
    fn test_2() {
        let v = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(10, 0),
            IntPoint::new(10, 20),
        );

        let mut vv = vec![v];
        sort_in_clockwise_order(&mut vv);

        assert_eq!(vv[0].next, IntPoint::new(10, 0));
        assert_eq!(vv[0].prev, IntPoint::new(10, 20));
    }

    #[test]
    fn test_3() {
        let v0 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(0, 5),
            IntPoint::new(5, 0),
        );
        let v1 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(5, 20),
            IntPoint::new(0, 15),
        );

        let mut vv0 = vec![v0.clone(), v1.clone()];
        sort_in_clockwise_order(&mut vv0);

        assert_eq!(vv0[0].next, IntPoint::new(0, 5));
        assert_eq!(vv0[0].prev, IntPoint::new(5, 0));
        assert_eq!(vv0[1].next, IntPoint::new(5, 20));
        assert_eq!(vv0[1].prev, IntPoint::new(0, 15));

        let mut vv1 = vec![v1, v0];
        sort_in_clockwise_order(&mut vv1);

        assert_eq!(vv1[0].next, IntPoint::new(0, 5));
        assert_eq!(vv1[0].prev, IntPoint::new(5, 0));
        assert_eq!(vv1[1].next, IntPoint::new(5, 20));
        assert_eq!(vv1[1].prev, IntPoint::new(0, 15));
    }

    #[test]
    fn test_4() {
        let v0 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(20, 15),
            IntPoint::new(15, 20),
        );
        let v1 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(15, 0),
            IntPoint::new(20, 5),
        );

        let mut vv0 = vec![v0.clone(), v1.clone()];
        sort_in_clockwise_order(&mut vv0);

        assert_eq!(vv0[0].next, IntPoint::new(20, 15));
        assert_eq!(vv0[0].prev, IntPoint::new(15, 20));
        assert_eq!(vv0[1].next, IntPoint::new(15, 0));
        assert_eq!(vv0[1].prev, IntPoint::new(20, 5));

        let mut vv1 = vec![v1, v0];
        sort_in_clockwise_order(&mut vv1);

        assert_eq!(vv1[0].next, IntPoint::new(20, 15));
        assert_eq!(vv1[0].prev, IntPoint::new(15, 20));
        assert_eq!(vv1[1].next, IntPoint::new(15, 0));
        assert_eq!(vv1[1].prev, IntPoint::new(20, 5));
    }


    #[test]
    fn test_5() {
        let v0 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(20, 0),
            IntPoint::new(10, 0),
        );
        let v1 = ChainVertex::new(
            IntPoint::new(10, 10),
            IntPoint::new(0, 20),
            IntPoint::new(10, 20),
        );

        let mut vv0 = vec![v0.clone(), v1.clone()];
        sort_in_clockwise_order(&mut vv0);

        assert_eq!(vv0[0].next, IntPoint::new(0, 20));
        assert_eq!(vv0[0].prev, IntPoint::new(10, 0));
        assert_eq!(vv0[1].next, IntPoint::new(20, 0));
        assert_eq!(vv0[1].prev, IntPoint::new(10, 20));

        let mut vv1 = vec![v1, v0];
        sort_in_clockwise_order(&mut vv1);

        assert_eq!(vv1[0].next, IntPoint::new(0, 20));
        assert_eq!(vv1[0].prev, IntPoint::new(10, 0));
        assert_eq!(vv1[1].next, IntPoint::new(20, 0));
        assert_eq!(vv1[1].prev, IntPoint::new(10, 20));
    }
}
