use crate::util::star_builder::StarBuilder;
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::unchecked::UncheckedTriangulatable;
use std::f64::consts::PI;
use std::time::Instant;

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


pub(crate) struct SimpleStarTest {}

impl SimpleStarTest {
    pub(crate) fn run_unchecked(count: usize) -> usize {
        Self::run::<UncheckedStarTest>(count)
    }

    pub(crate) fn run_raw(count: usize) -> usize {
        Self::run::<RawStarTest>(count)
    }

    pub(crate) fn run_delaunay(count: usize) -> usize {
        Self::run::<DelaunayStarTest>(count)
    }

    fn run<E: Experimentation>(count: usize) -> usize {
        let mut points = Vec::with_capacity(2 * count);

        let angle_step_count = 100;
        let angle_step = 2.0 * PI / angle_step_count as f64;

        let radius = 100.0;
        let radius_step = 0.01;
        let points_per_corner = 10;

        let start = Instant::now();

        let mut sum = 0;

        let mut radius_scaler = 0.0;
        while radius_scaler < 1.0 {
            // grow star
            let mut start_angle = 0.0;
            for _ in 0..angle_step_count {
                // rotate star
                StarBuilder::fill_star(radius, radius_scaler, start_angle, points_per_corner, count, &mut points);
                sum += E::run(&points);
                start_angle += angle_step;
            }
            radius_scaler += radius_step;
        }

        let duration = start.elapsed();
        let time = duration.as_secs_f64();

        println!("{} - {:.6}", count, time);
        sum
    }
}

trait Experimentation {
    fn run(points: &Vec<[f64; 2]>) -> usize;
}

struct UncheckedStarTest {}

impl Experimentation for UncheckedStarTest {
    #[inline]
    fn run(points: &Vec<[f64; 2]>) -> usize {
        points.unchecked_triangulate().triangle_indices().len()
    }
}

struct RawStarTest {}

impl Experimentation for RawStarTest {
    #[inline]
    fn run(points: &Vec<[f64; 2]>) -> usize {
        points.triangulate().triangle_indices().len()
    }
}

struct DelaunayStarTest {}

impl Experimentation for DelaunayStarTest {
    #[inline]
    fn run(points: &Vec<[f64; 2]>) -> usize {
        points
            .triangulate()
            .into_delaunay()
            .triangle_indices()
            .len()
    }
}
