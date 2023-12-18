#[cfg(test)]
mod tests {
    use i_float::fix_vec::FixVec;
    use i_shape::fix_shape::FixShape;
    use i_triangle::monotone::node_layout::{MNodeType, ShapeNodeLayout};

    #[test]
    fn test_0() {
        let shape = FixShape::new(
            [[
                FixVec::new_f64(-15.0,-15.0),
                FixVec::new_f64(-25.0,  0.0),
                FixVec::new_f64(-15.0, 15.0),
                FixVec::new_f64(15.0,  15.0),
                FixVec::new_f64(25.0,   0.0),
                FixVec::new_f64(15.0, -15.0)
            ].to_vec()].to_vec()
        );

        let layout = shape;

        let nodes = layout.node_layout().spec_nodes;

        assert_eq!(nodes.len(), 2);

        assert_eq!(nodes[0].node_type, MNodeType::Start);
        assert_eq!(nodes[1].node_type, MNodeType::End);

    }
}