use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedDelaunayExperiment, UncheckedRawExperiment};
use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::hint::black_box;
use std::time::Instant;
/*
unchecked raw: 
4 - 0.094759
8 - 0.186354
16 - 0.386123
32 - 0.785403
64 - 1.683104
128 - 3.545771
256 - 7.736618

unchecked delaunay: 
4 - 0.201063
8 - 0.381374
16 - 0.720753
32 - 1.494862
64 - 2.701149
128 - 5.387367
256 - 11.294545

raw: 
4 - 0.256936
8 - 0.552084
16 - 1.232090
32 - 3.129282
64 - 8.139909
128 - 22.668275
256 - 63.406176

delaunay: 
4 - 0.363410
8 - 0.754555
16 - 1.601335
32 - 3.790210
64 - 9.187079
128 - 24.597506
256 - 67.065579

earcutr: 
4 - 0.187054
8 - 0.492114
16 - 1.537897
32 - 6.108676
64 - 29.415327
128 - 201.363290

*/

pub(crate) struct StarWithHoleTest {
    pub(crate) angle_steps_count: usize,
    pub(crate) radius: f64,
    pub(crate) points_per_corner: usize,
    pub(crate) radius_steps_count: usize,
    pub(crate) min_radius_scale: f64,
    pub(crate) max_radius_scale: f64,
}

impl StarWithHoleTest {
    pub(crate) fn run_unchecked_raw(&self, count: usize) -> usize {
        self.run::<UncheckedRawExperiment>(count)
    }

    pub(crate) fn run_unchecked_delaunay(&self, count: usize) -> usize {
        self.run::<UncheckedDelaunayExperiment>(count)
    }

    pub(crate) fn run_raw(&self, count: usize) -> usize {
        self.run::<RawExperiment>(count)
    }

    pub(crate) fn run_delaunay(&self, count: usize) -> usize {
        self.run::<DelaunayExperiment>(count)
    }

    fn run<E: Experiment>(&self, count: usize) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut shape = vec![
            Vec::with_capacity(count_per_star),
            Vec::with_capacity(count_per_star),
        ];
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
                StarBuilder::fill_star_with_hole(
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    count,
                    &mut shape,
                );
                sum += black_box(E::run_shape(&shape));
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

impl StarWithHoleTest {
    pub(crate) fn run_earcutr(&self, count: usize) -> usize {
        let count_per_star = self.points_per_corner * count;
        let mut shape = Vec::with_capacity(2 * 2 * count_per_star);
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
                StarBuilder::fill_star_with_hole_flat(
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    count,
                    &mut shape,
                );

                sum += black_box(self.ear_cut(&shape));
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
    fn ear_cut(&self, points: &[f64]) -> usize {
        let points_count = points.len() / 2;
        let hole_index = points_count / 2;
        earcutr::earcut(points,&[hole_index],2).unwrap().len()
    }
}