use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedExperiment};
use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::time::Instant;

/*
unchecked: 
4 - 0.003435
8 - 0.013662
16 - 0.062749
32 - 0.268670
64 - 1.343702
128 - 5.680361
256 - 25.201445

raw: 
4 - 0.009029
8 - 0.035041
16 - 0.152936
32 - 0.534618
64 - 2.542741
128 - 10.131110
256 - 45.647115

delaunay: 
4 - 0.037794
8 - 0.112566
16 - 0.313631
32 - 0.927747
64 - 4.774121
128 - 23.526240
256 - 196.863314

 */

pub(crate) struct RectStarHolesTest {
    pub(crate) radius: f64,
    pub(crate) angle_steps_count: usize,
    pub(crate) points_per_corner: usize,
    pub(crate) radius_steps_count: usize,
    pub(crate) min_radius_scale: f64,
    pub(crate) max_radius_scale: f64,
    pub(crate) corners_count: usize,
}

impl RectStarHolesTest {
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
        let count_per_star = self.points_per_corner * self.corners_count;
        let mut shape = vec![Vec::with_capacity(count_per_star); count * count + 1];

        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let mut radius_scale = self.min_radius_scale;
        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();

        while radius_scale < self.max_radius_scale {
            // grow star
            let mut start_angle = 0.0;
            for _ in 0..self.angle_steps_count {
                // rotate star
                self.fill_rect_shape(radius_scale, start_angle, count, &mut shape);
                sum += E::run_shape(&shape);
                start_angle += angle_step;
            }
            radius_scale += radius_step;
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64();

        println!("{} - {:.6}", count, time);
        sum
    }

    fn fill_rect_shape(
        &self,
        radius_scale: f64,
        start_angle: f64,
        count: usize,
        shape: &mut Vec<Vec<[f64; 2]>>,
    ) {
        let dx = 4.0 * self.radius;
        let dy = dx;

        let w = dx * count as f64;
        let h = w;

        let rect = &mut shape[0];
        rect.clear();
        
        rect.push([0.0, 0.0]);
        rect.push([w, 0.0]);
        rect.push([w, h]);
        rect.push([0.0, h]);

        let mut x = 0.5 * dx;
        let mut i = 1;
        for _ in 0..count {
            let mut y = 0.5 * dy;
            for _ in 0..count {
                let contour = &mut shape[i];
                contour.clear();
                StarBuilder::fill_star_contour(
                    [x, y],
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    self.corners_count,
                    false,
                    contour,
                );

                i += 1;
                y += dy;
            }
            x += dx;
        }
    }
}
