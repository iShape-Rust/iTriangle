use alloc::vec::Vec;
use i_key_sort::sort::two_keys::TwoKeysSort;
use i_overlay::core::integer::OverlayInt;
use i_overlay::core::point_location::IntPointContainment;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::int::shape::{IntContour, IntShape};

const TRIANGLE_HEIGHT_NUMERATOR: u32 = 28_378;
const TRIANGLE_HEIGHT_SHIFT: u32 = 15;

/// Generates vertices of an equilateral triangular lattice inside integer geometry.
///
/// The input geometry must have resolved topology. A shape may contain holes and a
/// collection of shapes is treated as their union. Candidate points are tested in one
/// batch with [`IntPointContainment`], then points close to any boundary edge are removed.
pub trait IntUniformGrid<I: OverlayInt> {
    /// Returns lattice points contained by the geometry.
    ///
    /// `edge_length` is the horizontal lattice spacing. Consecutive rows are separated by
    /// `round(sqrt(3) / 2 * edge_length)` and shifted by half an edge.
    fn uniform_grid(&self, edge_length: I::WideUInt) -> Vec<IntPoint<I>>;
}

impl<I: OverlayInt> IntUniformGrid<I> for [IntPoint<I>] {
    #[inline]
    fn uniform_grid(&self, edge_length: I::WideUInt) -> Vec<IntPoint<I>> {
        let mut edges = Vec::with_capacity(self.len());
        append_edges(self, &mut edges);
        build_grid(self, self.iter(), &edges, edge_length)
    }
}

impl<I: OverlayInt> IntUniformGrid<I> for [IntContour<I>] {
    #[inline]
    fn uniform_grid(&self, edge_length: I::WideUInt) -> Vec<IntPoint<I>> {
        let mut edges = Vec::new();
        for contour in self {
            append_edges(contour, &mut edges);
        }
        build_grid(self, self.iter().flatten(), &edges, edge_length)
    }
}

impl<I: OverlayInt> IntUniformGrid<I> for [IntShape<I>] {
    #[inline]
    fn uniform_grid(&self, edge_length: I::WideUInt) -> Vec<IntPoint<I>> {
        let mut edges = Vec::new();
        for contour in self.iter().flatten() {
            append_edges(contour, &mut edges);
        }
        build_grid(self, self.iter().flatten().flatten(), &edges, edge_length)
    }
}

#[derive(Clone, Copy)]
struct Edge<I: OverlayInt> {
    a: IntPoint<I>,
    b: IntPoint<I>,
}

fn append_edges<I: OverlayInt>(contour: &[IntPoint<I>], edges: &mut Vec<Edge<I>>) {
    let Some(&mut_a) = contour.last() else {
        return;
    };
    let mut a = mut_a;
    for &b in contour {
        edges.push(Edge { a, b });
        a = b;
    }
}

fn build_grid<'a, I, G, It>(
    geometry: &G,
    points: It,
    edges: &[Edge<I>],
    edge_length: I::WideUInt,
) -> Vec<IntPoint<I>>
where
    I: OverlayInt + 'a,
    G: IntPointContainment<I> + ?Sized,
    It: Iterator<Item = &'a IntPoint<I>>,
{
    let Some(rect) = IntRect::with_iter(points) else {
        return Vec::new();
    };

    let step = I::Wide::from_uint(edge_length);
    if step <= I::Wide::ONE {
        return Vec::new();
    }

    let row_step = (step * I::Wide::from_u32(TRIANGLE_HEIGHT_NUMERATOR)
        + I::Wide::from_u32(1 << (TRIANGLE_HEIGHT_SHIFT - 1)))
        >> TRIANGLE_HEIGHT_SHIFT;
    if row_step <= I::Wide::ZERO {
        return Vec::new();
    }

    let half_step = step / I::Wide::TWO;
    let min_x = rect.min_x.to_wide();
    let max_x = rect.max_x.to_wide();
    let max_y = rect.max_y.to_wide();

    let mut candidates = Vec::new();
    let mut row = 0usize;
    let mut y = rect.min_y.to_wide() + row_step / I::Wide::TWO;

    while y < max_y {
        let row_offset = if row & 1 == 0 { half_step } else { step };
        let mut x = min_x + row_offset;

        while x < max_x {
            candidates.push(IntPoint::new(I::from_wide(x), I::from_wide(y)));
            x = x + step;
        }

        row += 1;
        y = y + row_step;
    }

    let contains = geometry.contains_points(&candidates);
    let contained = candidates
        .into_iter()
        .zip(contains)
        .filter_map(|(point, is_inside)| is_inside.then_some(point))
        .collect();

    let third = edge_length / I::WideUInt::from_u64(3);
    let clearance = if third == I::WideUInt::ZERO {
        I::WideUInt::ONE
    } else {
        third
    };

    filter_near_edges(contained, edges, rect, edge_length, clearance)
}

fn filter_near_edges<I: OverlayInt>(
    points: Vec<IntPoint<I>>,
    edges: &[Edge<I>],
    rect: IntRect<I>,
    cell_size: I::WideUInt,
    clearance: I::WideUInt,
) -> Vec<IntPoint<I>> {
    if points.is_empty() || edges.is_empty() {
        return points;
    }

    let origin_x = rect.min_x.to_wide();
    let origin_y = rect.min_y.to_wide();
    let limit_x = rect.max_x.to_wide();
    let limit_y = rect.max_y.to_wide();
    let cell_size_wide = I::Wide::from_uint(cell_size);
    let clearance_wide = I::Wide::from_uint(clearance);

    // (cell_y, cell_x, edge_index). Key sorting groups edge references by cell
    // without relying on hashing in this no_std crate.
    let mut cell_edges = Vec::new();
    for (edge_index, edge) in edges.iter().enumerate() {
        let min_x = (edge.a.x.min(edge.b.x).to_wide() - clearance_wide).max(origin_x);
        let max_x = (edge.a.x.max(edge.b.x).to_wide() + clearance_wide).min(limit_x);
        let min_y = (edge.a.y.min(edge.b.y).to_wide() - clearance_wide).max(origin_y);
        let max_y = (edge.a.y.max(edge.b.y).to_wide() + clearance_wide).min(limit_y);

        let min_cell_x = ((min_x - origin_x) / cell_size_wide).to_usize();
        let max_cell_x = ((max_x - origin_x) / cell_size_wide).to_usize();
        let min_cell_y = ((min_y - origin_y) / cell_size_wide).to_usize();
        let max_cell_y = ((max_y - origin_y) / cell_size_wide).to_usize();

        for cell_y in min_cell_y..=max_cell_y {
            for cell_x in min_cell_x..=max_cell_x {
                cell_edges.push((cell_y, cell_x, edge_index));
            }
        }
    }
    cell_edges.sort_by_two_keys(false, |entry| entry.0, |entry| entry.1);

    points
        .into_iter()
        .filter(|point| {
            let cell_x = ((point.x.to_wide() - origin_x) / cell_size_wide).to_usize();
            let cell_y = ((point.y.to_wide() - origin_y) / cell_size_wide).to_usize();
            let cell = (cell_y, cell_x);
            let start = cell_edges.partition_point(|entry| (entry.0, entry.1) < cell);
            let end = cell_edges.partition_point(|entry| (entry.0, entry.1) <= cell);

            !cell_edges[start..end]
                .iter()
                .any(|entry| is_close_to_edge(*point, edges[entry.2], clearance))
        })
        .collect()
}

#[inline]
fn is_close_to_edge<I: OverlayInt>(
    point: IntPoint<I>,
    edge: Edge<I>,
    clearance: I::WideUInt,
) -> bool {
    let ab = edge.b - edge.a;
    let ap = point - edge.a;
    let length_sqr = ab.sqr_length();
    let clearance_sqr = clearance * clearance;

    if length_sqr <= I::Wide::ZERO {
        return ap.sqr_length().to_uint() <= clearance_sqr;
    }

    let projection = ap.dot_product(ab);
    if projection <= I::Wide::ZERO {
        return ap.sqr_length().to_uint() <= clearance_sqr;
    }
    if projection >= length_sqr {
        return (point - edge.b).sqr_length().to_uint() <= clearance_sqr;
    }

    let cross = ab.cross_product(ap).unsigned_abs();
    let distance_product = <I::WideUInt as UIntNumber>::Product::multiply(cross, cross);
    let limit_product =
        <I::WideUInt as UIntNumber>::Product::multiply(clearance_sqr, length_sqr.to_uint());

    distance_product <= limit_product
}

#[cfg(test)]
mod tests {
    use super::{filter_near_edges, Edge, IntUniformGrid};
    use alloc::vec;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_float::int::rect::IntRect;

    #[test]
    fn square_grid_has_staggered_rows() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];

        let points = contour.uniform_grid(20u64);

        assert!(!points.is_empty());
        assert!(points
            .iter()
            .all(|p| 0 < p.x && p.x < 100 && 0 < p.y && p.y < 100));
        assert!(points.iter().any(|p| p.x == 10));
        assert!(points.iter().any(|p| p.x == 20));
    }

    #[test]
    fn shape_grid_excludes_hole() {
        let shape = vec![
            vec![
                IntPoint::new(0, 0),
                IntPoint::new(100, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
            ],
            vec![
                IntPoint::new(40, 40),
                IntPoint::new(40, 60),
                IntPoint::new(60, 60),
                IntPoint::new(60, 40),
            ],
        ];

        let points = shape.uniform_grid(10u64);

        assert!(!points.is_empty());
        assert!(points
            .iter()
            .all(|p| p.x <= 40 || 60 <= p.x || p.y <= 40 || 60 <= p.y));
    }

    #[test]
    fn grid_removes_points_in_edge_influence_across_cell_boundary() {
        let contour = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ];

        let points = contour.uniform_grid(20u64);

        assert!(points.iter().all(|p| 6 < p.x && p.x < 94));
        assert!(points.iter().all(|p| 6 < p.y && p.y < 94));
    }

    #[test]
    fn edge_influences_a_cell_it_does_not_cross() {
        let points = vec![IntPoint::new(9, 5), IntPoint::new(7, 5)];
        let edges = [Edge {
            a: IntPoint::new(10, 0),
            b: IntPoint::new(10, 10),
        }];

        let filtered = filter_near_edges(points, &edges, IntRect::new(0, 20, 0, 20), 10u64, 2u64);

        assert_eq!(filtered, vec![IntPoint::new(7, 5)]);
    }
}
