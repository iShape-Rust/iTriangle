use crate::advanced::delaunay::{DelaunayCondition, IntDelaunay};
use crate::geom::triangle::IntTriangle;
use crate::int::triangulation::RawIntTriangulation;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::string::line::IntLine;

#[derive(Clone, Copy, PartialEq, Eq)]
struct ConstraintEdge {
    a: usize,
    b: usize,
}

impl ConstraintEdge {
    #[inline]
    fn new(a: usize, b: usize) -> Self {
        if a < b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

pub(super) trait Constrain<I: IntNumber> {
    fn into_constrained_delaunay(self, constraints: &[IntLine<I>]) -> IntDelaunay<I>;
}

pub(super) fn constraint_points<I: IntNumber>(constraints: &[IntLine<I>]) -> Vec<IntPoint<I>> {
    let mut points = Vec::with_capacity(2 * constraints.len());
    for &line in constraints {
        if line[0] != line[1] {
            points.extend_from_slice(&line);
        }
    }
    points.sort_unstable();
    points.dedup();
    points
}

impl<I: IntNumber> Constrain<I> for RawIntTriangulation<I> {
    fn into_constrained_delaunay(mut self, constraints: &[IntLine<I>]) -> IntDelaunay<I> {
        let mut locked = Vec::new();
        let mut vertices = Vec::new();

        for &line in constraints {
            if line[0] == line[1] {
                continue;
            }

            self.vertices_on_line(line, &mut vertices);
            assert!(
                vertices.len() >= 2
                    && self.points[vertices[0]] == line[0].min(line[1])
                    && self.points[*vertices.last().unwrap()] == line[0].max(line[1]),
                "constraint endpoints must lie strictly inside the triangulated geometry"
            );

            for pair in vertices.windows(2) {
                let edge = ConstraintEdge::new(pair[0], pair[1]);
                self.recover_edge(edge, &locked);
                if !locked.contains(&edge) {
                    locked.push(edge);
                }
            }
        }

        self.refine_delaunay(&locked);

        IntDelaunay {
            triangles: self.triangles,
            points: self.points,
        }
    }
}

impl<I: IntNumber> RawIntTriangulation<I> {
    fn vertices_on_line(&self, line: IntLine<I>, result: &mut Vec<usize>) {
        result.clear();
        let min = line[0].min(line[1]);
        let max = line[0].max(line[1]);

        for (index, &point) in self.points.iter().enumerate() {
            if min <= point
                && point <= max
                && Triangle::area_two(line[0], line[1], point) == I::Wide::ZERO
            {
                result.push(index);
            }
        }

        result.sort_unstable_by_key(|&index| self.points[index]);
    }

    fn recover_edge(&mut self, constraint: ConstraintEdge, locked: &[ConstraintEdge]) {
        if self.has_edge(constraint) {
            return;
        }

        let flip_limit = self.triangles.len().saturating_mul(self.triangles.len()) + 1;
        for _ in 0..flip_limit {
            let mut candidate = None;

            'scan: for triangle_index in 0..self.triangles.len() {
                let neighbors = self.triangles[triangle_index].neighbors;
                for neighbor_index in neighbors {
                    if neighbor_index <= triangle_index || neighbor_index >= self.triangles.len() {
                        continue;
                    }

                    let abc = self.triangles[triangle_index].abc_by_neighbor(neighbor_index);
                    let edge = ConstraintEdge::new(abc.v1.vertex.index, abc.v2.vertex.index);
                    if locked.contains(&edge) {
                        continue;
                    }

                    let p0 = self.points[constraint.a];
                    let p1 = self.points[constraint.b];
                    if proper_intersection(p0, p1, abc.v1.vertex.point, abc.v2.vertex.point)
                        && can_flip(&self.triangles, triangle_index, neighbor_index)
                    {
                        let pcb = self.triangles[neighbor_index].abc_by_neighbor(triangle_index);
                        if !proper_intersection(p0, p1, abc.v0.vertex.point, pcb.v0.vertex.point) {
                            candidate = Some((triangle_index, neighbor_index));
                            break 'scan;
                        }
                    }
                }
            }

            if let Some((triangle_index, neighbor_index)) = candidate {
                flip(&mut self.triangles, triangle_index, neighbor_index);
                if self.has_edge(constraint) {
                    return;
                }
            } else {
                break;
            }
        }

        panic!("unable to recover constraint edge; constraints must not intersect");
    }

    #[inline]
    fn has_edge(&self, edge: ConstraintEdge) -> bool {
        self.triangles.iter().any(|triangle| {
            let mut has_a = false;
            let mut has_b = false;
            for vertex in triangle.vertices {
                has_a |= vertex.index == edge.a;
                has_b |= vertex.index == edge.b;
            }
            has_a && has_b
        })
    }

    fn refine_delaunay(&mut self, locked: &[ConstraintEdge]) {
        loop {
            let mut candidate = None;

            'scan: for triangle_index in 0..self.triangles.len() {
                let neighbors = self.triangles[triangle_index].neighbors;
                for neighbor_index in neighbors {
                    if neighbor_index <= triangle_index || neighbor_index >= self.triangles.len() {
                        continue;
                    }

                    let abc = self.triangles[triangle_index].abc_by_neighbor(neighbor_index);
                    let edge = ConstraintEdge::new(abc.v1.vertex.index, abc.v2.vertex.index);
                    if locked.contains(&edge)
                        || !can_flip(&self.triangles, triangle_index, neighbor_index)
                    {
                        continue;
                    }

                    let pcb = self.triangles[neighbor_index].abc_by_neighbor(triangle_index);
                    if !DelaunayCondition::is_flip_not_required(
                        pcb.v0.vertex.point,
                        abc.v0.vertex.point,
                        abc.v1.vertex.point,
                        abc.v2.vertex.point,
                    ) {
                        candidate = Some((triangle_index, neighbor_index));
                        break 'scan;
                    }
                }
            }

            if let Some((triangle_index, neighbor_index)) = candidate {
                flip(&mut self.triangles, triangle_index, neighbor_index);
            } else {
                return;
            }
        }
    }
}

#[inline]
fn proper_intersection<I: IntNumber>(
    a: IntPoint<I>,
    b: IntPoint<I>,
    c: IntPoint<I>,
    d: IntPoint<I>,
) -> bool {
    opposite_signs(Triangle::area_two(a, b, c), Triangle::area_two(a, b, d))
        && opposite_signs(Triangle::area_two(c, d, a), Triangle::area_two(c, d, b))
}

#[inline]
fn opposite_signs<W: WideIntNumber>(a: W, b: W) -> bool {
    a < W::ZERO && b > W::ZERO || a > W::ZERO && b < W::ZERO
}

#[inline]
fn can_flip<I: IntNumber>(
    triangles: &[IntTriangle<I>],
    triangle_index: usize,
    neighbor_index: usize,
) -> bool {
    let abc = triangles[triangle_index].abc_by_neighbor(neighbor_index);
    let pcb = triangles[neighbor_index].abc_by_neighbor(triangle_index);
    opposite_signs(
        Triangle::area_two(
            abc.v0.vertex.point,
            pcb.v0.vertex.point,
            abc.v1.vertex.point,
        ),
        Triangle::area_two(
            abc.v0.vertex.point,
            pcb.v0.vertex.point,
            abc.v2.vertex.point,
        ),
    )
}

fn flip<I: IntNumber>(
    triangles: &mut [IntTriangle<I>],
    triangle_index: usize,
    neighbor_index: usize,
) {
    let abc = triangles[triangle_index].abc_by_neighbor(neighbor_index);
    let pcb = triangles[neighbor_index].abc_by_neighbor(triangle_index);

    update_neighbor(triangles, abc.v1.neighbor, triangle_index, neighbor_index);
    update_neighbor(triangles, pcb.v1.neighbor, neighbor_index, triangle_index);

    let abp = &mut triangles[triangle_index];
    abp.neighbors[abc.v0.position] = pcb.v1.neighbor;
    abp.neighbors[abc.v1.position] = neighbor_index;
    abp.neighbors[abc.v2.position] = abc.v2.neighbor;
    abp.vertices[abc.v2.position] = pcb.v0.vertex;

    let pca = &mut triangles[neighbor_index];
    pca.neighbors[pcb.v0.position] = abc.v1.neighbor;
    pca.neighbors[pcb.v1.position] = triangle_index;
    pca.neighbors[pcb.v2.position] = pcb.v2.neighbor;
    pca.vertices[pcb.v2.position] = abc.v0.vertex;
}

#[inline]
fn update_neighbor<I: IntNumber>(
    triangles: &mut [IntTriangle<I>],
    neighbor_index: usize,
    old_index: usize,
    new_index: usize,
) {
    if neighbor_index >= triangles.len() {
        return;
    }

    let triangle = &mut triangles[neighbor_index];
    if triangle.neighbors[0] == old_index {
        triangle.neighbors[0] = new_index;
    } else if triangle.neighbors[1] == old_index {
        triangle.neighbors[1] = new_index;
    } else {
        debug_assert_eq!(triangle.neighbors[2], old_index);
        triangle.neighbors[2] = new_index;
    }
}
