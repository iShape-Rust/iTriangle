use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::time::Instant;
use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedExperiment};
/*
unchecked:
4 - 0.047844
8 - 0.098785
16 - 0.210214
32 - 0.455352
64 - 0.987139
128 - 2.107867
256 - 4.481360
512 - 9.372721

raw:
4 - 0.074298
8 - 0.165002
16 - 0.369625
32 - 0.875200
64 - 2.241044
128 - 5.867757
256 - 16.652639
512 - 46.585559

delaunay:
4 - 0.139643
8 - 0.284868
16 - 0.591876
32 - 1.256099
64 - 2.873866
128 - 7.073528
256 - 18.894593
512 - 50.478053
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
    pub(crate) fn run_unchecked(&self, count: usize) -> usize {
        self.run::<UncheckedExperiment>(count)
    }

    pub(crate) fn run_raw(&self, count: usize) -> usize {
        self.run::<RawExperiment>(count)
    }

    pub(crate) fn run_delaunay(&self, count: usize) -> usize {
        self.run::<DelaunayExperiment>(count)
    }

    fn run<E: Experiment>(&self, count: usize) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut points = Vec::with_capacity(count_per_star);
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
                StarBuilder::fill_star(
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    count,
                    true,
                    &mut points,
                );
                sum += E::run_contour(&points);
                start_angle += angle_step;
            }
            radius_scale += radius_step;
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64();

        println!("{} - {:.6}", count, time);
        sum
    }
}
