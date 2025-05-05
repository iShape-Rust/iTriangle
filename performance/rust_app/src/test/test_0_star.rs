use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::hint::black_box;
use std::time::Instant;
use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedDelaunayExperiment, UncheckedRawExperiment};
/*
unchecked raw: 
4 - 0.042951
8 - 0.085169
16 - 0.174965
32 - 0.361303
64 - 0.760934
128 - 1.606473
256 - 3.380065
512 - 6.979404

unchecked delaunay: 
4 - 0.111137
8 - 0.216192
16 - 0.400449
32 - 0.764024
64 - 1.468326
128 - 2.893402
256 - 5.714521
512 - 11.353683

raw: 
4 - 0.081034
8 - 0.168758
16 - 0.374195
32 - 0.841226
64 - 2.103556
128 - 5.580456
256 - 16.298824
512 - 46.693433

delaunay: 
4 - 0.151569
8 - 0.300934
16 - 0.604100
32 - 1.264401
64 - 2.843721
128 - 6.927733
256 - 18.731435
512 - 51.036346


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
                sum += black_box(E::run_contour(&points));
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
