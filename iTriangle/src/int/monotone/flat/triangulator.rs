use crate::int::monotone::chain::builder::ChainVertexExport;
use crate::int::monotone::chain::vertex::{ChainVertex, VertexType};
use crate::int::monotone::flat::section::FlatSection;
use crate::int::monotone::v_segment::VSegment;
use crate::int::triangulation::{IndexType, IntTriangulation};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::util::reserve::Reserve;
use i_tree::set::list::SetList;
use i_tree::set::sort::SetCollection;
use i_tree::set::tree::SetTree;

struct FlatBuilder<'a, I> {
    triangulation: &'a mut IntTriangulation<I>,
}

pub(crate) trait FlatTriangulation {
    fn flat_triangulate_into<I: IndexType>(
        &self,
        triangles_count: usize,
        triangulation: &mut IntTriangulation<I>,
    );
}

impl FlatTriangulation for [ChainVertex] {
    fn flat_triangulate_into<I: IndexType>(
        &self,
        triangles_count: usize,
        triangulation: &mut IntTriangulation<I>,
    ) {
        triangulation.indices.reserve_capacity(triangles_count);
        triangulation.indices.clear();

        let mut builder = FlatBuilder::new(triangulation);

        let n = self.len();
        let capacity = if n < 128 { 4 } else { n.ilog2() as usize };
        if capacity <= 12 {
            builder.triangulate(self, SetList::new(capacity));
        } else {
            builder.triangulate(self, SetTree::new(capacity));
        }

        self.feed_points(&mut triangulation.points);
    }
}

impl<'a, I> FlatBuilder<'a, I> {
    fn new(triangulation: &'a mut IntTriangulation<I>) -> Self {
        Self { triangulation }
    }
}

impl<I: IndexType> FlatBuilder<'_, I> {
    #[inline]
    fn triangulate<S: SetCollection<VSegment, FlatSection>>(
        &mut self,
        vertices: &[ChainVertex],
        mut store: S,
    ) {
        for v in vertices.iter() {
            match v.get_type() {
                VertexType::Start => self.start(v, &mut store),
                VertexType::End => self.end(v, &mut store),
                VertexType::Merge => self.merge(v, &mut store),
                VertexType::Split => self.split(v, &mut store),
                VertexType::Join => self.join(v, &mut store),
                VertexType::Steiner => {}
            }
        }
    }

    #[inline]
    fn join<S: SetCollection<VSegment, FlatSection>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        if section.sort.b == v.this {
            section.add_to_bottom(v, &mut self.triangulation.indices);
        } else {
            section.add_to_top(v, &mut self.triangulation.indices);
        }
    }

    #[inline]
    fn start<S: SetCollection<VSegment, FlatSection>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let section = FlatSection {
            sort: VSegment {
                a: v.this,
                b: v.next,
            },
            points: vec![v.index_point()],
        };
        tree.insert(section);
    }

    #[inline]
    fn end<S: SetCollection<VSegment, FlatSection>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        section.add_as_last(v, &mut self.triangulation.indices);
        tree.delete_by_index(index);
    }

    #[inline]
    fn split<S: SetCollection<VSegment, FlatSection>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let index = tree.find_section(v);
        let section = tree.value_by_index_mut(index);
        let new_section = section.add_to_middle(v, &mut self.triangulation.indices);
        tree.insert(new_section);
    }

    fn merge<S: SetCollection<VSegment, FlatSection>>(&mut self, v: &ChainVertex, tree: &mut S) {
        let prev_index = tree.find_section(v);
        let next_index = tree.index_before(prev_index);
        let next = tree.value_by_index_mut(next_index);
        next.add_from_start(v, &mut self.triangulation.indices);

        let mut next_points = if next.points.len() > 1 {
            next.points[1..].to_vec()
        } else {
            Vec::new()
        };

        let sort = next.sort;

        let prev = tree.value_by_index_mut(prev_index);
        prev.add_from_end(v, &mut self.triangulation.indices);

        prev.sort = sort;
        prev.points.append(&mut next_points);

        prev.sort = sort;

        tree.delete_by_index(next_index);
    }
}

impl FlatSection {
    #[inline]
    fn add_as_last<I: IndexType>(&mut self, v: &ChainVertex, triangles: &mut Vec<I>) {
        debug_assert!(self.points.len() >= 2);

        let a = v.index_point();
        for w in self.points.windows(2) {
            triangles.add_abc(a.index, w[0].index, w[1].index);
        }
    }

    #[inline]
    fn add_to_top<I: IndexType>(&mut self, v: &ChainVertex, triangles: &mut Vec<I>) {
        self.add_from_start(v, triangles);
    }

    #[inline]
    fn add_to_bottom<I: IndexType>(&mut self, v: &ChainVertex, triangles: &mut Vec<I>) {
        self.sort = VSegment {
            a: v.this,
            b: v.next,
        };
        self.add_from_end(v, triangles);
    }

    fn add_to_middle<I: IndexType>(
        &mut self,
        v: &ChainVertex,
        triangles: &mut Vec<I>,
    ) -> FlatSection {
        debug_assert!(!self.points.is_empty());
        let a = v.index_point();
        let mut b = self.points[0];
        if self.points.len() <= 1 {
            self.points.push(a);

            let bottom_section = FlatSection {
                sort: self.sort,
                points: vec![a, b],
            };

            self.sort = VSegment {
                a: v.this,
                b: v.next,
            };

            return bottom_section;
        }

        // skip first not valid triangles

        let mut i = 1;
        while i < self.points.len() {
            let c = self.points[i];
            if Triangle::is_cw_or_line_point(a.point, b.point, c.point) {
                i += 1;
                b = c;
                continue;
            }
            break;
        }

        let bottom_points = if i == self.points.len() {
            // do not add any triangles
            // we still must split section
            // peak the closest point by x to a.x
            let mut split_index = 0;
            let mut min_dist = i32::MAX;
            for (i, v) in self.points.iter().enumerate() {
                let dist = a.point.x - v.point.x;
                if dist < min_dist {
                    min_dist = dist;
                    split_index = i;
                }
            }

            let mut bottom_points = self.points.split_off(split_index);
            self.points.push(bottom_points[0]);
            self.points.push(a);
            bottom_points.insert(0, a);

            bottom_points
        } else {
            // we have at least one triangle

            // this section will be top
            // new section will be bottom

            let mut bottom_points = self.points.split_off(i);
            self.points.push(a);

            let c0 = bottom_points[0];
            triangles.add_abc(a.index, b.index, c0.index);
            b = c0;

            i = 1;
            let mut n = 0;
            while i < bottom_points.len() {
                let c = bottom_points[i];
                if Triangle::is_cw_or_line_point(a.point, b.point, c.point) {
                    break;
                }
                triangles.add_abc(a.index, b.index, bottom_points[i].index);
                n += 1;
                i += 1;
                b = c;
            }

            if n > 0 {
                bottom_points.drain(1..n);
                bottom_points[0] = a;
            } else {
                bottom_points.insert(0, a);
            }

            bottom_points
        };

        let bottom_sort = self.sort;
        self.sort = VSegment {
            a: v.this,
            b: v.next,
        };

        FlatSection {
            sort: bottom_sort,
            points: bottom_points,
        }
    }

    fn add_from_start<I: IndexType>(&mut self, v: &ChainVertex, triangles: &mut Vec<I>) {
        let a = v.index_point();
        debug_assert!(!self.points.is_empty());
        if self.points.len() <= 1 {
            self.points.insert(0, a);
            return;
        }

        let mut n = 0;
        let mut b = *self.points.first().unwrap();
        for &c in self.points.iter().skip(1) {
            if Triangle::is_cw_or_line_point(a.point, b.point, c.point) {
                break;
            }
            n += 1;
            triangles.add_abc(a.index, b.index, c.index);
            b = c;
        }

        if n == 0 {
            self.points.insert(0, a);
        } else {
            if n > 1 {
                self.points.drain(1..n);
            }
            self.points[0] = a;
        }
    }

    fn add_from_end<I: IndexType>(&mut self, v: &ChainVertex, triangles: &mut Vec<I>) {
        let a = v.index_point();
        debug_assert!(!self.points.is_empty());
        if self.points.len() <= 1 {
            self.points.push(a);
            return;
        }

        let mut n = 0;
        let mut c = *self.points.last().unwrap();
        for &b in self.points.iter().rev().skip(1) {
            if Triangle::is_cw_or_line_point(a.point, b.point, c.point) {
                break;
            }
            n += 1;
            triangles.add_abc(a.index, b.index, c.index);
            c = b;
        }

        self.points.truncate(self.points.len() - n);
        self.points.push(a);
    }
}

trait AddTriangle {
    fn add_abc(&mut self, a: usize, b: usize, c: usize);
}

impl<I: IndexType> AddTriangle for Vec<I> {
    fn add_abc(&mut self, a: usize, b: usize, c: usize) {
        unsafe {
            self.push(I::try_from(a).unwrap_unchecked());
            self.push(I::try_from(b).unwrap_unchecked());
            self.push(I::try_from(c).unwrap_unchecked());
        }
    }
}

trait FindSection {
    fn find_section(&self, v: &ChainVertex) -> u32;
}

impl<C> FindSection for C
where
    C: SetCollection<VSegment, FlatSection>,
{
    #[inline]
    fn find_section(&self, v: &ChainVertex) -> u32 {
        self.first_index_less_by(|s| {
            let point_search = s.is_under_point_order(v.this);
            match point_search {
                Ordering::Equal => {
                    if v.prev == s.a {
                        Ordering::Equal
                    } else {
                        Triangle::clock_order_point(s.a, v.next, s.b)
                    }
                }
                _ => point_search,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::int::monotone::triangulator::MonotoneTriangulator;
    use crate::int::triangulation::IntTriangulation;
    use alloc::vec;
    use alloc::vec::Vec;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::core::simplify::Simplify;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::area::Area;
    use i_overlay::i_shape::int::path::IntPath;
    use rand::Rng;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    #[test]
    fn test_0() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_1() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(5, 0),
            IntPoint::new(0, 5),
            IntPoint::new(-5, 0),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_2() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(5, 10),
            IntPoint::new(0, 10),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_3() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 5),
            IntPoint::new(0, 10),
            IntPoint::new(5, 5),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_4() {
        let shape = vec![vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_5() {
        let shape = vec![vec![
            IntPoint::new(-15, -15),
            IntPoint::new(15, -15),
            IntPoint::new(25, 0),
            IntPoint::new(15, 15),
            IntPoint::new(-15, 15),
            IntPoint::new(-25, 0),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 4);
        raw.validate(shape_area);
    }

    #[test]
    fn test_6() {
        let shape = vec![vec![
            IntPoint::new(0, -5),
            IntPoint::new(-10, -15),
            IntPoint::new(10, -5),
            IntPoint::new(5, 0),
            IntPoint::new(10, 5),
            IntPoint::new(-10, 15),
            IntPoint::new(0, 5),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 5);
        raw.validate(shape_area);
    }

    #[test]
    fn test_7() {
        let shape = vec![vec![
            IntPoint::new(15, -15),
            IntPoint::new(0, 15),
            IntPoint::new(0, 0),
            IntPoint::new(-15, 0),
            IntPoint::new(-15, -15),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 3);
        raw.validate(shape_area);
    }

    #[test]
    fn test_8() {
        let shape = vec![vec![
            IntPoint::new(-5, -10),
            IntPoint::new(-10, -15),
            IntPoint::new(5, -20),
            IntPoint::new(0, 0),
            IntPoint::new(5, 20),
            IntPoint::new(-10, 15),
            IntPoint::new(-5, 10),
            IntPoint::new(-15, 0),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 6);
        raw.validate(shape_area);
    }

    #[test]
    fn test_9() {
        let shape = vec![vec![
            IntPoint::new(-5, -10),
            IntPoint::new(-10, -15),
            IntPoint::new(-2, -20),
            IntPoint::new(5, -20),
            IntPoint::new(0, 0),
            IntPoint::new(5, 20),
            IntPoint::new(-2, 20),
            IntPoint::new(-10, 15),
            IntPoint::new(-5, 10),
            IntPoint::new(-15, 0),
        ]];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 8);
        raw.validate(shape_area);
    }

    #[test]
    fn test_10() {
        let shape = vec![
            path(&[[-15, -15], [15, -15], [15, 15], [-15, 15]]),
            path(&[[-10, -5], [-10, 5], [0, 0]]),
            path(&[[5, -10], [-5, -10], [0, 0]]),
            path(&[[10, 5], [10, -5], [0, 0]]),
            path(&[[-5, 10], [5, 10], [0, 0]]),
        ];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 16);
        raw.validate(shape_area);
    }

    #[test]
    fn test_11() {
        let shape = vec![
            path(&[[-5, -5], [20, -5], [20, 20], [-5, 20]]),
            path(&[[0, 0], [0, 5], [5, 5], [5, 0]]),
            path(&[[0, 10], [0, 15], [5, 15], [5, 10]]),
            path(&[[10, 0], [10, 5], [15, 5], [15, 0]]),
            path(&[[10, 10], [10, 15], [15, 15], [15, 10]]),
            path(&[[5, 5], [5, 10], [10, 10], [10, 5]]),
        ];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 24);
        raw.validate(shape_area);
    }

    #[test]
    fn test_12() {
        let shape = vec![
            path(&[
                [-30, -30],
                [0, -15],
                [30, -30],
                [15, 0],
                [30, 30],
                [0, 15],
                [-30, 30],
                [-15, 0],
            ]),
            path(&[
                [-20, 20],
                [0, 10],
                [20, 20],
                [10, 0],
                [20, -20],
                [0, -10],
                [-20, -20],
                [-10, 0],
            ]),
        ];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 16);
        raw.validate(shape_area);
    }

    #[test]
    fn test_13() {
        let shape = vec![path(&[
            [-15, 15],
            [10, 15],
            [18, -15],
            [15, -15],
            [30, -30],
            [15, 0],
            [30, 30],
            [-15, 30],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 6);
        raw.validate(shape_area);
    }

    #[test]
    fn test_14() {
        let shape = vec![path(&[[-2, -3], [-4, -4], [5, -1], [1, -1], [2, 3]])];
        let s = &shape.simplify(FillRule::EvenOdd, IntOverlayOptions::default())[0];

        let shape_area = s.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 3);
        raw.validate(shape_area);
    }

    #[test]
    fn test_15() {
        let shape = vec![path(&[[0, 2], [2, 0], [5, 0], [4, 6]])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 2);
        raw.validate(shape_area);
    }

    #[test]
    fn test_16() {
        let shape = vec![path(&[[0, 4], [-4, -3], [-2, -2], [1, -2], [0, -1]])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 3);
        raw.validate(shape_area);
    }

    #[test]
    fn test_17() {
        let shape = vec![path(&[
            [-1, -2],
            [-2, -2],
            [1, -4],
            [1, -1],
            [3, -1],
            [1, -2],
            [5, -2],
            [0, 5],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 6);
        raw.validate(shape_area);
    }

    #[test]
    fn test_18() {
        let shape = vec![path(&[
            [3, 3],
            [-4, 3],
            [1, -2],
            [-2, 2],
            [0, 1],
            [1, -2],
            [1, -4],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 5);
        raw.validate(shape_area);
    }

    #[test]
    fn test_19() {
        let shape = vec![path(&[
            [-2, 0],
            [-3, 2],
            [0, -10],
            [2, 1],
            [-1, 2],
            [-1, 5],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 4);
        raw.validate(shape_area);
    }

    #[test]
    fn test_20() {
        let shape = vec![path(&[
            [5, 5],
            [-5, 1],
            [2, 0],
            [-2, 2],
            [1, 3],
            [2, 0],
            [2, -5],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 5);
        raw.validate(shape_area);
    }

    #[test]
    fn test_21() {
        let shape = vec![path(&[
            [-2, 0],
            [-5, 1],
            [5, -5],
            [3, -1],
            [-1, 0],
            [2, 0],
            [3, -1],
            [4, 4],
        ])];
        let shape_area = shape.area_two();

        let mut raw = IntTriangulation::<u32>::default();
        MonotoneTriangulator::default().shape_into_flat_triangulation(&shape, &mut raw);

        assert_eq!(raw.indices.len() / 3, 6);
        raw.validate(shape_area);
    }

    #[test]
    fn test_random_0() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..100_000 {
            let path = random(8, 5);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    #[test]
    fn test_random_1() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..100_000 {
            let path = random(10, 6);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    #[test]
    fn test_random_2() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..100_000 {
            let path = random(10, 12);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    #[test]
    fn test_random_3() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..50_000 {
            let path = random(20, 20);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    #[test]
    fn test_random_4() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..10_000 {
            let path = random(30, 50);
            let shape = vec![path];
            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    #[test]
    fn test_random_5() {
        let mut raw = IntTriangulation::<u32>::default();
        for _ in 0..2_000 {
            let main = random(50, 20);
            let mut shape = vec![main];
            for _ in 0..10 {
                shape.push(random(30, 5));
            }

            if let Some(first) = shape
                .simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points())
                .first()
            {
                let shape_area = first.area_two();

                MonotoneTriangulator::default().shape_into_flat_triangulation(&first, &mut raw);

                raw.validate(shape_area);
            };
        }
    }

    fn random(radius: i32, n: usize) -> IntPath {
        let a = radius / 2;
        let mut points = Vec::with_capacity(n);
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a);
            let y = rng.random_range(-a..=a);
            points.push(IntPoint { x, y })
        }

        points
    }
}
