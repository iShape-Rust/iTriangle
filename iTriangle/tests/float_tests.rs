#[cfg(test)]
mod tests {
    use i_overlay::i_float::float::point::FloatPoint;
    use i_triangle::float::triangulatable::Triangulatable;

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

        let triangulation = shape.triangulate().to_triangulation();

        assert_eq!(triangulation.points.len(), 6);
        assert_eq!(triangulation.indices.len(), 12);
    }
}