#[cfg(test)]
mod tests {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::float::simplify::SimplifyShape;
    use i_overlay::i_shape::base::data::Contour;
    use i_overlay::i_shape::float::area::Area;
    use i_triangle::float::triangulatable::Triangulatable;
    use i_triangle::float::triangulation::Triangulation;
    use i_triangle::float::triangulator::Triangulator;
    use rand::Rng;

    #[test]
    fn test_0() {
        let shape = [[
            [1.0, 1.0],
            [1.0, 4.0],
            [2.0, 4.0],
            [2.0, 3.0],
            [3.0, 3.0],
            [3.0, 1.0],
        ]
        .to_vec()]
        .to_vec();

        let triangulation = shape.triangulate().to_triangulation::<u8>();

        assert_eq!(triangulation.points.len(), 6);
        assert_eq!(triangulation.indices.len(), 12);
    }

    #[test]
    fn test_1() {
        let contour = [
            [0.0, 2.0],
            [0.0, 0.0],
            [4.0, 2.0],
            [4.0, 0.0]
        ];

        let mut triangulator = Triangulator::<u32>::default();

        let t0 = triangulator.triangulate(&contour, false);
        let t1 = triangulator.triangulate(&contour, true);

        let area = contour.simplify_shape(FillRule::NonZero).area();

        t0.validate(area, 0.001);
        t1.validate(area, 0.001);
    }

    #[test]
    fn test_2() {
        let contour = [
            [0.0, 3.0],
            [-4.0, -3.0],
            [4.0, -3.0],
            [3.0, -3.0],
            [0.0, 3.0]
        ];

        let simple = contour.simplify_shape(FillRule::NonZero);
        let area = simple.area();
        
        let mut triangulator = Triangulator::<u32>::default();

        let t0 = triangulator.triangulate(&contour, false);
        let t1 = triangulator.triangulate(&contour, true);

        

        t0.validate(area, 0.001);
        t1.validate(area, 0.001);
    }
    
    #[test]
    fn test_3() {
        let contour = [
            [2.0, 3.0],
            [2.0, -2.0],
            [0.0, 3.0],
            [-1.0, 4.0],
            [0.0, 1.0]
        ];

        let mut triangulator = Triangulator::<u32>::default();

        let t0 = triangulator.triangulate(&contour, false);
        let t1 = triangulator.triangulate(&contour, true);

        let area = contour.simplify_shape(FillRule::NonZero).area();

        t0.validate(area, 0.001);
        t1.validate(area, 0.001);
    }

    #[test]
    fn test_random_0() {
        let mut triangulator = Triangulator::<u32>::default();

        let mut t0 = Triangulation::with_capacity(8);
        let mut t1 = Triangulation::with_capacity(8);

        for _ in 0..100_000 {
            let contour = random(8, 5);
            triangulator.triangulate_into(&contour, false, &mut t0);
            triangulator.triangulate_into(&contour, true, &mut t1);

            let area = contour.simplify_shape(FillRule::NonZero).area();

            t0.validate(area, 0.001);
            t1.validate(area, 0.001);
        }
    }

    fn random(radius: i32, n: usize) -> Contour<[f32; 2]> {
        let a = radius / 2;
        let mut points = Vec::with_capacity(n);
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a) as f32;
            let y = rng.random_range(-a..=a) as f32;
            points.push([x, y]);
        }

        points
    }
}
