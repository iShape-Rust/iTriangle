use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::IntContour;

pub(crate) struct MeshMeta {
    pub(crate) triangles_count: usize,
    pub(crate) vertices_count: usize,
}

pub(crate) trait TrianglesCount {
    fn triangles_count(&self, points_count: usize) -> usize;
}

impl TrianglesCount for FlatContoursBuffer {
    #[inline]
    fn triangles_count(&self, points_count: usize) -> usize {
        let mut count = 2 * points_count;
        count += self.points.len();
        count -= 2 * self.ranges.len();
        count
    }
}

impl TrianglesCount for [IntContour] {
    #[inline]
    fn triangles_count(&self, points_count: usize) -> usize {
        let mut count = 2 * points_count;
        for contour in self.iter() {
            count += contour.len() - 2;
        }
        count
    }
}

impl TrianglesCount for [IntPoint] {
    #[inline]
    fn triangles_count(&self, points_count: usize) -> usize {
        self.len() - 2 + 2 * points_count
    }
}

pub(crate) trait MeshMetaProvider {
    fn meta(&self, points_count: usize) -> MeshMeta;
}

impl MeshMetaProvider for [IntPoint] {
    #[inline]
    fn meta(&self, points_count: usize) -> MeshMeta {
        MeshMeta {
            triangles_count: self.triangles_count(points_count),
            vertices_count: self.len() + points_count,
        }
    }
}

impl MeshMetaProvider for [IntContour] {
    #[inline]
    fn meta(&self, points_count: usize) -> MeshMeta {
        let mut triangles_count = 2 * points_count;
        let mut vertices_count = points_count;
        for contour in self {
            triangles_count += contour.len() - 2;
            vertices_count += contour.len();
        }

        MeshMeta {
            triangles_count,
            vertices_count,
        }
    }
}