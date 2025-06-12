#[cfg(test)]
mod tests {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::float::overlay::{FloatOverlay, OverlayOptions};
    use i_overlay::i_float::adapter::FloatPointAdapter;
    use i_overlay::i_float::float::rect::FloatRect;
    use i_overlay::i_shape::base::data::{Contour, Shapes};
    use rand::Rng;
    use i_triangle::float::triangulatable::Triangulatable;
    use i_triangle::float::triangulation::Triangulation;
    use i_triangle::float::triangulator::Triangulator;

    #[test]
    fn test_0() {
        let shape = vec![
            vec![
                // body
                [0.0, 20.0],    // 0
                [-10.0, 8.0],   // 1
                [-7.0, 6.0],    // 2
                [-6.0, 2.0],    // 3
                [-8.0, -2.0],   // 4
                [-13.0, -4.0],  // 5
                [-16.0, -3.0],  // 6
                [-18.0, 0.0],   // 7
                [-25.0, -7.0],  // 8
                [-14.0, -15.0], // 9
                [0.0, -18.0],   // 10
                [14.0, -15.0],  // 11
                [26.0, -7.0],   // 12
                [17.0, 1.0],    // 13
                [13.0, -1.0],   // 14
                [9.0, 1.0],     // 15
                [7.0, 6.0],     // 16
                [8.0, 10.0],    // 17
            ],
            vec![
                // hole
                [2.0, 0.0],   // 0
                [5.0, -2.0],  // 1
                [7.0, -5.0],  // 2
                [5.0, -9.0],  // 3
                [2.0, -11.0], // 4
                [-2.0, -9.0], // 5
                [-4.0, -5.0], // 6
                [-2.0, -2.0], // 7
            ],
        ];

        let triangulation = shape.triangulate().to_triangulation::<u16>();

        println!("points: {:?}", triangulation.points);
        println!("indices: {:?}", triangulation.indices);

        let delaunay_triangulation: Triangulation<[f64; 2], u16> =
            shape.triangulate().into_delaunay().to_triangulation();

        println!("points: {:?}", delaunay_triangulation.points);
        println!("indices: {:?}", delaunay_triangulation.indices);

        let convex_polygons = shape.triangulate().into_delaunay().to_convex_polygons();

        println!("convex polygons: {:?}", convex_polygons);

        let tessellation: Triangulation<[f64; 2], u16> = shape
            .triangulate()
            .into_delaunay()
            .refine_with_circumcenters_by_obtuse_angle(0.0)
            .to_triangulation();

        println!("points: {:?}", tessellation.points);
        println!("indices: {:?}", tessellation.indices);

        let centroids = shape
            .triangulate()
            .into_delaunay()
            .refine_with_circumcenters_by_obtuse_angle(0.0)
            .to_centroid_net(0.0);

        println!("centroids: {:?}", centroids);
    }

    #[test]
    fn test_1() {
        let contours = random_contours(100);

        let mut triangulator = Triangulator::<u32>::default();

        // apply Delaunay condition
        triangulator.delaunay(true);

        // use fast earcut solver for a small contours less 64 points
        triangulator.earcut(true);

        let mut triangulation = Triangulation::with_capacity(100);

        for contour in contours.iter() {
            triangulator.triangulate_into(contour, &mut triangulation);
            // do something with triangulation draw, accumulate, etc

            println!("points: {:?}", triangulation.points);
            println!("indices: {:?}", triangulation.indices);
        }
    }

    #[test]
    fn test_2() {
        let window = FloatRect::new(-100.0, 100.0, -100.0, 100.0);
        let adapter = FloatPointAdapter::new(window);

        let list_of_shapes = random_shapes(100, &adapter);

        let mut triangulator = Triangulator::<u32>::default();

        // apply Delaunay condition
        triangulator.delaunay(true);

        // use fast earcut solver for a small contours less 64 points
        triangulator.earcut(true);

        let mut triangulation = Triangulation::with_capacity(100);

        for shapes in list_of_shapes.iter() {
            // it safe to uncheck triangulate valid shapes
            triangulator.uncheck_triangulate_into(shapes, &mut triangulation);

            // do something with triangulation draw, accumulate, etc

            println!("points: {:?}", triangulation.points);
            println!("indices: {:?}", triangulation.indices);
        }
    }

    fn random_contours(count: usize) -> Vec<Contour<[f32; 2]>> {
        let mut contours = Vec::with_capacity(count);
        for _ in 0..count {
            contours.push(random(100.0, 100));
        }
        contours
    }

    fn random_shapes(count: usize, adapter: &FloatPointAdapter<[f32; 2], f32>) -> Vec<Shapes<[f32; 2]>> {
        let mut list = Vec::with_capacity(count);

        let mut opt = OverlayOptions::default();
        // important option to get a valid contours for triangulation
        opt.preserve_output_collinear = true;

        let mut overlay = FloatOverlay::new_custom(adapter.clone(), opt, Default::default(), 100);
        for _ in 0..count {
            // contour must be in adapter window!
            let contour = random(100.0, 100);
            overlay = overlay.unsafe_add_source(&contour, ShapeType::Subject);

            // get a valid geometry
            let shapes = overlay.overlay(OverlayRule::Subject, FillRule::NonZero);

            list.push(shapes);
        }

        list
    }

    fn random(radius: f32, n: usize) -> Contour<[f32; 2]> {
        let a = 0.5 * radius;
        let mut points = Vec::with_capacity(n);
        let mut rng = rand::rng();
        for _ in 0..n {
            let x = rng.random_range(-a..=a);
            let y = rng.random_range(-a..=a);
            points.push([x, y]);
        }

        points
    }

}
