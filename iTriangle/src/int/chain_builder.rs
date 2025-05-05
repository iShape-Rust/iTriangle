use crate::int::chain_builder_bin::ChainVerticesBinBuilder;
use crate::int::chain_builder_direct::ChainVerticesDirectBuilder;
use crate::int::chain_vertex::ChainVertex;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};

pub(super) trait ToChainVertices {
    fn to_chain_vertices(&self) -> Vec<ChainVertex>;
    fn to_chain_vertices_with_steiner_points(&self, points: &[IntPoint]) -> Vec<ChainVertex>;
}

impl ToChainVertices for IntShape {
    fn to_chain_vertices(&self) -> Vec<ChainVertex> {
        if let Some(mut builder) = ChainVerticesBinBuilder::with_shape(self, 0) {
            for contour in self.iter() {
                builder.reserve_space_for_contour(contour);
            }

            builder.prepare_space();

            let mut vertices = vec![ChainVertex::EMPTY; builder.count];

            for contour in self.iter() {
                builder.add_contour_to_vertices(contour, &mut vertices);
            }

            builder.sort_chain_vertices(&mut vertices);

            vertices
        } else {
            let capacity = self.iter().fold(0, |s, path| s + path.len());
            let mut builder = ChainVerticesDirectBuilder::with_capacity(capacity);

            for path in self.iter() {
                builder.add_path(path);
            }

            builder.into_chain_vertices()
        }
    }

    fn to_chain_vertices_with_steiner_points(&self, points: &[IntPoint]) -> Vec<ChainVertex> {
        if let Some(mut builder) = ChainVerticesBinBuilder::with_shape(self, points.len()) {
            for contour in self.iter() {
                builder.reserve_space_for_contour(contour);
            }

            builder.reserve_space_for_points(points);

            builder.prepare_space();

            let mut vertices = vec![ChainVertex::EMPTY; builder.count];

            for contour in self.iter() {
                builder.add_contour_to_vertices(contour, &mut vertices);
            }

            builder.add_steiner_points_to_vertices(points, &mut vertices);

            builder.sort_chain_vertices(&mut vertices);

            vertices
        } else {
            let capacity = self.iter().fold(0, |s, path| s + path.len()) + points.len();
            let mut builder = ChainVerticesDirectBuilder::with_capacity(capacity);

            for path in self.iter() {
                builder.add_path(path);
            }

            builder.add_steiner_points(points);

            builder.into_chain_vertices()
        }
    }
}
impl ToChainVertices for IntContour {
    fn to_chain_vertices(&self) -> Vec<ChainVertex> {
        if let Some(mut builder) = ChainVerticesBinBuilder::with_contour(self, 0) {
            builder.reserve_space_for_contour(self);
            builder.prepare_space();

            let mut vertices = vec![ChainVertex::EMPTY; builder.count];

            builder.add_contour_to_vertices(self, &mut vertices);

            builder.sort_chain_vertices(&mut vertices);

            vertices
        } else {
            let mut builder = ChainVerticesDirectBuilder::with_capacity(self.len());
            builder.add_path(self);
            builder.into_chain_vertices()
        }
    }

    fn to_chain_vertices_with_steiner_points(&self, points: &[IntPoint]) -> Vec<ChainVertex> {
        if let Some(mut builder) = ChainVerticesBinBuilder::with_contour(self, points.len()) {
            builder.reserve_space_for_contour(self);
            builder.reserve_space_for_points(points);
            builder.prepare_space();

            let mut vertices = vec![ChainVertex::EMPTY; builder.count];

            builder.add_contour_to_vertices(self, &mut vertices);
            builder.add_steiner_points_to_vertices(points, &mut vertices);

            builder.sort_chain_vertices(&mut vertices);

            vertices
        } else {
            let mut builder = ChainVerticesDirectBuilder::with_capacity(self.len() + points.len());
            builder.add_path(self);
            builder.add_steiner_points(points);
            builder.into_chain_vertices()
        }
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

pub(super) fn sort_in_clockwise_order(vertices: &mut [ChainVertex]) {
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

#[cfg(test)]
mod tests {

    use crate::int::chain_builder::{sort_in_clockwise_order, ToChainVertices};
    use crate::int::chain_vertex::ChainVertex;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::shape::IntShape;

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
        let vertices = shape.to_chain_vertices();

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

    #[test]
    fn test_6() {
        let v0 = ChainVertex::new(
            IntPoint::new(2, 0),
            IntPoint::new(2, -5),
            IntPoint::new(1, 3),
        );
        let v1 = ChainVertex::new(
            IntPoint::new(2, 0),
            IntPoint::new(-2, 2),
            IntPoint::new(-5, 1),
        );

        let mut vv0 = vec![v0.clone(), v1.clone()];
        sort_in_clockwise_order(&mut vv0);

        assert_eq!(vv0[0].next, IntPoint::new(-2, 2));
        assert_eq!(vv0[0].prev, IntPoint::new(-5, 1));
        assert_eq!(vv0[1].next, IntPoint::new(2, -5));
        assert_eq!(vv0[1].prev, IntPoint::new(1, 3));

        let mut vv1 = vec![v1, v0];
        sort_in_clockwise_order(&mut vv1);

        assert_eq!(vv1[0].next, IntPoint::new(-2, 2));
        assert_eq!(vv1[0].prev, IntPoint::new(-5, 1));
        assert_eq!(vv1[1].next, IntPoint::new(2, -5));
        assert_eq!(vv1[1].prev, IntPoint::new(1, 3));
    }

    #[test]
    fn test_7() {
        let v0 = ChainVertex::new(
            IntPoint::new(3, -1),
            IntPoint::new(-1, 0),
            IntPoint::new(5, -5),
        );
        let v1 = ChainVertex::new(
            IntPoint::new(3, -1),
            IntPoint::new(4, 4),
            IntPoint::new(2, 0),
        );

        let mut vv0 = vec![v0.clone(), v1.clone()];
        sort_in_clockwise_order(&mut vv0);

        assert_eq!(vv0[0].next, IntPoint::new(-1, 0));
        assert_eq!(vv0[0].prev, IntPoint::new(5, -5));
        assert_eq!(vv0[1].next, IntPoint::new(4, 4));
        assert_eq!(vv0[1].prev, IntPoint::new(2, 0));

        let mut vv1 = vec![v1, v0];
        sort_in_clockwise_order(&mut vv1);

        assert_eq!(vv1[0].next, IntPoint::new(-1, 0));
        assert_eq!(vv1[0].prev, IntPoint::new(5, -5));
        assert_eq!(vv1[1].next, IntPoint::new(4, 4));
        assert_eq!(vv1[1].prev, IntPoint::new(2, 0));
    }

    #[test]
    fn test_8() {
        let shape: IntShape = vec![vec![
            IntPoint::new(3, 1),
            IntPoint::new(-2, 2),
            IntPoint::new(0, -4),
        ]];
        let vertices = shape.to_chain_vertices();

        assert_eq!(vertices.len(), 3);
    }
}
