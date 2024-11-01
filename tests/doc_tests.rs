#[cfg(test)]
mod tests {
    use i_float::float::point::FloatPoint;
    use i_overlay::core::fill_rule::FillRule;
    use i_triangle::triangulation::float::FloatTriangulate;

    #[test]
    fn test_0() {
        let shape = [
            [ // body
                FloatPoint::new(0.0, 20.0),    // 0
                FloatPoint::new(8.0, 10.0),    // 1
                FloatPoint::new(7.0, 6.0),     // 2
                FloatPoint::new(9.0, 1.0),     // 3
                FloatPoint::new(13.0, -1.0),   // 4
                FloatPoint::new(17.0, 1.0),    // 5
                FloatPoint::new(26.0, -7.0),   // 6
                FloatPoint::new(14.0, -15.0),  // 7
                FloatPoint::new(0.0, -18.0),   // 8
                FloatPoint::new(-14.0, -15.0), // 9
                FloatPoint::new(-25.0, -7.0),  // 10
                FloatPoint::new(-18.0, 0.0),   // 11
                FloatPoint::new(-16.0, -3.0),  // 12
                FloatPoint::new(-13.0, -4.0),  // 13
                FloatPoint::new(-8.0, -2.0),   // 14
                FloatPoint::new(-6.0, 2.0),    // 15
                FloatPoint::new(-7.0, 6.0),    // 16
                FloatPoint::new(-10.0, 8.0)    // 17
            ].to_vec(),
            [ // hole
                FloatPoint::new(2.0, 0.0),     // 18
                FloatPoint::new(-2.0, -2.0),   // 19
                FloatPoint::new(-4.0, -5.0),   // 20
                FloatPoint::new(-2.0, -9.0),   // 21
                FloatPoint::new(2.0, -11.0),   // 22
                FloatPoint::new(5.0, -9.0),    // 23
                FloatPoint::new(7.0, -5.0),    // 24
                FloatPoint::new(5.0, -2.0)     // 25
            ].to_vec()
        ].to_vec();

        let triangulation = shape.to_triangulation(Some(FillRule::NonZero), 0.0);

        println!("points: {:?}", triangulation.points);
        println!("indices: {:?}", triangulation.indices);
    }
}