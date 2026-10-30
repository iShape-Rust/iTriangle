use core::cmp::Ordering;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::int::vector::IntVector;

struct Walk<'a, I: OverlayInt> {
    points: &'a [IntPoint<I>],
    index: usize,
    reverse: bool,
    remaining: usize,
    reflected: bool,
    segment: Option<(usize, usize)>,
}

fn angle_cmp<I: OverlayInt>(a: IntVector<I>, b: IntVector<I>) -> Ordering {
    let lower =
        |v: IntVector<I>| v.y < I::Wide::ZERO || (v.y == I::Wide::ZERO && v.x < I::Wide::ZERO);
    lower(a)
        .cmp(&lower(b))
        .then_with(|| I::Wide::ZERO.cmp(&a.cross_product(b)))
}

impl<'a, I: OverlayInt> Walk<'a, I> {
    fn new(points: &'a [IntPoint<I>], reflected: bool) -> Option<Self> {
        let key = |&i: &usize| (points[i].y, points[i].x);
        let index = if reflected {
            (0..points.len()).max_by_key(key)?
        } else {
            (0..points.len()).min_by_key(key)?
        };
        let mut walk = Self {
            points,
            index,
            reverse: false,
            remaining: points.len(),
            reflected,
            segment: None,
        };
        let origin = points[index];
        let mut winding = I::Wide::ZERO;
        for _ in 0..points.len() {
            let a = points[walk.index] - origin;
            let b = points[walk.next_index()] - origin;
            winding = winding + a.cross_product(b);
            walk.advance();
        }
        if winding == I::Wide::ZERO {
            let end = if reflected {
                (0..points.len()).min_by_key(key)?
            } else {
                (0..points.len()).max_by_key(key)?
            };
            walk.index = index;
            walk.segment = Some((index, end));
            walk.remaining = if points[index] == points[end] { 0 } else { 2 };
            return Some(walk);
        }
        walk.index = index;
        walk.remaining = points.len();
        walk.reverse = winding < I::Wide::ZERO;
        Some(walk)
    }

    #[inline]
    fn next_index(&self) -> usize {
        if let Some((start, end)) = self.segment {
            return if self.index == start { end } else { start };
        }
        if self.reverse {
            if self.index == 0 {
                self.points.len() - 1
            } else {
                self.index - 1
            }
        } else if self.index + 1 == self.points.len() {
            0
        } else {
            self.index + 1
        }
    }

    #[inline]
    fn edge(&self) -> IntVector<I> {
        let a = self.points[self.index];
        let b = self.points[self.next_index()];
        if self.reflected {
            a - b
        } else {
            b - a
        }
    }

    #[inline]
    fn point(&self) -> IntPoint<I> {
        let p = self.points[self.index];
        if self.reflected {
            IntPoint::new(-p.x, -p.y)
        } else {
            p
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.index = self.next_index();
        self.remaining -= 1;
    }
}

pub(super) struct ConvexSum<'a, I: OverlayInt> {
    a: Walk<'a, I>,
    b: Walk<'a, I>,
}

impl<'a, I: OverlayInt> ConvexSum<'a, I> {
    pub(super) fn new(a: &'a [IntPoint<I>], b: &'a [IntPoint<I>]) -> Option<Self> {
        Some(Self {
            a: Walk::new(a, false)?,
            b: Walk::new(b, false)?,
        })
    }

    pub(super) fn difference(a: &'a [IntPoint<I>], b: &'a [IntPoint<I>]) -> Option<Self> {
        Some(Self {
            a: Walk::new(a, false)?,
            b: Walk::new(b, true)?,
        })
    }

    fn next_step(&self) -> Option<(bool, IntVector<I>)> {
        if self.a.remaining == 0 && self.b.remaining == 0 {
            return None;
        }
        if self.a.remaining == 0 {
            return Some((false, self.b.edge()));
        }
        if self.b.remaining == 0 {
            return Some((true, self.a.edge()));
        }
        let a = self.a.edge();
        let b = self.b.edge();
        let zero = IntVector::<I>::new(I::Wide::ZERO, I::Wide::ZERO);
        if a == zero || (b != zero && angle_cmp(a, b) != Ordering::Greater) {
            Some((true, a))
        } else {
            Some((false, b))
        }
    }
}

impl<I: OverlayInt> Iterator for ConvexSum<'_, I> {
    type Item = IntPoint<I>;

    fn next(&mut self) -> Option<Self::Item> {
        let point = self.a.point() + self.b.point();
        let zero = IntVector::<I>::new(I::Wide::ZERO, I::Wide::ZERO);
        let mut direction = None;
        while let Some((advance_a, edge)) = self.next_step() {
            if edge != zero {
                if let Some(previous) = direction {
                    if angle_cmp(previous, edge) != Ordering::Equal {
                        break;
                    }
                } else {
                    direction = Some(edge);
                }
            }
            if advance_a {
                self.a.advance();
            } else {
                self.b.advance();
            }
        }
        direction.map(|_| point)
    }
}

#[cfg(test)]
mod tests {
    use super::ConvexSum;
    use alloc::vec::Vec;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int_path;

    #[test]
    fn collinear_edges_and_repeated_vertices_are_merged() {
        let a = int_path![[0, 0], [2, 0], [2, 0], [4, 0], [4, 4], [0, 4]];
        let b = int_path![[0, 0], [2, 0], [2, 2], [0, 2]];
        let sum: Vec<_> = ConvexSum::new(&a, &b).unwrap().collect();
        assert_eq!(sum, int_path![[0, 0], [6, 0], [6, 6], [0, 6]]);
    }

    #[test]
    fn merges_closing_edges_for_all_starts_and_windings() {
        let contour = int_path![
            [0, 0],
            [2, 0],
            [4, 0],
            [4, 2],
            [4, 4],
            [2, 4],
            [0, 4],
            [0, 2],
            [0, 0]
        ];
        let expected = int_path![[0, 0], [8, 0], [8, 8], [0, 8]];
        for reverse_a in [false, true] {
            for reverse_b in [false, true] {
                for start_a in 0..contour.len() {
                    for start_b in 0..contour.len() {
                        let mut a = contour.clone();
                        let mut b = contour.clone();
                        if reverse_a {
                            a.reverse();
                        }
                        if reverse_b {
                            b.reverse();
                        }
                        a.rotate_left(start_a);
                        b.rotate_left(start_b);
                        let mut sum = ConvexSum::new(&a, &b).unwrap();
                        assert_eq!(sum.by_ref().collect::<Vec<_>>(), expected);
                        assert_eq!(sum.next(), None);
                        assert_eq!(sum.next(), None);
                    }
                }
            }
        }
    }

    #[test]
    fn slightly_nonconvex_parts_are_accepted() {
        let a = int_path![[0, 0], [3000, 0], [2999, 1], [3000, 3000], [0, 3000]];
        let point = int_path![[2, 3], [2, 3], [2, 3]];
        let sum: Vec<_> = ConvexSum::new(&a, &point).unwrap().collect();
        let difference: Vec<_> = ConvexSum::difference(&a, &point).unwrap().collect();
        assert_eq!(
            sum,
            a.iter()
                .map(|p| IntPoint::new(p.x + 2, p.y + 3))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            difference,
            a.iter()
                .map(|p| IntPoint::new(p.x - 2, p.y - 3))
                .collect::<Vec<_>>()
        );
    }
}
