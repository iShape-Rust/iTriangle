#[cfg(test)]
mod tests {
    use i_overlay::i_float::int::point::IntPoint;
    use i_triangle::monotone::node_layout::{MNodeType, ShapeNodeLayout};

    #[test]
    fn test_0() {
        let shape =
            [[
                IntPoint::new(-15, -15),
                IntPoint::new(-25, 0),
                IntPoint::new(-15, 15),
                IntPoint::new(15, 15),
                IntPoint::new(25, 0),
                IntPoint::new(15, -15)
            ].to_vec()].to_vec();

        let nodes = shape.node_layout().spec_nodes;

        assert_eq!(nodes.len(), 2);

        assert_eq!(nodes[0].node_type, MNodeType::Start);
        assert_eq!(nodes[1].node_type, MNodeType::End);
    }
}