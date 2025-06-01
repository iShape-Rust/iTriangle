use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use std::f32::consts::PI;
use std::hint::black_box;
use std::time::Instant;
use crate::test::test::Test;

pub(crate) struct SpikeTest {
    pub(crate) inner_radius: f32,
    pub(crate) outer_radius: f32,
}

impl SpikeTest {
    pub(crate) fn run_triangle(&self, test: &Test, delaunay: bool, earcut: bool) -> usize {
        let contour = Self::contour(test.count, self.inner_radius, self.outer_radius);

        let start = Instant::now();

        let mut triangulator = Triangulator::<u32>::default();
        triangulator.delaunay(delaunay);
        triangulator.earcut(earcut);

        let mut sum = 0;

        let mut triangulation = Triangulation::with_capacity(test.count);

        for _ in 0..test.repeat {
            black_box(triangulator.triangulate_into(&contour, &mut triangulation));
            sum += triangulation.indices.len();
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64() / test.repeat as f64;

        println!("{} - {:.8}", test.count, time);
        sum
    }
}

// earcutr
impl SpikeTest {
    pub(crate) fn run_earcutr(&self, test: &Test) -> usize {
        let contour: Vec<f32> = Self::contour(test.count, self.inner_radius, self.outer_radius)
            .into_iter()
            .flat_map(|p| p)
            .collect();

        let start = Instant::now();

        let mut sum = 0;

        for _ in 0..test.repeat {
            let triangulation = black_box(earcutr::earcut(&contour, &[], 2)).unwrap();
            sum += triangulation.len();
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64() / test.repeat as f64;

        println!("{} - {:.8}", test.count, time);
        sum
    }
}

impl SpikeTest {
    fn contour(count: usize, r0: f32, r1: f32) -> Vec<[f32; 2]> {
        let n = count / 2;
        let da = PI / (n as f32);
        let mut a = 0.0f32;

        let mut path = Vec::with_capacity(count);
        for _ in 0..n {
            let (s, c) = a.sin_cos();
            let x = c * r0;
            let y = s * r0;
            path.push([x, y]);

            a += da;

            let (s, c) = a.sin_cos();
            let x = c * r1;
            let y = s * r1;

            path.push([x, y]);

            a += da;
        }

        path
    }
}
