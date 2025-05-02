use std::f64::consts::PI;

pub(crate) struct StarBuilder {}

impl StarBuilder {
    pub(crate) fn fill_star(
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        points: &mut Vec<[f64; 2]>,
    ) {
        let points_count: usize = points_per_corner * corners_count;
        let da = 2.0 * PI / points_count as f64;
        let w = corners_count as f64;
        let mut a = 0.0f64;

        points.clear();

        for _ in 0..points_count {
            let r = radius * (1.0 + radius_scale * (w * a).cos());
            let (sn, cs) = (a + start_angle).sin_cos();
            let x = r * cs;
            let y = r * sn;

            a += da;

            points.push([x, y]);
        }
    }
}
