use i_shape::fix_shape::FixShape;
use i_shape::triangle::Triangle;
use crate::delaunay::vertex::DVertex;
use crate::monotone::mnav_node::MNavNode;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum MNodeType {
    End,
    Start,
    Merge,
    Split
}

impl MNodeType {

    fn to_usize(&self) -> usize {
        match self {
            MNodeType::End => 0,
            MNodeType::Start => 1,
            MNodeType::Merge => 2,
            MNodeType::Split => 3
        }
    }
}
#[derive(Clone, Debug, Copy)]
pub struct MSpecialNode {
    pub index: usize,
    pub node_type: MNodeType,
    pub sort: i64
}

impl MSpecialNode {

    pub (crate) fn node_type(&self) -> MNodeType {
        self.node_type
    }
}

pub struct NodeLayout {
    pub nav_nodes: Vec<MNavNode>,
    pub spec_nodes: Vec<MSpecialNode>
}

pub trait ShapeNodeLayout {
    fn node_layout(&self) -> NodeLayout;
}

impl ShapeNodeLayout for FixShape {

    fn node_layout(&self) -> NodeLayout {
        let mut n = 0;
        for path in self.paths.iter() {
            n += path.len();
        }

        let mut verts = vec!(MNavNode::EMPTY; n);
        let mut nodes = Vec::new();

        let mut s = 0;
        for path in self.paths.iter() {
            let mut i0 = path.len() - 2;

            let mut p0 = path[i0];

            let mut i1 = i0 + 1;

            let mut p1 = path[i1];

            for i2 in 0..path.len() {

                let i = i1 + s;

                let p2 = path[i2];

                let b0 = p0.bit_pack();
                let b1 = p1.bit_pack();
                let b2 = p2.bit_pack();

                let c0 = b0 > b1 && b1 < b2;
                let c1 = b0 < b1 && b1 > b2;

                if c0 || c1 {
                    let is_cw = Triangle::is_clockwise(p0, p1, p2);
                    let node_type = if c0 {
                        if is_cw { MNodeType::Start } else { MNodeType::Split }
                    } else {
                        if is_cw { MNodeType::End } else { MNodeType::Merge }
                    };
                    nodes.push(MSpecialNode { index: i, node_type, sort: b1 })
                }

                unsafe {
                    *verts.get_unchecked_mut(i) = MNavNode {
                        next: i2 + s,
                        index: i,
                        prev: i0 + s,
                        vert: DVertex::new(i, p1)
                    };
                }

                i0 = i1;
                i1 = i2;

                p0 = p1;
                p1 = p2;
            }

            s += path.len();
        }

        nodes.sort_by(|a, b| {
            if a.sort != b.sort {
                a.sort.cmp(&b.sort)
            } else {
                a.node_type.to_usize().cmp(&b.node_type.to_usize())
            }
        });

        NodeLayout{ nav_nodes: verts, spec_nodes: nodes }
    }

}