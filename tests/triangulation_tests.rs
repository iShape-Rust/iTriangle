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
}