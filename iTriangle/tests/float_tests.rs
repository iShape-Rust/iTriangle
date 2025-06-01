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

        triangulator.delaunay(false);
        let t0 = triangulator.triangulate(&contour);

        triangulator.delaunay(true);
        let t1 = triangulator.triangulate(&contour);

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

        triangulator.delaunay(false);
        let t0 = triangulator.triangulate(&contour);

        triangulator.delaunay(true);
        let t1 = triangulator.triangulate(&contour);

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

        triangulator.delaunay(false);
        let t0 = triangulator.triangulate(&contour);

        triangulator.delaunay(true);
        let t1 = triangulator.triangulate(&contour);

        let area = contour.simplify_shape(FillRule::NonZero).area();

        t0.validate(area, 0.001);
        t1.validate(area, 0.001);
    }

    #[test]
    fn test_random_0() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..20_000 {
            let contour = random(8, 5);
            let area = contour.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);
        }
    }

    #[test]
    fn test_random_1() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..20_000 {
            let contour = random(10, 6);
            let area = contour.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);
        }
    }

    #[test]
    fn test_random_2() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..20_000 {
            let contour = random(10, 12);
            let area = contour.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);
        }
    }

    #[test]
    fn test_random_3() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..10_000 {
            let contour = random(20, 20);
            let area = contour.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);
        }
    }

    #[test]
    fn test_random_4() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..1_000 {
            let contour = random(30, 50);
            let area = contour.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&contour, &mut t);
            t.validate(area, 0.001);
        }
    }

    #[test]
    fn test_random_5() {
        let mut triangulator = Triangulator::<u32>::default();
        let mut t = Triangulation::with_capacity(8);

        for _ in 0..500 {
            let main = random(50, 20);
            let mut shape = vec![main];
            for _ in 0..10 {
                shape.push(random(30, 5));
            }

            let area = shape.simplify_shape(FillRule::NonZero).area();

            triangulator.delaunay(false);
            triangulator.triangulate_into(&shape, &mut t);
            t.validate(area, 0.001);

            triangulator.delaunay(true);
            triangulator.triangulate_into(&shape, &mut t);
            t.validate(area, 0.001);
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
