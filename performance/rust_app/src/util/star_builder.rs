use std::f32::consts::PI;

pub(crate) struct StarBuilder {}

impl StarBuilder {

    pub(crate) fn generate_star(
        radius: f32,
        scale: f32,
        points_per_corner: usize,
        corners_count: usize,
        direction: bool,
        center: [f32; 2]
    ) -> Vec<[f32; 2]> {
        let points_count: usize = points_per_corner * corners_count;
        let mut points = Vec::with_capacity(points_count);
        let sign = if direction { 1.0 } else { -1.0 };
        let da = sign * 2.0 * PI / points_count as f32;
        let w = corners_count as f32;
        let mut a = 0.0f32;

        for _ in 0..points_count {
            let r = radius * (1.0f32 + scale * (w * a).cos());
            let (sn, cs) = a.sin_cos();
            let x = r * cs + center[0];
            let y = r * sn + center[1];

            a += da;

            points.push([x, y]);
        }
        points
    }
}
