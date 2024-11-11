#[cfg(test)]
mod tests {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::i_float::int::point::IntPoint;
    use i_triangle::triangulation::int::IntTriangulate;

    #[test]
    fn test_0() {
        let shapes =
            [
                square(IntPoint::new(-10, -10)),
                square(IntPoint::new(-10, 0)),
                square(IntPoint::new(-10, 10)),
                square(IntPoint::new(0, -10)),
                square(IntPoint::new(0, 10)),
                square(IntPoint::new(10, -10)),
                square(IntPoint::new(10, 0)),
                square(IntPoint::new(10, 10))
            ].to_vec();

        let triangulation = shapes.to_triangulation(Some(FillRule::NonZero), 0);

        assert_eq!(triangulation.indices.len() / 3, 8);
    }

    fn square(pos: IntPoint) -> Vec<Vec<IntPoint>> {
        [[
            IntPoint::new(-5 + pos.x, 5 + pos.y),
            IntPoint::new(-5 + pos.x, 5 + pos.y),
            IntPoint::new(5 + pos.x, 5 + pos.y),
            IntPoint::new(5 + pos.x, -5 + pos.y)
        ].to_vec()].to_vec()
    }
}