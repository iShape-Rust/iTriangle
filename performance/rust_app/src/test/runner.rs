use std::hint::black_box;
use std::time::Instant;
use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use i_triangle::i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_triangle::i_overlay::i_float::float::number::FloatNumber;
use i_triangle::i_overlay::i_shape::source::resource::ShapeResource;
use crate::test::test::TestData;

pub(crate) struct Runner;

impl Runner {

    pub(crate) fn run_triangle<R, P, T>(resource: &R, test: &TestData, delaunay: bool, earcut: bool) -> usize
    where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let start = Instant::now();

        let mut triangulator = Triangulator::<u32>::default();
        triangulator.delaunay(delaunay);
        triangulator.earcut(earcut);

        let mut sum = 0;

        let mut triangulation = Triangulation::with_capacity(test.count);

        for _ in 0..test.repeat {
            black_box(triangulator.uncheck_triangulate_into(resource, &mut triangulation));
            sum += triangulation.indices.len();
        }

        let duration = start.elapsed();
        let time = 1000_000.0 * duration.as_secs_f64() / test.repeat as f64;

        println!("{} - {:.8}", test.count, time);
        sum
    }

    pub(crate) fn run_earcut(points: Vec<f32>, test: &TestData) -> usize {
        let start = Instant::now();

        let mut sum = 0;

        for _ in 0..test.repeat {
            let triangulation = black_box(earcutr::earcut(&points, &[], 2)).unwrap();
            sum += triangulation.len();
        }

        let duration = start.elapsed();
        let time = 1000_000.0 * duration.as_secs_f64() / test.repeat as f64;

        println!("{} - {:.8}", test.count, time);
        sum
    }

}