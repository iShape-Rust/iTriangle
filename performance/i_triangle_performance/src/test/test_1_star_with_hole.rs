use crate::util::star_builder::StarBuilder;
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::unchecked::UncheckedTriangulatable;
use std::f64::consts::PI;
use std::time::Instant;

/*
unchecked: 
4 - 0.110370
8 - 0.229584
16 - 0.480577
32 - 1.031568
64 - 2.223870
128 - 4.708570
256 - 9.891213

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
*/


pub(crate) struct StarWithHoleTest {}

impl StarWithHoleTest {
    pub(crate) fn run_unchecked(count: usize) -> usize {
        Self::run::<UncheckedStarWithHoleTest>(count)
    }

    pub(crate) fn run_raw(count: usize) -> usize {
        Self::run::<RawStarWithHoleTest>(count)
    }

    pub(crate) fn run_delaunay(count: usize) -> usize {
        Self::run::<DelaunayStarWithHoleTest>(count)
    }

    fn run<E: Experimentation>(count: usize) -> usize {
        let mut shape = vec![
            Vec::with_capacity(2 * count),
            Vec::with_capacity(2 * count)
        ];

        let angle_step_count = 100;
        let angle_step = 2.0 * PI / angle_step_count as f64;

        let radius = 100.0;
        let points_per_corner = 10;

        let start = Instant::now();

        let mut sum = 0;

        let min_radius_scaler = 0.1; // to prevent self intersection we will not start from 0
        let max_radius_scaler = 1.0; 
        
        let mut radius_scaler = min_radius_scaler;
        let radius_step = (max_radius_scaler - min_radius_scaler) / 100.0;
        
        while radius_scaler < max_radius_scaler {
            // grow star
            let mut start_angle = 0.0;
            for _ in 0..angle_step_count {
                // rotate star
                StarBuilder::fill_star_with_hole(radius, radius_scaler, start_angle, points_per_corner, count, &mut shape);
                sum += E::run(&shape);
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
    fn run(shape: &Vec<Vec<[f64; 2]>>) -> usize;
}

struct UncheckedStarWithHoleTest {}

impl Experimentation for UncheckedStarWithHoleTest {
    #[inline]
    fn run(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape.unchecked_triangulate().triangle_indices().len()
    }
}

struct RawStarWithHoleTest {}

impl Experimentation for RawStarWithHoleTest {
    #[inline]
    fn run(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape.triangulate().triangle_indices().len()
    }
}

struct DelaunayStarWithHoleTest {}

impl Experimentation for DelaunayStarWithHoleTest {
    #[inline]
    fn run(shape: &Vec<Vec<[f64; 2]>>) -> usize {
        shape
            .triangulate()
            .into_delaunay()
            .triangle_indices()
            .len()
    }
}
