use std::f64::consts::PI;

pub(crate) struct StarBuilder {}

impl StarBuilder {
    pub(crate) fn fill_star(
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        direction: bool,
        points: &mut Vec<[f64; 2]>,
    ) {
        points.clear();
        Self::fill_star_contour(
            [0.0, 0.0],
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            direction,
            points,
        );
    }

    pub(crate) fn fill_star_with_hole(
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        contours: &mut Vec<Vec<[f64; 2]>>,
    ) {
        contours[0].clear();
        contours[1].clear();
        Self::fill_star_contour(
            [0.0, 0.0],
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            &mut contours[0],
        );
        Self::fill_star_contour(
            [0.0, 0.0],
            0.5 * radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            false,
            &mut contours[1],
        );
    }

    pub(crate) fn fill_star_contour(
        center: [f64; 2],
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        direction: bool,
        points: &mut Vec<[f64; 2]>,
    ) {
        let points_count: usize = points_per_corner * corners_count;
        let sign = if direction { 1.0 } else { -1.0 };
        let da = sign * 2.0 * PI / points_count as f64;
        let w = corners_count as f64;
        let mut a = 0.0f64;

        for _ in 0..points_count {
            let r = radius * (1.0 + radius_scale * (w * a).cos());
            let (sn, cs) = (a + start_angle).sin_cos();
            let x = r * cs + center[0];
            let y = r * sn + center[1];

            a += da;

            points.push([x, y]);
        }
    }

    pub(crate) fn fill_star_flat(
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        direction: bool,
        points: &mut Vec<f64>,
    ) {
        points.clear();
        Self::fill_star_contour_flat(
            [0.0, 0.0],
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            direction,
            points,
        );
    }

    pub(crate) fn fill_star_contour_flat(
        center: [f64; 2],
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        direction: bool,
        points: &mut Vec<f64>,
    ) {
        let points_count: usize = points_per_corner * corners_count;
        let sign = if direction { 1.0 } else { -1.0 };
        let da = sign * 2.0 * PI / points_count as f64;
        let w = corners_count as f64;
        let mut a = 0.0f64;

        for _ in 0..points_count {
            let r = radius * (1.0 + radius_scale * (w * a).cos());
            let (sn, cs) = (a + start_angle).sin_cos();
            let x = r * cs + center[0];
            let y = r * sn + center[1];

            a += da;

            points.push(x);
            points.push(y);
        }
    }

    pub(crate) fn fill_star_with_hole_flat(
        radius: f64,
        radius_scale: f64,
        start_angle: f64,
        points_per_corner: usize,
        corners_count: usize,
        shape: &mut Vec<f64>,
    ) {
        shape.clear();
        Self::fill_star_contour_flat(
            [0.0, 0.0],
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            shape,
        );
        Self::fill_star_contour_flat(
            [0.0, 0.0],
            0.5 * radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            shape,
        );
    }
}
