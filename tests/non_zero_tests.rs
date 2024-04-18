#[cfg(test)]
mod tests {
    use i_float::fix_vec::FixVec;
    use i_overlay::core::fill_rule::FillRule;
    use i_shape::fix_shape::FixShape;
    use i_triangle::triangulation::triangulate::Triangulate;

    #[test]
    fn test_0() {

        let shapes =
            [
                square_shape(FixVec::new_f64(-10.0, -10.0)),
                square_shape(FixVec::new_f64(-10.0,  0.0)),
                square_shape(FixVec::new_f64(-10.0,  10.0)),
                square_shape(FixVec::new_f64(0.0, -10.0)),
                square_shape(FixVec::new_f64( 0.0,  10.0)),
                square_shape(FixVec::new_f64(10.0, -10.0)),
                square_shape(FixVec::new_f64(10.0,  0.0)),
                square_shape(FixVec::new_f64( 10.0,  10.0))
            ].to_vec();

        let triangulation = shapes.to_triangulation(Some(FillRule::NonZero));

        assert_eq!(triangulation.indices.len() / 3, 8);
    }

    fn square(pos: FixVec) -> Vec<FixVec> {
        [
            FixVec::new_f64(-5.0, -5.0) + pos,
            FixVec::new_f64(-5.0,  5.0) + pos,
            FixVec::new_f64( 5.0,  5.0) + pos,
            FixVec::new_f64( 5.0, -5.0) + pos
        ].to_vec()
    }

    fn square_shape(pos: FixVec) -> FixShape {
        FixShape::new_with_contour(square(pos))
    }

}