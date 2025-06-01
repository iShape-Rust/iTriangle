use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use std::hint::black_box;
use std::time::Instant;
use crate::test::test::Test;

pub(crate) struct SpiralTest {
    pub(crate) width: f32,
}

impl SpiralTest {
    pub(crate) fn run_triangle(&self, test: &Test, delaunay: bool, earcut: bool) -> usize {
        let contour = Self::contour(test.count, self.width);

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
impl SpiralTest {
    pub(crate) fn run_earcutr(&self, test: &Test) -> usize {
        let contour: Vec<f32> = Self::contour(test.count, self.width)
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

impl SpiralTest {
    fn contour(count: usize, s: f32) -> Vec<[f32; 2]> {
        let mut path_0: Vec<[f32; 2]> = Vec::with_capacity(count);
        let mut path_1: Vec<[f32; 2]> = Vec::with_capacity(count / 2);

        let mut s0 = s;
        let mut s1 = 2.0 * s;

        let mut x0 = 0.0f32;
        let mut y0 = 0.0f32;

        let mut x1 = 0.0f32;
        let mut y1 = 0.0f32;

        y0 += s0;
        path_0.push([x0, y0]);

        x0 += s0;
        path_0.push([x0, y0]);

        path_1.push([x1, y1]);

        x1 += s1;
        path_1.push([x1, y1]);
        s1 += s;

        let n = count - 4;

        for i in 0..n / 2 {
            match i % 4 {
                0 => {
                    y0 += s0;
                    y1 += s1;
                }
                1 => {
                    x0 -= s0;
                    x1 -= s1;
                }
                2 => {
                    y0 -= s0;
                    y1 -= s1;
                }
                _ => {
                    x0 += s0;
                    x1 += s1;
                }
            }
            path_0.push([x0, y0]);
            path_1.push([x1, y1]);

            s0 += s;
            s1 += s;
        }

        path_0.extend(path_1.into_iter().rev());
        path_0
    }
}
