use crate::raw::v_segment::VSegment;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::IntShape;
use i_tree::key::exp::KeyExpCollection;
use i_tree::key::tree::KeyExpTree;
use i_tree::ExpiredKey;

#[derive(Debug, Clone, Copy)]
struct SegmentData {
    direction: bool,
    shape_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct IdSegment {
    data: SegmentData,
    v_segment: VSegment,
}

impl ExpiredKey<i32> for VSegment {
    fn expiration(&self) -> i32 {
        self.b.x
    }
}

pub(super) trait SteinerInference {
    fn group_by_shapes(&self, points: &[IntPoint]) -> Vec<Vec<IntPoint>>;
}

impl SteinerInference for [IntShape] {
    fn group_by_shapes(&self, points: &[IntPoint]) -> Vec<Vec<IntPoint>> {
        if points.is_empty() {
            return vec![Vec::new(); self.len()];
        }

        let mut points = points.to_vec();
        points.sort_unstable_by(|a, b| a.x.cmp(&b.x));

        let x_min = points[0].x;
        let x_max = points.last().unwrap().x;

        let mut segments = Vec::new();

        for (shape_index, shape) in self.iter().enumerate() {
            for path in shape.iter() {
                let mut a = *path.last().unwrap();
                for &b in path.iter() {
                    if a.x == b.x || a.x < x_min && b.x < x_min || a.x > x_max && b.x > x_max {
                        a = b;
                        continue;
                    }
                    let direction = a.x < b.x;
                    let v_segment = if direction {
                        VSegment { a, b }
                    } else {
                        VSegment { a: b, b: a }
                    };
                    segments.push(IdSegment {
                        data: SegmentData {
                            direction,
                            shape_index,
                        },
                        v_segment,
                    });

                    a = b;
                }
            }
        }

        let mut groups = vec![Vec::new(); self.len()];
        let capacity = segments.len().ilog2() as usize;
        let mut tree = KeyExpTree::new(capacity);

        let empty_data = SegmentData {
            direction: false,
            shape_index: usize::MAX,
        };

        let mut i = 0;
        for &p in points.iter() {
            while i < segments.len() {
                let id_segment = &segments[i];
                if p.x < id_segment.v_segment.a.x {
                    break;
                }
                
                if p.x < id_segment.v_segment.b.x {
                    tree.insert(id_segment.v_segment, id_segment.data, p.x);
                }
                i += 1
            }

            let segment_data = tree.first_less_by(p.x, empty_data, |s| s.is_under_point_order(p));

            if segment_data.direction && segment_data.shape_index < groups.len() {
                groups[segment_data.shape_index].push(p);
            }
        }

        groups
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::binder::SteinerInference;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::path::IntPath;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    #[test]
    fn test_0() {
        let shapes = vec![vec![path(&[[0, 0], [10, 0], [10, 10], [0, 10]])]];

        let groups = shapes.group_by_shapes(&[
            IntPoint::new(5, 5),
            IntPoint::new(15, 5),
            IntPoint::new(-15, 5),
        ]);

        assert_eq!(groups[0].len(), 1);
    }

    #[test]
    fn test_1() {
        let shapes = vec![
            vec![path(&[[0, 0], [10, 0], [10, 10], [0, 10]])],
            vec![path(&[[20, 0], [30, 0], [30, 10], [20, 10]])]
        ];

        let groups = shapes.group_by_shapes(&[
            IntPoint::new(5, 5),
            IntPoint::new(15, 5),
            IntPoint::new(25, 5),
            IntPoint::new(-15, 5),
        ]);

        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn test_2() {
        let shapes = vec![
            vec![path(&[[0, 0], [10, 0], [10, 10], [0, 10]])],
            vec![path(&[[0, 20], [10, 20], [10, 30], [0, 30]])],
            vec![path(&[[0, 40], [10, 40], [10, 50], [0, 50]])],
            vec![path(&[[0, 60], [10, 60], [10, 70], [0, 70]])],
            vec![path(&[[0, 80], [10, 80], [10, 90], [0, 90]])],
        ];

        let groups = shapes.group_by_shapes(&[
            IntPoint::new(5, 5),
            IntPoint::new(15, 5),
            IntPoint::new(5, 25),
            IntPoint::new(6, 25),
            IntPoint::new(7, 25),
            IntPoint::new(25, 5),
            IntPoint::new(-15, 5),
        ]);

        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 3);
    }
}
