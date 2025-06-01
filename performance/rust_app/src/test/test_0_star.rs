use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::hint::black_box;
use std::time::Instant;
use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedDelaunayExperiment, UncheckedRawExperiment};
/*

unchecked raw: 
4 - 0.042724
8 - 0.083704
16 - 0.174608
32 - 0.363115
64 - 0.769955
128 - 1.688746
256 - 3.465031
512 - 7.063979

unchecked delaunay: 
4 - 0.111433
8 - 0.217058
16 - 0.403747
32 - 0.773319
64 - 1.490260
128 - 2.911652
256 - 5.761777
512 - 11.478699

raw: 
4 - 0.079023
8 - 0.169321
16 - 0.373843
32 - 0.832105
64 - 2.077839
128 - 5.533498
256 - 16.054511
512 - 45.202637

delaunay: 
4 - 0.149286
8 - 0.303273
16 - 0.616759
32 - 1.253739
64 - 2.849603
128 - 6.883017
256 - 18.574095
512 - 49.509120


earcutr: 
4 - 0.057268
8 - 0.127877
16 - 0.290149
32 - 0.693053
64 - 1.699206
128 - 4.253521
256 - 11.460828
512 - 40.623172

*/

pub(crate) struct SimpleStarTest {
    pub(crate) radius: f64,
    pub(crate) angle_steps_count: usize,
    pub(crate) points_per_corner: usize,
    pub(crate) radius_steps_count: usize,
    pub(crate) min_radius_scale: f64,
    pub(crate) max_radius_scale: f64,
}

impl SimpleStarTest {
    pub(crate) fn run_unchecked_raw(&self, count: usize, repeat_count: usize) -> usize {
        self.run::<UncheckedRawExperiment>(count, repeat_count)
    }

    pub(crate) fn run_unchecked_delaunay(&self, count: usize, repeat_count: usize) -> usize {
        self.run::<UncheckedDelaunayExperiment>(count, repeat_count)
    }

    pub(crate) fn run_raw(&self, count: usize, repeat_count: usize) -> usize {
        self.run::<RawExperiment>(count, repeat_count)
    }

    pub(crate) fn run_delaunay(&self, count: usize, repeat_count: usize) -> usize {
        self.run::<DelaunayExperiment>(count, repeat_count)
    }

    fn run<E: Experiment>(&self, count: usize, repeat_count: usize) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut points = Vec::with_capacity(count_per_star);
        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();
        for _ in 0..repeat_count {

            let mut radius_scale = self.min_radius_scale;

            for _ in 0..self.radius_steps_count {
                // grow star
                let mut start_angle = 0.0;
                for _ in 0..self.angle_steps_count {
                    // rotate star
                    StarBuilder::fill_star(
                        self.radius,
                        radius_scale,
                        start_angle,
                        self.points_per_corner,
                        count,
                        true,
                        &mut points,
                    );
                    sum += black_box(E::run_contour(&points));
                    start_angle += angle_step;
                }
                radius_scale += radius_step;
            }            
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64() / repeat_count as f64;

        println!("{} - {:.6}", count, time);
        sum
    }
}

// earcutr
impl SimpleStarTest {
    pub(crate) fn run_earcutr(&self, count: usize) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut points = Vec::with_capacity(2 * count_per_star);
        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let mut radius_scale = self.min_radius_scale;
        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();
        for _ in 0..self.radius_steps_count {
            // grow star
            let mut start_angle = 0.0;
            for _ in 0..self.angle_steps_count {
                // rotate star
                StarBuilder::fill_star_flat(
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    count,
                    true,
                    &mut points,
                );
                sum += black_box(Self::ear_cut(&points));
                start_angle += angle_step;
            }
            radius_scale += radius_step;
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64();

        println!("{} - {:.6}", count, time);
        sum
    }

    #[inline]
    fn ear_cut(points: &[f64]) -> usize {
        earcutr::earcut(points,&[],2).unwrap().len()
    }    
}

impl SimpleStarTest {
    pub(crate) fn run_triangulator(&self, count: usize, repeat_count: usize, delaunay: bool) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut points = Vec::with_capacity(count_per_star);
        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();
        let mut triangulator = Triangulator::<u32>::new(points.len(), Default::default(), Default::default());
        triangulator.delaunay(delaunay);
        let mut triangulation = Triangulation::with_capacity(points.len());
        for _ in 0..repeat_count {

            let mut radius_scale = self.min_radius_scale;

            for _ in 0..self.radius_steps_count {
                // grow star
                let mut start_angle = 0.0;
                for _ in 0..self.angle_steps_count {
                    // rotate star
                    StarBuilder::fill_star(
                        self.radius,
                        radius_scale,
                        start_angle,
                        self.points_per_corner,
                        count,
                        true,
                        &mut points,
                    );
                    triangulator.triangulate_into(&points, &mut triangulation);
                    start_angle += angle_step;
                    sum += triangulation.points.len();
                }
                radius_scale += radius_step;
            }
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64() / repeat_count as f64;

        println!("{} - {:.6}", count, time);
        sum
    }
}

impl SimpleStarTest {
    pub(crate) fn run_unchecked_triangulator(&self, count: usize, repeat_count: usize, delaunay: bool) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut points = Vec::with_capacity(count_per_star);
        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();
        let mut triangulator = Triangulator::<u32>::new(points.len(), Default::default(), Default::default());
        triangulator.delaunay(delaunay);
        let mut triangulation = Triangulation::with_capacity(points.len());
        for _ in 0..repeat_count {

            let mut radius_scale = self.min_radius_scale;

            for _ in 0..self.radius_steps_count {
                // grow star
                let mut start_angle = 0.0;
                for _ in 0..self.angle_steps_count {
                    // rotate star
                    StarBuilder::fill_star(
                        self.radius,
                        radius_scale,
                        start_angle,
                        self.points_per_corner,
                        count,
                        true,
                        &mut points,
                    );
                    triangulator.uncheck_triangulate_into(&points, &mut triangulation);
                    start_angle += angle_step;
                    sum += triangulation.indices.len();
                }
                radius_scale += radius_step;
            }
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64() / repeat_count as f64;

        println!("{} - {:.6}", count, time);
        sum
    }
}