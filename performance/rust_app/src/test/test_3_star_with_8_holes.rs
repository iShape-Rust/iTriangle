use std::f32::consts::PI;
use crate::util::star_builder::StarBuilder;

pub(crate) struct StarWith8HolesTest;

impl StarWith8HolesTest {
    pub(crate) fn resource(count: usize) -> Vec<Vec<[f32; 2]>> {
        Self::star(count)
    }

    pub(crate) fn points(count: usize) -> Vec<f32> {
        Self::star(count)
            .into_iter()
            .flatten()
            .flat_map(|p| p)
            .collect()
    }
}

impl StarWith8HolesTest {
    fn star(count: usize) -> Vec<Vec<[f32; 2]>> {
        let corners_count = 8;
        let holes = 8;
        let main_points_count = count / 2;
        let holes_points_count = count - main_points_count;
        let main_points_per_corner = main_points_count / corners_count;
        let hole_points_per_corner = holes_points_count / (corners_count * holes);

        let main = StarBuilder::generate_star(
            80.0,
            0.3,
            main_points_per_corner,
            corners_count,
            true,
            [0.0, 0.0],
        );

        let mut shape = vec![main];

        let r = 50.0f32;
        let n = holes as f32;
        for i in 0..holes {
            let a = i as f32 * 2.0 * PI / n;
            let sc = a.sin_cos();
            let x = r * sc.1;
            let y = r * sc.0;
            let hole = StarBuilder::generate_star(
                10.0,
                0.3,
                hole_points_per_corner,
                corners_count,
                false,
                [x, y],
            );
            shape.push(hole);
        }

        shape
    }
}
