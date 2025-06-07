use crate::util::star_builder::StarBuilder;

pub(crate) struct StarWithHoleTest;

impl StarWithHoleTest {
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

impl StarWithHoleTest {
    fn star(count: usize) -> Vec<Vec<[f32; 2]>> {
        let corners_count = 8;
        let points_per_corner = count / 8;
        let main = StarBuilder::generate_star(
            80.0,
            0.3,
            points_per_corner,
            corners_count,
            true,
            [0.0, 0.0],
        );
        let hole = StarBuilder::generate_star(
            40.0,
            0.3,
            points_per_corner,
            corners_count,
            false,
            [0.0, 0.0],
        );

        vec![main, hole]
    }
}
