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
                [8.0, 10.0],    // 1
                [7.0, 6.0],     // 2
                [9.0, 1.0],     // 3
                [13.0, -1.0],   // 4
                [17.0, 1.0],    // 5
                [26.0, -7.0],   // 6
                [14.0, -15.0],  // 7
                [0.0, -18.0],   // 8
                [-14.0, -15.0], // 9
                [-25.0, -7.0],  // 10
                [-18.0, 0.0],   // 11
                [-16.0, -3.0],  // 12
                [-13.0, -4.0],  // 13
                [-8.0, -2.0],   // 14
                [-6.0, 2.0],    // 15
                [-7.0, 6.0],    // 16
                [-10.0, 8.0],   // 17
            ],
            vec![
                // hole
                [2.0, 0.0],   // 18
                [-2.0, -2.0], // 19
                [-4.0, -5.0], // 20
                [-2.0, -9.0], // 21
                [2.0, -11.0], // 22
                [5.0, -9.0],  // 23
                [7.0, -5.0],  // 24
                [5.0, -2.0],  // 25
            ],
        ];

        let triangulation = shape.triangulate().to_triangulation::<u16>();

        println!("points: {:?}", triangulation.points);
        println!("indices: {:?}", triangulation.indices);

        let delaunay_triangulation: Triangulation<[f64; 2], u16> = shape.triangulate()
            .into_delaunay()
            .to_triangulation();

        println!("points: {:?}", delaunay_triangulation.points);
        println!("indices: {:?}", delaunay_triangulation.indices);

        let convex_polygons = shape.triangulate()
            .into_delaunay()
            .to_convex_polygons();

        println!("convex polygons: {:?}", convex_polygons);

        let tessellation: Triangulation<[f64; 2], u16> = shape.triangulate()
            .into_delaunay()
            .refine_with_circumcenters_by_obtuse_angle(0.0)
            .to_triangulation();

        println!("points: {:?}", tessellation.points);
        println!("indices: {:?}", tessellation.indices);

        let centroids = shape.triangulate()
            .into_delaunay()
            .refine_with_circumcenters_by_obtuse_angle(0.0)
            .to_centroid_net(0.0);

        println!("centroids: {:?}", centroids);
    }
}
