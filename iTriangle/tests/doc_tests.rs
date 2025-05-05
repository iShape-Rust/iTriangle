#[cfg(test)]
mod tests {
    use i_triangle::float::triangulatable::Triangulatable;
    use i_triangle::float::triangulation::Triangulation;

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
}
