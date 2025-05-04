use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::unchecked::UncheckedTriangulatable;

pub(super) trait Experiment {
    fn run_contour(contour: &Vec<[f64; 2]>) -> usize;
    fn run_shape(shape: &Vec<Vec<[f64; 2]>>) -> usize;
}

pub(super) struct UncheckedRawExperiment {}
pub(super) struct UncheckedDelaunayExperiment {}
pub(super) struct RawExperiment {}
pub(super) struct DelaunayExperiment {}

impl Experiment for UncheckedRawExperiment {
    #[inline]
    fn run_contour(contour: &Vec<[f64; 2]>) -> usize {
        contour.unchecked_triangulate().triangle_indices().len()
    }

    #[inline]
    fn run_shape(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape.unchecked_triangulate().triangle_indices().len()
    }
}

impl Experiment for UncheckedDelaunayExperiment {
    #[inline]
    fn run_contour(contour: &Vec<[f64; 2]>) -> usize {
        contour
            .unchecked_triangulate()
            .into_delaunay()
            .triangle_indices()
            .len()
    }

    #[inline]
    fn run_shape(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape
            .unchecked_triangulate()
            .into_delaunay()
            .triangle_indices()
            .len()
    }
}

impl Experiment for RawExperiment {
    #[inline]
    fn run_contour(contour: &Vec<[f64; 2]>) -> usize {
        contour.triangulate().triangle_indices().len()
    }

    #[inline]
    fn run_shape(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape.triangulate().triangle_indices().len()
    }
}

impl Experiment for DelaunayExperiment {
    #[inline]
    fn run_contour(contour: &Vec<[f64; 2]>) -> usize {
        contour
            .triangulate()
            .into_delaunay()
            .triangle_indices()
            .len()
    }

    #[inline]
    fn run_shape(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape.triangulate().into_delaunay().triangle_indices().len()
    }
}
