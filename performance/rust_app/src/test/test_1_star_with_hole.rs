use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedDelaunayExperiment, UncheckedRawExperiment};
use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::hint::black_box;
use std::time::Instant;
/*
unchecked raw:
4 - 0.110370
8 - 0.229584
16 - 0.480577
32 - 1.031568
64 - 2.223870
128 - 4.708570
256 - 9.891213

unchecked delaunay:
4 - 0.212785
8 - 0.423034
16 - 0.830413
32 - 1.754029
64 - 3.289262
128 - 6.567074
256 - 13.818567

raw:
4 - 0.268433
8 - 0.593994
16 - 1.344304
32 - 3.294683
64 - 8.547062
128 - 24.225092
256 - 67.140800

delaunay:
4 - 0.369640
8 - 0.781789
16 - 1.689566
32 - 3.976463
64 - 9.689934
128 - 26.156747
256 - 71.023947

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