use crate::test::experiment::{DelaunayExperiment, Experiment, RawExperiment, UncheckedDelaunayExperiment, UncheckedRawExperiment};
use crate::util::star_builder::StarBuilder;
use std::f64::consts::PI;
use std::hint::black_box;
use std::time::Instant;
use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
/*
unchecked raw: 
4 - 0.002339
8 - 0.008906
16 - 0.044372
32 - 0.186836
64 - 0.786142
128 - 4.351395
256 - 18.935817

unchecked delaunay: 
4 - 0.008275
8 - 0.024921
16 - 0.116687
32 - 0.510489
64 - 2.617753
128 - 18.809027
256 - 149.748440

raw: 
4 - 0.009139
8 - 0.036900
16 - 0.142827
32 - 0.460254
64 - 1.985195
128 - 9.719830
256 - 37.087321

delaunay: 
4 - 0.042705
8 - 0.104350
16 - 0.323554
32 - 0.936904
64 - 4.231544
128 - 24.878510
256 - 179.928663



earcutr:
4 - 0.037897
8 - 0.076405
16 - 0.605979
32 - 7.496754
64 - 210.383871

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
        let count_per_star = self.points_per_corner * self.corners_count;
        let mut shape = vec![Vec::with_capacity(count_per_star); count * count + 1];

        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();
        
        for _ in 0..repeat_count {
            let mut radius_scale = self.min_radius_scale;
            while radius_scale < self.max_radius_scale {
                // grow star
                let mut start_angle = 0.0;
                for _ in 0..self.angle_steps_count {
                    // rotate star
                    self.fill_rect_shape(radius_scale, start_angle, count, &mut shape);
                    sum += black_box(E::run_shape(&shape));
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

    fn fill_rect_shape(
        &self,
        radius_scale: f64,
        start_angle: f64,
        count: usize,
        shape: &mut [Vec<[f64; 2]>],
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

impl RectStarHolesTest {
    pub(crate) fn run_earcutr(&self, count: usize) -> usize {
        let count_per_star = self.points_per_corner * self.corners_count;
        let capacity = 2 * (count * count * count_per_star + 4);
        let mut shape = Vec::with_capacity(capacity);
        let mut hole_indices = Vec::with_capacity(count * count);

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
                self.fill_rect_shape_flat(radius_scale, start_angle, count, &mut hole_indices, &mut shape);
                sum += black_box(self.ear_cut(&hole_indices, &shape));
                start_angle += angle_step;
            }
            radius_scale += radius_step;
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64();

        println!("{} - {:.6}", count, time);
        sum
    }

    fn fill_rect_shape_flat(
        &self,
        radius_scale: f64,
        start_angle: f64,
        count: usize,
        hole_indices: &mut Vec<usize>,
        shape: &mut Vec<f64>,
    ) {
        hole_indices.clear();
        shape.clear();

        let dx = 4.0 * self.radius;
        let dy = dx;

        let w = dx * count as f64;
        let h = w;

        shape.extend_from_slice(&[
            0.0, 0.0,
            w, 0.0,
            w, h,
            0.0, h
        ]);

        let mut x = 0.5 * dx;
        for _ in 0..count {
            let mut y = 0.5 * dy;
            for _ in 0..count {
                hole_indices.push(shape.len() / 2);
                StarBuilder::fill_star_contour_flat(
                    [x, y],
                    self.radius,
                    radius_scale,
                    start_angle,
                    self.points_per_corner,
                    self.corners_count,
                    true,
                    shape,
                );

                y += dy;
            }
            x += dx;
        }
    }

    #[inline]
    fn ear_cut(&self, indices: &[usize], points: &[f64]) -> usize {
        earcutr::earcut(points, indices, 2).unwrap().len()
    }
}

impl RectStarHolesTest {
    pub(crate) fn run_triangulator(&self, count: usize, repeat_count: usize, delaunay: bool) -> usize {
        let count_per_star = self.points_per_corner * self.corners_count;
        let mut shape = vec![Vec::with_capacity(count_per_star); count * count + 1];

        let mut sum = 0;

        let angle_step = 2.0 * PI / self.angle_steps_count as f64;

        let radius_step =
            (self.max_radius_scale - self.min_radius_scale) / self.radius_steps_count as f64;

        let start = Instant::now();

        let max_capacity = count_per_star * shape.len();
        let mut triangulator = Triangulator::<u32>::new(max_capacity, Default::default(), Default::default());
        let mut triangulation = Triangulation::with_capacity(max_capacity);

        for _ in 0..repeat_count {
            let mut radius_scale = self.min_radius_scale;
            while radius_scale < self.max_radius_scale {
                // grow star
                let mut start_angle = 0.0;
                for _ in 0..self.angle_steps_count {
                    // rotate star
                    self.fill_rect_shape(radius_scale, start_angle, count, &mut shape);
                    triangulator.triangulate_into(&shape, delaunay, &mut triangulation);
                    sum += triangulation.points.len();

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