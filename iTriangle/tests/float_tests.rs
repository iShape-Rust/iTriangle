#[cfg(test)]
mod tests {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::i_float::float::point::FloatPoint;
    use i_triangle::triangulation::float::FloatTriangulate;

    #[test]
    fn test_0() {
        let shape = [
            [
                FloatPoint::<f32>::new(1.0, 1.0),
                FloatPoint::<f32>::new(1.0, 4.0),
                FloatPoint::<f32>::new(2.0, 4.0),
                FloatPoint::<f32>::new(2.0, 3.0),
                FloatPoint::<f32>::new(3.0, 3.0),
                FloatPoint::<f32>::new(3.0, 1.0),
            ].to_vec(),
        ].to_vec();

        let triangulation = shape.to_triangulation(Some(FillRule::NonZero), 0.0);

        assert_eq!(triangulation.points.len(), 6);
        assert_eq!(triangulation.indices.len(), 12);
    }
}