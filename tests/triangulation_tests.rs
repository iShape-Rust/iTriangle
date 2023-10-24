#[cfg(test)]
mod tests {
    use i_float::fix_vec::FixVec;
    use i_shape::fix_shape::FixShape;
    use i_triangle::triangulate::Triangulate;

    #[test]
    fn test_0() {
        let shape = FixShape::new_with_contour(
            [
                FixVec::new_i64(-100, -100),
                FixVec::new_i64(-100,  100),
                FixVec::new_i64(100, 100),
                FixVec::new_i64(100,  -100)
            ].to_vec()
        );

        let triangulation = shape.into_triangulation(false);
        assert_eq!(triangulation.points.len(), 4);

        let points_must_be = [
            FixVec::new_i64(-100, -100),
            FixVec::new_i64(-100,  100),
            FixVec::new_i64(100, 100),
            FixVec::new_i64(100,  -100)
        ].to_vec();

        let points_indices = [
            0, 1, 3, 1, 2, 3
        ].to_vec();

        assert_eq!(points_must_be, triangulation.points);
        assert_eq!(points_indices, triangulation.indices);
    }

    #[test]
    fn test_1() {
        let shape = FixShape::new_with_contour(
            [
                FixVec::new_i64(-15, 0),
                FixVec::new_i64(0, 15),
                FixVec::new_i64(15, 0),
                FixVec::new_i64(0, -15)
            ].to_vec()
        );

        let triangulation = shape.into_triangulation(false);

        let points_must_be = [
            FixVec::new_i64(-15, 0),
            FixVec::new_i64(0, 15),
            FixVec::new_i64(15, 0),
            FixVec::new_i64(0, -15)
        ].to_vec();

        let indices_must_be = [
            0, 1, 3, 3, 1, 2
        ].to_vec();

        assert_eq!(points_must_be, triangulation.points);
        assert_eq!(indices_must_be, triangulation.indices);
    }

    #[test]
    fn test_2() {
        let shape = FixShape::new_with_contour(
            [
                FixVec::new_i64(-15, -15),
                FixVec::new_i64(-25, 0),
                FixVec::new_i64(-15, 15),
                FixVec::new_i64(15, 15),
                FixVec::new_i64(25, 0),
                FixVec::new_i64(15, -15)
            ].to_vec()
        );

        let triangulation = shape.into_triangulation(false);

        let points_must_be = [
            FixVec::new_i64(-15, 0),
            FixVec::new_i64(0, 15),
            FixVec::new_i64(15, 0),
            FixVec::new_i64(0, -15)
        ].to_vec();

        let indices_must_be = [
            0, 1, 3, 3, 1, 2
        ].to_vec();

        assert_eq!(points_must_be, triangulation.points);
        assert_eq!(indices_must_be, triangulation.indices);
    }

    #[test]
    fn test_3() {
        let shape = FixShape::new_with_contour(
            [
                FixVec::new_i64(-5, -15),
                FixVec::new_i64(-10, 0),
                FixVec::new_i64(0, 15),
                FixVec::new_i64(10, 5),
                FixVec::new_i64(5, -10)
            ].to_vec()
        );

        let triangulation = shape.into_triangulation(false);

        let points_must_be = [
            FixVec::new_i64(-5, -15),
            FixVec::new_i64(-10, 0),
            FixVec::new_i64(0, 15),
            FixVec::new_i64(10, 5),
            FixVec::new_i64(5, -10)
        ].to_vec();

        let indices_must_be = [
            3, 1, 2, 1, 4, 0, 3, 4, 1
        ].to_vec();

        assert_eq!(points_must_be, triangulation.points);
        assert_eq!(indices_must_be, triangulation.indices);
    }

    #[test]
    fn test_4() {
        let shape = FixShape::new(
            [
                [
                    FixVec::new_i64(-20, -20),
                    FixVec::new_i64(-20, 20),
                    FixVec::new_i64(20, 20),
                    FixVec::new_i64(20, -20)
                ].to_vec(),
                [
                    FixVec::new_i64(-10, -10),
                    FixVec::new_i64(10, -10),
                    FixVec::new_i64(10, 10),
                    FixVec::new_i64(-10, 10)
                ].to_vec()
            ].to_vec()
        );

        let triangulation = shape.into_triangulation(false);

        let points_must_be = [
            FixVec::new_i64(-20, -20),
            FixVec::new_i64(-20, 20),
            FixVec::new_i64(20, 20),
            FixVec::new_i64(20, -20),
            FixVec::new_i64(-10, 10),
            FixVec::new_i64(10, 10),
            FixVec::new_i64(10, -10),
            FixVec::new_i64(-10, -10)
        ].to_vec();

        let indices_must_be = [
            0, 1, 3, 1, 2, 3, 7, 4, 6, 4, 5, 6
        ].to_vec();

        assert_eq!(points_must_be, triangulation.points);
        assert_eq!(indices_must_be, triangulation.indices);
    }

}